// Example file demonstrating how to use the enhanced faculty scope authorization
// This file shows practical usage patterns for the new middleware functionality

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

// Import the new faculty scope functionality
use trackivity::middleware::session::{
    FacultyScopedAdminUser, AdminUser, SessionUser,
    has_faculty_access, validate_faculty_access, get_accessible_faculty_ids,
    require_faculty_access_from_path,
};

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

// === EXAMPLES USING FacultyScopedAdminUser EXTRACTOR ===

/// Get departments within a faculty
/// The FacultyScopedAdminUser extractor automatically validates:
/// - User is authenticated and is an admin
/// - User has access to the specified faculty
/// - SuperAdmin can access any faculty
/// - FacultyAdmin/RegularAdmin can only access their assigned faculty
pub async fn get_faculty_departments(
    faculty_scoped_admin: FacultyScopedAdminUser,
    Query(pagination): Query<PaginationQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id;
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    
    // Query departments for this faculty - no additional authorization needed!
    // The extractor already ensured we have access to this faculty
    let departments = query_departments_by_faculty(
        &app_state.db_pool, 
        faculty_id, 
        page, 
        per_page
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "faculty_id": faculty_id,
            "departments": departments,
            "pagination": {
                "page": page,
                "per_page": per_page
            }
        }
    })))
}

/// Create a new department within a faculty
pub async fn create_faculty_department(
    faculty_scoped_admin: FacultyScopedAdminUser,
    State(app_state): State<AppState>,
    Json(request): Json<CreateDepartmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id;
    let admin_level = &faculty_scoped_admin.admin_role.admin_level;
    
    // Create department - authorization is already handled by the extractor
    let department_id = create_department(
        &app_state.db_pool,
        faculty_id,
        &request.name,
        &request.code,
        request.description.as_deref(),
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "department_id": department_id,
            "faculty_id": faculty_id,
            "created_by": format!("{:?}", admin_level)
        },
        "message": "Department created successfully"
    })))
}

/// Update department within a faculty
pub async fn update_faculty_department(
    faculty_scoped_admin: FacultyScopedAdminUser,
    Path(department_id): Path<Uuid>,
    State(app_state): State<AppState>,
    Json(request): Json<UpdateDepartmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let faculty_id = faculty_scoped_admin.faculty_id;
    
    // Verify the department belongs to this faculty
    verify_department_belongs_to_faculty(&app_state.db_pool, department_id, faculty_id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Update department
    update_department(&app_state.db_pool, department_id, &request).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "department_id": department_id,
            "faculty_id": faculty_id
        },
        "message": "Department updated successfully"
    })))
}

// === EXAMPLES USING MANUAL VALIDATION WITH HELPER FUNCTIONS ===

/// Get faculty statistics with manual validation
/// This approach gives more control over the validation logic
pub async fn get_faculty_statistics(
    Path(faculty_id): Path<Uuid>,
    admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Manual validation using helper function
    validate_faculty_access(&admin_user.session_user, faculty_id)?;
    
    // Get statistics for this faculty
    let stats = get_faculty_stats(&app_state.db_pool, faculty_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "faculty_id": faculty_id,
            "stats": stats,
            "admin_level": format!("{:?}", admin_user.admin_role.admin_level)
        }
    })))
}

/// Bulk operation across multiple faculties
/// Demonstrates how to handle operations that might span multiple faculties
pub async fn bulk_faculty_operation(
    admin_user: AdminUser,
    State(app_state): State<AppState>,
    Json(faculty_ids): Json<Vec<Uuid>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accessible_faculties = get_accessible_faculty_ids(&admin_user.session_user);
    
    let processed_faculties = match accessible_faculties {
        None => {
            // SuperAdmin - can process all requested faculties
            process_all_faculties(&app_state.db_pool, &faculty_ids).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        Some(allowed_faculties) => {
            // Filter to only allowed faculties
            let filtered_ids: Vec<Uuid> = faculty_ids.into_iter()
                .filter(|id| allowed_faculties.contains(id))
                .collect();
            
            if filtered_ids.is_empty() {
                return Err(StatusCode::FORBIDDEN);
            }
            
            process_all_faculties(&app_state.db_pool, &filtered_ids).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "processed_count": processed_faculties.len(),
            "faculties": processed_faculties
        }
    })))
}

