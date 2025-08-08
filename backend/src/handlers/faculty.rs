use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::middleware::session::SessionState;
use crate::models::{
    faculty::Faculty,
    department::Department,
    analytics::{FacultyStatsResponse, DepartmentStatsResponse},
    user::User,
};

#[derive(Debug, Deserialize)]
pub struct CreateFacultyRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacultyRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub status: Option<bool>,
}

/// Get all faculties
pub async fn get_faculties(
    State(session_state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Query all faculties from database
    let query_result = sqlx::query_as::<_, Faculty>(
        "SELECT id, name, code, description, status, created_at, updated_at FROM faculties ORDER BY name",
    )
    .fetch_all(&session_state.db_pool)
    .await;

    match query_result {
        Ok(faculties) => {
            let response = json!({
                "status": "success",
                "data": {
                    "faculties": faculties
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch faculties: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get faculty by ID
pub async fn get_faculty(
    State(session_state): State<SessionState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, Faculty>(
        "SELECT id, name, code, description, status, created_at, updated_at FROM faculties WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&session_state.db_pool)
    .await;

    match query_result {
        Ok(faculty) => {
            let response = json!({
                "status": "success",
                "data": faculty
            });
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Faculty not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch faculty: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create new faculty
pub async fn create_faculty(
    State(session_state): State<SessionState>,
    Json(request): Json<CreateFacultyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, Faculty>(
        "INSERT INTO faculties (name, code, description, status) VALUES ($1, $2, $3, $4) RETURNING id, name, code, description, status, created_at, updated_at"
    )
    .bind(&request.name)
    .bind(&request.code)
    .bind(&request.description)
    .bind(request.status.unwrap_or(true))
    .fetch_one(&session_state.db_pool)
    .await;

    match query_result {
        Ok(faculty) => {
            let response = json!({
                "status": "success",
                "message": "Faculty created successfully",
                "data": faculty
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create faculty: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Update faculty
pub async fn update_faculty(
    State(session_state): State<SessionState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateFacultyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Build dynamic update query
    let mut query = "UPDATE faculties SET updated_at = NOW()".to_string();
    let mut param_count = 1;
    let mut query_builder = sqlx::query_as::<_, Faculty>("");

    if let Some(name) = &request.name {
        query.push_str(&format!(", name = ${}", param_count));
        param_count += 1;
    }

    if let Some(code) = &request.code {
        query.push_str(&format!(", code = ${}", param_count));
        param_count += 1;
    }

    if let Some(description) = &request.description {
        query.push_str(&format!(", description = ${}", param_count));
        param_count += 1;
    }

    if let Some(_) = &request.status {
        query.push_str(&format!(", status = ${}", param_count));
        param_count += 1;
    }

    query.push_str(&format!(
        " WHERE id = ${} RETURNING id, name, code, description, status, created_at, updated_at",
        param_count
    ));

    query_builder = sqlx::query_as::<_, Faculty>(&query);

    if let Some(name) = &request.name {
        query_builder = query_builder.bind(name);
    }

    if let Some(code) = &request.code {
        query_builder = query_builder.bind(code);
    }

    if let Some(description) = &request.description {
        query_builder = query_builder.bind(description);
    }

    if let Some(status) = &request.status {
        query_builder = query_builder.bind(status);
    }

    query_builder = query_builder.bind(id);

    match query_builder.fetch_one(&session_state.db_pool).await {
        Ok(faculty) => {
            let response = json!({
                "status": "success",
                "message": "Faculty updated successfully",
                "data": faculty
            });
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Faculty not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update faculty: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Delete faculty
pub async fn delete_faculty(
    State(session_state): State<SessionState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query("DELETE FROM faculties WHERE id = $1")
        .bind(id)
        .execute(&session_state.db_pool)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response = json!({
                    "status": "error",
                    "message": "Faculty not found"
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            } else {
                let response = json!({
                    "status": "success",
                    "message": "Faculty deleted successfully"
                });
                Ok(Json(response))
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to delete faculty: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DepartmentQueryParams {
    pub include_stats: Option<bool>,
}

/// Get all departments for a faculty
pub async fn get_faculty_departments(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    Query(params): Query<DepartmentQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // First verify faculty exists
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
    let departments_query = sqlx::query_as::<_, Department>(
        "SELECT id, name, code, faculty_id, description, created_at, updated_at 
         FROM departments WHERE faculty_id = $1 ORDER BY name"
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match departments_query {
        Ok(departments) => {
            let mut response_data = json!({
                "departments": departments
            });

            // Include analytics if requested
            if params.include_stats.unwrap_or(false) {
                let stats_query = sqlx::query_as::<_, crate::models::analytics::DepartmentAnalytics>(
                    "SELECT * FROM department_analytics WHERE faculty_id = $1 ORDER BY calculated_at DESC"
                )
                .bind(faculty_id)
                .fetch_all(&session_state.db_pool)
                .await;

                if let Ok(stats) = stats_query {
                    response_data["analytics"] = serde_json::to_value(stats).unwrap_or(json!([]));
                }
            }

            let response = json!({
                "status": "success",
                "data": response_data
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

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

/// Create new department in faculty
pub async fn create_faculty_department(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    Json(request): Json<CreateDepartmentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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

    // Create department
    let query_result = sqlx::query_as::<_, Department>(
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

    match query_result {
        Ok(department) => {
            let response = json!({
                "status": "success",
                "message": "Department created successfully",
                "data": department
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

/// Get faculty students
pub async fn get_faculty_students(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get students in faculty through departments
    let students_query = sqlx::query_as::<_, User>(
        "SELECT u.id, u.student_id, u.email, u.first_name, u.last_name, 
                u.qr_secret, u.department_id, u.created_at, u.updated_at 
         FROM users u 
         JOIN departments d ON u.department_id = d.id 
         WHERE d.faculty_id = $1 
         ORDER BY u.last_name, u.first_name"
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match students_query {
        Ok(students) => {
            let response = json!({
                "status": "success",
                "data": {
                    "students": students,
                    "total_count": students.len()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch students: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Toggle faculty status
pub async fn toggle_faculty_status(
    State(session_state): State<SessionState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, Faculty>(
        "UPDATE faculties SET status = NOT status, updated_at = NOW() 
         WHERE id = $1 
         RETURNING id, name, code, description, status, created_at, updated_at"
    )
    .bind(id)
    .fetch_one(&session_state.db_pool)
    .await;

    match query_result {
        Ok(faculty) => {
            let response = json!({
                "status": "success",
                "message": "Faculty status toggled successfully",
                "data": faculty
            });
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Faculty not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to toggle faculty status: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get faculty analytics
pub async fn get_faculty_analytics(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get faculty info
    let faculty_query = sqlx::query_as::<_, Faculty>(
        "SELECT * FROM faculties WHERE id = $1"
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    let faculty = match faculty_query {
        Ok(f) => f,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Faculty not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch faculty: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Calculate analytics if not exists or outdated (older than 1 hour)
    let _ = sqlx::query("SELECT calculate_faculty_analytics($1)")
        .bind(faculty_id)
        .execute(&session_state.db_pool)
        .await;

    // Get faculty analytics
    let analytics_query = sqlx::query_as::<_, crate::models::analytics::FacultyAnalytics>(
        "SELECT * FROM faculty_analytics WHERE faculty_id = $1 ORDER BY calculated_at DESC LIMIT 1"
    )
    .bind(faculty_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    // Get department analytics
    let dept_analytics_query = sqlx::query_as::<_, crate::models::analytics::DepartmentAnalytics>(
        "SELECT da.* FROM department_analytics da 
         JOIN departments d ON da.department_id = d.id 
         WHERE d.faculty_id = $1 
         ORDER BY da.calculated_at DESC"
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match (analytics_query, dept_analytics_query) {
        (Ok(analytics), Ok(dept_analytics)) => {
            let response_data = if let Some(analytics) = analytics {
                // Build department stats
                let department_stats: Vec<DepartmentStatsResponse> = dept_analytics
                    .into_iter()
                    .map(|da| DepartmentStatsResponse {
                        department_id: da.department_id,
                        department_name: "Department".to_string(), // TODO: Join with departments table
                        department_code: "DEPT".to_string(),
                        total_students: da.total_students,
                        active_students: da.active_students,
                        total_activities: da.total_activities,
                        participation_rate: da.participation_rate.to_string().parse().unwrap_or(0.0),
                        last_calculated: da.calculated_at,
                    })
                    .collect();

                FacultyStatsResponse {
                    faculty_id: faculty.id,
                    faculty_name: faculty.name,
                    faculty_code: faculty.code,
                    total_students: analytics.total_students,
                    active_students: analytics.active_students,
                    total_activities: analytics.total_activities,
                    completed_activities: analytics.completed_activities,
                    participation_rate: analytics.average_participation_rate.to_string().parse().unwrap_or(0.0),
                    monthly_activity_count: analytics.monthly_activity_count,
                    department_count: analytics.department_count,
                    departments: department_stats,
                    last_calculated: analytics.calculated_at,
                }
            } else {
                // Return basic faculty info if no analytics
                FacultyStatsResponse {
                    faculty_id: faculty.id,
                    faculty_name: faculty.name,
                    faculty_code: faculty.code,
                    total_students: 0,
                    active_students: 0,
                    total_activities: 0,
                    completed_activities: 0,
                    participation_rate: 0.0,
                    monthly_activity_count: 0,
                    department_count: 0,
                    departments: vec![],
                    last_calculated: faculty.created_at,
                }
            };

            let response = json!({
                "status": "success",
                "data": response_data
            });
            Ok(Json(response))
        }
        (Err(e), _) | (_, Err(e)) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch analytics: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
