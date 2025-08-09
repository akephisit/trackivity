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

/// Assign admin to department (Faculty Admin or SuperAdmin only)
#[derive(Debug, Serialize, Deserialize)]
pub struct AssignDepartmentAdminRequest {
    pub user_id: Uuid,
    pub admin_level: AdminLevel, // Only RegularAdmin allowed for department assignment
}

pub async fn assign_department_admin(
    State(session_state): State<SessionState>,
    Path(department_id): Path<Uuid>,
    _admin: FacultyAdminUser,
    Json(request): Json<AssignDepartmentAdminRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify department exists and get faculty_id
    let department_query = sqlx::query_as::<_, Department>(
        "SELECT id, name, code, faculty_id, description, created_at, updated_at FROM departments WHERE id = $1"
    )
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let department = match department_query {
        Ok(Some(dept)) => dept,
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

    // Verify user exists and belongs to same faculty
    let user_query = sqlx::query(
        r#"
        SELECT u.id, u.student_id, u.first_name, u.last_name, d.faculty_id as user_faculty_id
        FROM users u
        LEFT JOIN departments d ON u.department_id = d.id
        WHERE u.id = $1
        "#
    )
    .bind(request.user_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let user_row = match user_query {
        Ok(Some(user)) => user,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch user: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let user_faculty_id = user_row.get::<Option<Uuid>, _>("user_faculty_id");
    if user_faculty_id != Some(department.faculty_id) {
        let error_response = json!({
            "status": "error",
            "message": "User must belong to the same faculty as the department"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Only allow RegularAdmin assignment to departments
    if request.admin_level != AdminLevel::RegularAdmin {
        let error_response = json!({
            "status": "error",
            "message": "Only RegularAdmin level can be assigned to departments"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Check if user already has admin role
    let existing_role = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM admin_roles WHERE user_id = $1)"
    )
    .bind(request.user_id)
    .fetch_one(&session_state.db_pool)
    .await
    .unwrap_or(false);

    if existing_role {
        let error_response = json!({
            "status": "error",
            "message": "User already has an admin role"
        });
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    // Create admin role
    let create_admin_result = sqlx::query(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING id
        "#
    )
    .bind(request.user_id)
    .bind(&request.admin_level)
    .bind(department.faculty_id)
    .bind(&vec!["ManageDepartmentActivities", "ViewDepartmentUsers"])
    .fetch_one(&session_state.db_pool)
    .await;

    match create_admin_result {
        Ok(_) => {
            let response = json!({
                "status": "success",
                "data": {
                    "department_id": department_id,
                    "department_name": department.name,
                    "user_id": request.user_id,
                    "user_name": format!("{} {}", 
                        user_row.get::<String, _>("first_name"),
                        user_row.get::<String, _>("last_name")
                    ),
                    "admin_level": request.admin_level,
                    "faculty_id": department.faculty_id
                },
                "message": "Department admin assigned successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to assign department admin: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Transfer admin between departments (Faculty Admin or SuperAdmin only)
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferDepartmentAdminRequest {
    pub admin_role_id: Uuid,
    pub target_department_id: Uuid,
}

pub async fn transfer_department_admin(
    State(session_state): State<SessionState>,
    Path(source_department_id): Path<Uuid>,
    _admin: FacultyAdminUser,
    Json(request): Json<TransferDepartmentAdminRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify both departments exist and belong to same faculty
    let departments_query = sqlx::query(
        r#"
        SELECT d1.id as source_id, d1.name as source_name, d1.faculty_id as source_faculty,
               d2.id as target_id, d2.name as target_name, d2.faculty_id as target_faculty
        FROM departments d1
        CROSS JOIN departments d2
        WHERE d1.id = $1 AND d2.id = $2
        "#
    )
    .bind(source_department_id)
    .bind(request.target_department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let dept_row = match departments_query {
        Ok(Some(row)) => row,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Source or target department not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch departments: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let source_faculty = dept_row.get::<Uuid, _>("source_faculty");
    let target_faculty = dept_row.get::<Uuid, _>("target_faculty");

    if source_faculty != target_faculty {
        let error_response = json!({
            "status": "error",
            "message": "Cannot transfer admin between different faculties"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Verify admin role exists and get admin info
    let admin_info_query = sqlx::query(
        r#"
        SELECT ar.id, ar.user_id, ar.admin_level, ar.faculty_id,
               u.first_name, u.last_name, u.student_id,
               d.id as current_dept_id, d.name as current_dept_name
        FROM admin_roles ar
        JOIN users u ON ar.user_id = u.id
        LEFT JOIN departments d ON u.department_id = d.id
        WHERE ar.id = $1 AND ar.faculty_id = $2
        "#
    )
    .bind(request.admin_role_id)
    .bind(source_faculty)
    .fetch_optional(&session_state.db_pool)
    .await;

    let admin_info = match admin_info_query {
        Ok(Some(row)) => row,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Admin role not found or not in correct faculty"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch admin role: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Transfer user to target department
    let transfer_result = sqlx::query(
        "UPDATE users SET department_id = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(request.target_department_id)
    .bind(admin_info.get::<Uuid, _>("user_id"))
    .execute(&session_state.db_pool)
    .await;

    match transfer_result {
        Ok(_) => {
            let response = json!({
                "status": "success",
                "data": {
                    "admin_role_id": request.admin_role_id,
                    "user_id": admin_info.get::<Uuid, _>("user_id"),
                    "user_name": format!("{} {}", 
                        admin_info.get::<String, _>("first_name"),
                        admin_info.get::<String, _>("last_name")
                    ),
                    "admin_level": admin_info.get::<AdminLevel, _>("admin_level"),
                    "source_department": {
                        "id": source_department_id,
                        "name": dept_row.get::<String, _>("source_name")
                    },
                    "target_department": {
                        "id": request.target_department_id,
                        "name": dept_row.get::<String, _>("target_name")
                    },
                    "faculty_id": source_faculty
                },
                "message": "Department admin transferred successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to transfer department admin: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Remove admin from department (Faculty Admin or SuperAdmin only)
pub async fn remove_department_admin(
    State(session_state): State<SessionState>,
    Path(department_id): Path<Uuid>,
    Path(admin_role_id): Path<Uuid>,
    _admin: FacultyAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get admin role info with department verification
    let admin_query = sqlx::query(
        r#"
        SELECT ar.id, ar.user_id, ar.admin_level, ar.faculty_id,
               u.first_name, u.last_name, u.student_id, u.department_id,
               d.name as department_name
        FROM admin_roles ar
        JOIN users u ON ar.user_id = u.id
        LEFT JOIN departments d ON u.department_id = d.id
        WHERE ar.id = $1 AND u.department_id = $2
        "#
    )
    .bind(admin_role_id)
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let admin_info = match admin_query {
        Ok(Some(row)) => row,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Admin role not found in this department"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch admin role: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Remove admin role
    let remove_result = sqlx::query("DELETE FROM admin_roles WHERE id = $1")
        .bind(admin_role_id)
        .execute(&session_state.db_pool)
        .await;

    match remove_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response = json!({
                    "status": "error",
                    "message": "Admin role not found"
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            } else {
                let response = json!({
                    "status": "success",
                    "data": {
                        "admin_role_id": admin_role_id,
                        "department_id": department_id,
                        "department_name": admin_info.get::<Option<String>, _>("department_name"),
                        "user_id": admin_info.get::<Uuid, _>("user_id"),
                        "user_name": format!("{} {}", 
                            admin_info.get::<String, _>("first_name"),
                            admin_info.get::<String, _>("last_name")
                        ),
                        "faculty_id": admin_info.get::<Uuid, _>("faculty_id")
                    },
                    "message": "Department admin removed successfully"
                });
                Ok(Json(response))
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to remove department admin: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get department admins (Faculty Admin or SuperAdmin only)
pub async fn get_department_admins(
    State(session_state): State<SessionState>,
    Path(department_id): Path<Uuid>,
    _admin: FacultyAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify department exists
    let department_query = sqlx::query_as::<_, Department>(
        "SELECT id, name, code, faculty_id, description, created_at, updated_at FROM departments WHERE id = $1"
    )
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let department = match department_query {
        Ok(Some(dept)) => dept,
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

    // Get department admins
    let admins_query = sqlx::query(
        r#"
        SELECT ar.id as admin_role_id, ar.admin_level, ar.permissions, ar.created_at as role_created_at,
               u.id as user_id, u.student_id, u.first_name, u.last_name, u.email, u.created_at as user_created_at,
               COUNT(a.id) as activities_managed
        FROM admin_roles ar
        JOIN users u ON ar.user_id = u.id
        LEFT JOIN activities a ON (ar.faculty_id = a.faculty_id AND ar.admin_level = 'regular_admin')
        WHERE u.department_id = $1 AND ar.admin_level = 'regular_admin'
        GROUP BY ar.id, ar.admin_level, ar.permissions, ar.created_at, u.id, u.student_id, u.first_name, u.last_name, u.email, u.created_at
        ORDER BY u.last_name, u.first_name
        "#
    )
    .bind(department_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match admins_query {
        Ok(rows) => {
            let mut department_admins = Vec::new();

            for row in rows {
                let admin_info = json!({
                    "admin_role_id": row.get::<Uuid, _>("admin_role_id"),
                    "user_id": row.get::<Uuid, _>("user_id"),
                    "student_id": row.get::<String, _>("student_id"),
                    "first_name": row.get::<String, _>("first_name"),
                    "last_name": row.get::<String, _>("last_name"),
                    "email": row.get::<String, _>("email"),
                    "admin_level": row.get::<AdminLevel, _>("admin_level"),
                    "permissions": row.get::<Vec<String>, _>("permissions"),
                    "activities_managed": row.get::<i64, _>("activities_managed"),
                    "role_created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("role_created_at"),
                    "user_created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("user_created_at")
                });

                department_admins.push(admin_info);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "department": {
                        "id": department.id,
                        "name": department.name,
                        "code": department.code,
                        "faculty_id": department.faculty_id
                    },
                    "admins": department_admins,
                    "admin_count": department_admins.len()
                },
                "message": "Department admins retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch department admins: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}