/// List all accessible faculties for the current admin
pub async fn list_my_accessible_faculties(
    admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> Json<serde_json::Value> {
    let accessible_faculty_ids = get_accessible_faculty_ids(&admin_user.session_user);
    
    match accessible_faculty_ids {
        None => {
            // SuperAdmin - get all faculties
            let all_faculties = get_all_faculties(&app_state.db_pool).await
                .unwrap_or_else(|_| vec![]);
            
            Json(json!({
                "status": "success",
                "data": {
                    "access_level": "superadmin",
                    "message": "SuperAdmin has access to all faculties",
                    "faculties": all_faculties,
                    "total_count": all_faculties.len()
                }
            }))
        }
        Some(faculty_ids) => {
            let faculties = get_faculties_by_ids(&app_state.db_pool, &faculty_ids).await
                .unwrap_or_else(|_| vec![]);
            
            Json(json!({
                "status": "success",
                "data": {
                    "access_level": "restricted",
                    "accessible_faculty_ids": faculty_ids,
                    "faculties": faculties,
                    "total_count": faculty_ids.len()
                }
            }))
        }
    }
}

// === CONDITIONAL ACCESS EXAMPLES ===

/// Handler that adapts behavior based on faculty access level
pub async fn adaptive_faculty_handler(
    Path(faculty_id): Path<Uuid>,
    session_user: SessionUser,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if has_faculty_access(&session_user, faculty_id) {
        // User has admin access to this faculty
        let admin_data = get_admin_faculty_data(&app_state.db_pool, faculty_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(json!({
            "status": "success",
            "access_level": "admin",
            "data": admin_data
        })))
    } else {
        // User has no admin access, provide public information only
        let public_data = get_public_faculty_data(&app_state.db_pool, faculty_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Json(json!({
            "status": "success",
            "access_level": "public",
            "data": public_data
        })))
    }
}

// === ROUTER SETUP ===

pub fn create_faculty_scope_router() -> Router<AppState> {
    Router::new()
        // Routes using FacultyScopedAdminUser extractor
        // These automatically validate faculty access
        .route(
            "/api/admin/faculties/:faculty_id/departments",
            get(get_faculty_departments).post(create_faculty_department)
        )
        .route(
            "/api/admin/faculties/:faculty_id/departments/:department_id",
            put(update_faculty_department).delete(delete_faculty_department)
        )
        
        // Routes using manual validation
        // These provide more flexibility for complex scenarios
        .route(
            "/api/admin/faculties/:faculty_id/statistics",
            get(get_faculty_statistics)
        )
        .route(
            "/api/admin/bulk-faculty-operation",
            post(bulk_faculty_operation)
        )
        .route(
            "/api/admin/my-faculties",
            get(list_my_accessible_faculties)
        )
        
        // Routes with conditional access
        .route(
            "/api/faculties/:faculty_id/info",
            get(adaptive_faculty_handler)
        )
        
        // Route with middleware-level faculty validation
        .route(
            "/api/admin/faculties/:faculty_id/protected-resource",
            get(protected_resource_handler)
        )
        .layer(axum::middleware::from_fn(require_faculty_access_from_path()))
}

/// Handler for route protected by faculty access middleware
pub async fn protected_resource_handler(
    Path(faculty_id): Path<Uuid>,
    session_user: SessionUser,
) -> Json<serde_json::Value> {
    // If we reach here, middleware already validated faculty access
    Json(json!({
        "status": "success",
        "message": "Access granted to protected resource",
        "faculty_id": faculty_id,
        "user_id": session_user.user_id
    }))
}

/// Placeholder delete handler
pub async fn delete_faculty_department(
    faculty_scoped_admin: FacultyScopedAdminUser,
    Path(department_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Implementation would go here
    Ok(Json(json!({
        "status": "success",
        "message": "Department deleted"
    })))
}

// === MOCK DATABASE FUNCTIONS ===
// These would be implemented in your actual database service

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
}

async fn query_departments_by_faculty(
    _pool: &sqlx::PgPool,
    _faculty_id: Uuid,
    _page: u64,
    _per_page: u64,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    // Mock implementation
    Ok(vec![])
}

async fn create_department(
    _pool: &sqlx::PgPool,
    _faculty_id: Uuid,
    _name: &str,
    _code: &str,
    _description: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    // Mock implementation
    Ok(Uuid::new_v4())
}

async fn update_department(
    _pool: &sqlx::PgPool,
    _department_id: Uuid,
    _request: &UpdateDepartmentRequest,
) -> Result<(), sqlx::Error> {
    // Mock implementation
    Ok(())
}

async fn verify_department_belongs_to_faculty(
    _pool: &sqlx::PgPool,
    _department_id: Uuid,
    _faculty_id: Uuid,
) -> Result<(), sqlx::Error> {
    // Mock implementation
    Ok(())
}

async fn get_faculty_stats(
    _pool: &sqlx::PgPool,
    _faculty_id: Uuid,
) -> Result<serde_json::Value, sqlx::Error> {
    // Mock implementation
    Ok(json!({"total_departments": 5, "total_students": 150}))
}

async fn process_all_faculties(
    _pool: &sqlx::PgPool,
    _faculty_ids: &[Uuid],
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    // Mock implementation
    Ok(vec![])
}

async fn get_all_faculties(
    _pool: &sqlx::PgPool,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    // Mock implementation
    Ok(vec![])
}

async fn get_faculties_by_ids(
    _pool: &sqlx::PgPool,
    _faculty_ids: &[Uuid],
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    // Mock implementation
    Ok(vec![])
}

async fn get_admin_faculty_data(
    _pool: &sqlx::PgPool,
    _faculty_id: Uuid,
) -> Result<serde_json::Value, sqlx::Error> {
    // Mock implementation
    Ok(json!({"admin_level_data": true}))
}

async fn get_public_faculty_data(
    _pool: &sqlx::PgPool,
    _faculty_id: Uuid,
) -> Result<serde_json::Value, sqlx::Error> {
    // Mock implementation
    Ok(json!({"public_data_only": true}))
}