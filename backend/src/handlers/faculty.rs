use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

use crate::middleware::session::SessionState;
use crate::models::faculty::Faculty;

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