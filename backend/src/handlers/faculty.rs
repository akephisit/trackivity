use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde_json::json;
use serde::Deserialize;

use crate::middleware::session::SessionState;
use crate::models::faculty::Faculty;

#[derive(Debug, Deserialize)]
pub struct CreateFacultyRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacultyRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

/// Get all faculties
pub async fn get_faculties(
    State(session_state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Query all faculties from database
    let query_result = sqlx::query_as::<_, Faculty>(
        "SELECT id, name, code, description, created_at, updated_at FROM faculties ORDER BY name"
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
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, Faculty>(
        "SELECT id, name, code, description, created_at, updated_at FROM faculties WHERE id = $1"
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
        "INSERT INTO faculties (name, code, description) VALUES ($1, $2, $3) RETURNING id, name, code, description, created_at, updated_at"
    )
    .bind(&request.name)
    .bind(&request.code)
    .bind(&request.description)
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
    Path(id): Path<i32>,
    Json(request): Json<UpdateFacultyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Build dynamic update query
    let mut query = "UPDATE faculties SET updated_at = NOW()".to_string();
    let mut params: Vec<String> = vec![];
    let mut param_count = 1;

    if let Some(name) = &request.name {
        query.push_str(&format!(", name = ${}", param_count));
        params.push(name.clone());
        param_count += 1;
    }
    
    if let Some(code) = &request.code {
        query.push_str(&format!(", code = ${}", param_count));
        params.push(code.clone());
        param_count += 1;
    }
    
    if let Some(description) = &request.description {
        query.push_str(&format!(", description = ${}", param_count));
        params.push(description.clone());
        param_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING id, name, code, description, created_at, updated_at", param_count));

    let mut query_builder = sqlx::query_as::<_, Faculty>(&query);
    
    for param in params {
        query_builder = query_builder.bind(param);
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
    Path(id): Path<i32>,
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