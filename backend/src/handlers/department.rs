use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::middleware::session::{SessionState, FacultyAdminUser};
use crate::models::{
    department::Department,
    admin_role::AdminLevel,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

/// Get departments in a faculty with proper authorization
/// FacultyAdmin+ can access their faculty, SuperAdmin can access any
pub async fn get_faculty_departments(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    admin: FacultyAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check authorization - FacultyAdmin can only access their own faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only access departments in your faculty"
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Verify faculty exists
    let faculty_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM faculties WHERE id = $1)"
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await
    .unwrap_or(false);

    if !faculty_exists {
        let error_response = json!({
            "status": "error",
            "message": "Faculty not found"
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // Get departments
    let departments_result = sqlx::query_as::<_, Department>(
        "SELECT id, name, code, faculty_id, description, created_at, updated_at 
         FROM departments WHERE faculty_id = $1 ORDER BY name"
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match departments_result {
        Ok(departments) => {
            let response = json!({
                "status": "success",
                "data": {
                    "departments": departments,
                    "faculty_id": faculty_id,
                    "total_count": departments.len()
                },
                "message": "Departments retrieved successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch departments: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create department in faculty with proper authorization
/// FacultyAdmin+ for their faculty, SuperAdmin for any
pub async fn create_faculty_department(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    admin: FacultyAdminUser,
    Json(request): Json<CreateDepartmentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check authorization - FacultyAdmin can only create in their own faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only create departments in your faculty"
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Verify faculty exists
    let faculty_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM faculties WHERE id = $1)"
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await
    .unwrap_or(false);

    if !faculty_exists {
        let error_response = json!({
            "status": "error",
            "message": "Faculty not found"
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // Check if department code already exists in this faculty
    let existing_dept = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM departments WHERE faculty_id = $1 AND code = $2)"
    )
    .bind(faculty_id)
    .bind(&request.code)
    .fetch_one(&session_state.db_pool)
    .await
    .unwrap_or(false);

    if existing_dept {
        let error_response = json!({
            "status": "error",
            "message": "Department with this code already exists in the faculty"
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    // Create department
    let department_result = sqlx::query_as::<_, Department>(
        "INSERT INTO departments (name, code, faculty_id, description) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, name, code, faculty_id, description, created_at, updated_at"
    )
    .bind(&request.name)
    .bind(&request.code)
    .bind(faculty_id)
    .bind(&request.description)
    .fetch_one(&session_state.db_pool)
    .await;

    match department_result {
        Ok(department) => {
            let response = json!({
                "status": "success",
                "data": department,
                "message": "Department created successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create department: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Update department with proper authorization
/// FacultyAdmin+ for their faculty departments, SuperAdmin for any
pub async fn update_department(
    State(session_state): State<SessionState>,
    Path(department_id): Path<Uuid>,
    admin: FacultyAdminUser,
    Json(request): Json<UpdateDepartmentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // First, get the department to check faculty ownership
    let department_info = sqlx::query(
        "SELECT d.id, d.faculty_id, f.name as faculty_name 
         FROM departments d 
         JOIN faculties f ON d.faculty_id = f.id 
         WHERE d.id = $1"
    )
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let department_info = match department_info {
        Ok(Some(row)) => row,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Department not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch department: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let department_faculty_id: Uuid = department_info.get("faculty_id");

    // Check authorization - FacultyAdmin can only update departments in their faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(department_faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only update departments in your faculty"
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Build dynamic update query
    let mut query = "UPDATE departments SET updated_at = NOW()".to_string();
    let mut param_count = 1;
    let mut query_builder = sqlx::query_as::<_, Department>("");

    if let Some(_) = &request.name {
        query.push_str(&format!(", name = ${}", param_count));
        param_count += 1;
    }

    if let Some(_) = &request.code {
        query.push_str(&format!(", code = ${}", param_count));
        param_count += 1;
    }

    if let Some(_) = &request.description {
        query.push_str(&format!(", description = ${}", param_count));
        param_count += 1;
    }

    query.push_str(&format!(
        " WHERE id = ${} RETURNING id, name, code, faculty_id, description, created_at, updated_at",
        param_count
    ));

    query_builder = sqlx::query_as::<_, Department>(&query);

    if let Some(name) = &request.name {
        query_builder = query_builder.bind(name);
    }

    if let Some(code) = &request.code {
        query_builder = query_builder.bind(code);
    }

    if let Some(description) = &request.description {
        query_builder = query_builder.bind(description);
    }

    query_builder = query_builder.bind(department_id);

    match query_builder.fetch_one(&session_state.db_pool).await {
        Ok(department) => {
            let response = json!({
                "status": "success",
                "data": department,
                "message": "Department updated successfully"
            });
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Department not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update department: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Delete department with proper authorization
/// FacultyAdmin+ for their faculty departments, SuperAdmin for any
pub async fn delete_department(
    State(session_state): State<SessionState>,
    Path(department_id): Path<Uuid>,
    admin: FacultyAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // First, get the department to check faculty ownership
    let department_info = sqlx::query(
        "SELECT d.id, d.faculty_id, f.name as faculty_name 
         FROM departments d 
         JOIN faculties f ON d.faculty_id = f.id 
         WHERE d.id = $1"
    )
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let department_info = match department_info {
        Ok(Some(row)) => row,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Department not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch department: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let department_faculty_id: Uuid = department_info.get("faculty_id");

    // Check authorization - FacultyAdmin can only delete departments in their faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(department_faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only delete departments in your faculty"
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Check if department has users - prevent deletion if users exist
    let user_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE department_id = $1"
    )
    .bind(department_id)
    .fetch_one(&session_state.db_pool)
    .await
    .unwrap_or(0);

    if user_count > 0 {
        let error_response = json!({
            "status": "error",
            "message": format!("Cannot delete department: {} users are still assigned to this department", user_count)
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    // Delete department
    let delete_result = sqlx::query("DELETE FROM departments WHERE id = $1")
        .bind(department_id)
        .execute(&session_state.db_pool)
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response = json!({
                    "status": "error",
                    "message": "Department not found"
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            } else {
                let response = json!({
                    "status": "success",
                    "message": "Department deleted successfully"
                });
                Ok(Json(response))
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to delete department: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}


