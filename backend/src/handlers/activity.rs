use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::middleware::session::{AdminUser, SessionState};
use crate::models::session::SessionUser;
use crate::models::{
    activity::{ActivityStatus},
    participation::{Participation, ParticipationStatus},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateActivityRequest {
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateActivityRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub max_participants: Option<i32>,
    pub status: Option<ActivityStatus>,
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityWithDetails {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub current_participants: i64,
    pub status: ActivityStatus,
    pub activity_type: Option<String>,
    pub faculty_id: Option<Uuid>,
    pub faculty_name: Option<String>,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_registered: bool,
    pub user_participation_status: Option<ParticipationStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipationWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub student_id: String,
    pub email: String,
    pub department_name: Option<String>,
    pub status: ParticipationStatus,
    pub registered_at: DateTime<Utc>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub checked_out_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrScanRequest {
    pub qr_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrScanResponse {
    pub success: bool,
    pub message: String,
    pub participation_status: Option<ParticipationStatus>,
    pub user_name: Option<String>,
    pub student_id: Option<String>,
}

/// Get activities with filtering and pagination
pub async fn get_activities(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50);

    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0);

    let search = params.get("search").cloned();
    let status_filter = params.get("status");
    let faculty_id = params
        .get("faculty_id")
        .and_then(|f| Uuid::parse_str(f).ok());
    // department filter removed

    let mut query = r#"
        SELECT 
            a.id,
            a.title,
            a.description,
            a.location,
            ((a.start_date::timestamp + a.start_time_only) AT TIME ZONE 'UTC') as start_time,
            ((a.end_date::timestamp + a.end_time_only) AT TIME ZONE 'UTC') as end_time,
            a.max_participants,
            a.status,
            a.activity_type,
            a.faculty_id,
            a.created_by,
            a.created_at,
            a.updated_at,
            f.name as faculty_name,
            u.first_name || ' ' || u.last_name as created_by_name,
            COALESCE(COUNT(p.id), 0) as current_participants,
            CASE WHEN up.id IS NOT NULL THEN true ELSE false END as is_registered,
            up.status as user_participation_status
        FROM activities a
        LEFT JOIN faculties f ON a.faculty_id = f.id
        LEFT JOIN users u ON a.created_by = u.id
        LEFT JOIN participations p ON a.id = p.activity_id
        LEFT JOIN participations up ON a.id = up.activity_id AND up.user_id = $3
    "#
    .to_string();

    let mut count_query = r#"
        SELECT COUNT(DISTINCT a.id) 
        FROM activities a
        LEFT JOIN faculties f ON a.faculty_id = f.id
    "#
    .to_string();

    let mut conditions = Vec::new();
    let mut param_count = 4;

    if let Some(_search_term) = &search {
        conditions.push(format!(
            "(a.title ILIKE ${} OR a.description ILIKE ${} OR a.location ILIKE ${})",
            param_count, param_count, param_count
        ));
        param_count += 1;
    }

    if let Some(_status) = status_filter {
        conditions.push(format!("a.status = ${}", param_count));
        param_count += 1;
    }

    if faculty_id.is_some() {
        conditions.push(format!("a.faculty_id = ${}", param_count));
        param_count += 1;
    }

    // department filter removed

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        query.push_str(&where_clause);
        count_query.push_str(&where_clause);
    }

    query.push_str(" GROUP BY a.id, a.title, a.description, a.location, a.start_date, a.end_date, a.start_time_only, a.end_time_only, a.max_participants, a.status, a.faculty_id, a.created_by, a.created_at, a.updated_at, f.name, u.first_name, u.last_name, up.id, up.status");
    query.push_str(" ORDER BY a.start_date DESC, a.start_time_only DESC LIMIT $1 OFFSET $2");

    let mut query_builder = sqlx::query(&query)
        .bind(limit)
        .bind(offset)
        .bind(user.user_id);

    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        query_builder = query_builder.bind(search_pattern.clone());
        count_query_builder = count_query_builder.bind(search_pattern);
    }

    if let Some(status) = status_filter {
        query_builder = query_builder.bind(status);
        count_query_builder = count_query_builder.bind(status);
    }

    if let Some(f_id) = faculty_id {
        query_builder = query_builder.bind(f_id);
        count_query_builder = count_query_builder.bind(f_id);
    }

    // no department filter

    let activities_result = query_builder.fetch_all(&session_state.db_pool).await;
    let total_count_result = count_query_builder.fetch_one(&session_state.db_pool).await;

    match (activities_result, total_count_result) {
        (Ok(rows), Ok(total_count)) => {
            let mut activities_with_details = Vec::new();

            for row in rows {
                let activity_detail = ActivityWithDetails {
                    id: row.get("id"),
                    title: row.get("title"),
                    description: row.get("description"),
                    location: row.get("location"),
                    start_time: row.get("start_time"),
                    end_time: row.get("end_time"),
                    max_participants: row.get::<Option<i32>, _>("max_participants"),
                    current_participants: row
                        .get::<Option<i64>, _>("current_participants")
                        .unwrap_or(0),
                    status: row.get::<ActivityStatus, _>("status"),
                    activity_type: row.get::<Option<String>, _>("activity_type"),
                    faculty_id: row.get::<Option<Uuid>, _>("faculty_id"),
                    faculty_name: row.get::<Option<String>, _>("faculty_name"),
                    created_by: row.get("created_by"),
                    created_by_name: row.get("created_by_name"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    is_registered: row.get::<Option<bool>, _>("is_registered").unwrap_or(false),
                    user_participation_status: row
                        .get::<Option<ParticipationStatus>, _>("user_participation_status"),
                };

                activities_with_details.push(activity_detail);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "activities": activities_with_details,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset
                },
                "message": "Activities retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve activities"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get activity by ID with detailed information
pub async fn get_activity(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query_result = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.title,
            a.description,
            a.location,
            ((a.start_date::timestamp + a.start_time_only) AT TIME ZONE 'UTC') as start_time,
            ((a.end_date::timestamp + a.end_time_only) AT TIME ZONE 'UTC') as end_time,
            a.max_participants,
            a.status,
            a.activity_type,
            a.faculty_id,
            a.created_by,
            a.created_at,
            a.updated_at,
            f.name as faculty_name,
            u.first_name || ' ' || u.last_name as created_by_name,
            COALESCE(COUNT(p.id), 0) as current_participants,
            CASE WHEN up.id IS NOT NULL THEN true ELSE false END as is_registered,
            up.status as user_participation_status
        FROM activities a
        LEFT JOIN faculties f ON a.faculty_id = f.id
        LEFT JOIN users u ON a.created_by = u.id
        LEFT JOIN participations p ON a.id = p.activity_id
        LEFT JOIN participations up ON a.id = up.activity_id AND up.user_id = $2
        WHERE a.id = $1
        GROUP BY a.id, a.title, a.description, a.location, a.start_date, a.end_date, a.start_time_only, a.end_time_only, a.max_participants, a.status, a.activity_type, a.faculty_id, a.created_by, a.created_at, a.updated_at, f.name, u.first_name, u.last_name, up.id, up.status
        "#
    )
    .bind(&activity_id)
    .bind(&user.user_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match query_result {
        Ok(row) => {
            let activity_detail = ActivityWithDetails {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                location: row.get("location"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                max_participants: row.get("max_participants"),
                current_participants: row.get::<i64, _>("current_participants"),
                status: row.get("status"),
                activity_type: row.get::<Option<String>, _>("activity_type"),
                faculty_id: row.get("faculty_id"),
                faculty_name: row
                    .get::<Option<String>, _>("faculty_name")
                    .filter(|s| !s.is_empty()),
                created_by: row.get("created_by"),
                created_by_name: row
                    .get::<Option<String>, _>("created_by_name")
                    .unwrap_or_else(|| "Unknown".to_string()),
                created_at: row
                    .get::<Option<DateTime<Utc>>, _>("created_at")
                    .unwrap_or_else(|| Utc::now()),
                updated_at: row
                    .get::<Option<DateTime<Utc>>, _>("updated_at")
                    .unwrap_or_else(|| Utc::now()),
                is_registered: row.get::<Option<bool>, _>("is_registered").unwrap_or(false),
                user_participation_status: row.get("user_participation_status"),
            };

            let response = json!({
                "status": "success",
                "data": activity_detail,
                "message": "Activity retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve activity"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create new activity
pub async fn create_activity(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Json(request): Json<CreateActivityRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user has permission to create activities
    if !user
        .permissions
        .iter()
        .any(|p| p.contains("ManageActivities") || p.contains("CreateActivity"))
    {
        let error_response = json!({
            "status": "error",
            "message": "Access denied: You don't have permission to create activities"
        });
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    // Validate time range
    if request.start_time >= request.end_time {
        let error_response = json!({
            "status": "error",
            "message": "Start time must be before end time"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let start_naive = request.start_time.naive_utc();
    let end_naive = request.end_time.naive_utc();
    let create_result = sqlx::query(
        r#"
        INSERT INTO activities (
            title, description, location, max_participants, faculty_id, created_by,
            start_date, end_date, start_time_only, end_time_only
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::date, $8::date, $9::time, $10::time)
        RETURNING id, title, description, location,
          ((start_date::timestamp + start_time_only) AT TIME ZONE 'UTC') as start_time,
          ((end_date::timestamp + end_time_only) AT TIME ZONE 'UTC') as end_time,
          max_participants, status, faculty_id, created_by, created_at, updated_at
        "#
    )
    .bind(&request.title)
    .bind(&request.description)
    .bind(&request.location)
    .bind(request.max_participants)
    .bind(request.faculty_id)
    .bind(user.user_id)
    .bind(start_naive.date())
    .bind(end_naive.date())
    .bind(start_naive.time())
    .bind(end_naive.time())
    .fetch_one(&session_state.db_pool)
    .await;

    match create_result {
        Ok(row) => {
            let response = json!({
                "status": "success",
                "data": {
                    "id": row.get::<Uuid, _>("id"),
                    "title": row.get::<String, _>("title"),
                    "description": row.get::<String, _>("description"),
                    "location": row.get::<String, _>("location"),
                    "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                    "end_time": row.get::<DateTime<Utc>, _>("end_time"),
                    "max_participants": row.get::<Option<i32>, _>("max_participants"),
                    "status": row.get::<ActivityStatus, _>("status"),
                    "faculty_id": row.get::<Option<Uuid>, _>("faculty_id"),
                    "created_by": row.get::<Uuid, _>("created_by"),
                    "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                    "updated_at": row.get::<DateTime<Utc>, _>("updated_at")
                },
                "message": "Activity created successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create activity: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Update activity
pub async fn update_activity(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Path(activity_id): Path<Uuid>,
    Json(request): Json<UpdateActivityRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user has permission to update activities or is the creator
    let activity_check = sqlx::query("SELECT created_by FROM activities WHERE id = $1")
        .bind(&activity_id)
        .fetch_one(&session_state.db_pool)
        .await;

    let can_update = match activity_check {
        Ok(activity) => {
            activity.get::<Uuid, _>("created_by") == user.user_id
                || user
                    .permissions
                    .iter()
                    .any(|p| p.contains("ManageActivities"))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check activity"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    if !can_update {
        let error_response = json!({
            "status": "error",
            "message": "Access denied: You can only update your own activities or need ManageActivities permission"
        });
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    // Validate time range if both times are provided
    if let (Some(start_time), Some(end_time)) = (&request.start_time, &request.end_time) {
        if start_time >= end_time {
            let error_response = json!({
                "status": "error",
                "message": "Start time must be before end time"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Build dynamic update query
    let mut query = "UPDATE activities SET updated_at = NOW()".to_string();
    let mut param_count = 1;

    if let Some(_title) = &request.title {
        query.push_str(&format!(", title = ${}", param_count));
        param_count += 1;
    }

    if let Some(_description) = &request.description {
        query.push_str(&format!(", description = ${}", param_count));
        param_count += 1;
    }

    if let Some(_location) = &request.location {
        query.push_str(&format!(", location = ${}", param_count));
        param_count += 1;
    }

    if request.start_time.is_some() {
        query.push_str(&format!(", start_date = ${}, start_time_only = ${}", param_count, param_count + 1));
        param_count += 2;
    }

    if request.end_time.is_some() {
        query.push_str(&format!(", end_date = ${}, end_time_only = ${}", param_count, param_count + 1));
        param_count += 2;
    }

    if request.max_participants.is_some() {
        query.push_str(&format!(", max_participants = ${}", param_count));
        param_count += 1;
    }

    if let Some(_status) = &request.status {
        query.push_str(&format!(", status = ${}", param_count));
        param_count += 1;
    }

    if request.faculty_id.is_some() {
        query.push_str(&format!(", faculty_id = ${}", param_count));
        param_count += 1;
    }

    // department removed

    query.push_str(&format!(" WHERE id = ${} RETURNING id, title, description, location,
        ((start_date::timestamp + start_time_only) AT TIME ZONE 'UTC') as start_time,
        ((end_date::timestamp + end_time_only) AT TIME ZONE 'UTC') as end_time,
        max_participants, status, faculty_id, created_by, created_at, updated_at", param_count));

    // Execute query with proper parameter binding
    let mut query_builder = sqlx::query(&query);

    if let Some(title) = &request.title {
        query_builder = query_builder.bind(title);
    }
    if let Some(description) = &request.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(location) = &request.location {
        query_builder = query_builder.bind(location);
    }
    if let Some(start_time) = request.start_time {
        let st = start_time.naive_utc();
        query_builder = query_builder.bind(st.date()).bind(st.time());
    }
    if let Some(end_time) = request.end_time {
        let et = end_time.naive_utc();
        query_builder = query_builder.bind(et.date()).bind(et.time());
    }
    if let Some(max_participants) = request.max_participants {
        query_builder = query_builder.bind(max_participants);
    }
    if let Some(status) = &request.status {
        query_builder = query_builder.bind(status);
    }
    if let Some(faculty_id) = request.faculty_id {
        query_builder = query_builder.bind(faculty_id);
    }
    query_builder = query_builder.bind(activity_id);

    match query_builder.fetch_one(&session_state.db_pool).await {
        Ok(row) => {
            let response = json!({
                "status": "success",
                "data": {
                    "id": row.get::<Uuid, _>("id"),
                    "title": row.get::<String, _>("title"),
                    "description": row.get::<String, _>("description"),
                    "location": row.get::<String, _>("location"),
                    "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                    "end_time": row.get::<DateTime<Utc>, _>("end_time"),
                    "max_participants": row.get::<Option<i32>, _>("max_participants"),
                    "status": row.get::<ActivityStatus, _>("status"),
                    "faculty_id": row.get::<Option<Uuid>, _>("faculty_id"),
                    "created_by": row.get::<Uuid, _>("created_by"),
                    "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                    "updated_at": row.get::<DateTime<Utc>, _>("updated_at")
                },
                "message": "Activity updated successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update activity: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Delete activity
pub async fn delete_activity(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user has permission to delete activities or is the creator
    let activity_check = sqlx::query("SELECT created_by FROM activities WHERE id = $1")
        .bind(&activity_id)
        .fetch_one(&session_state.db_pool)
        .await;

    let can_delete = match activity_check {
        Ok(activity) => {
            activity.get::<Uuid, _>("created_by") == user.user_id
                || user
                    .permissions
                    .iter()
                    .any(|p| p.contains("ManageActivities"))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check activity"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    if !can_delete {
        let error_response = json!({
            "status": "error",
            "message": "Access denied: You can only delete your own activities or need ManageActivities permission"
        });
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    let delete_result = sqlx::query("DELETE FROM activities WHERE id = $1")
        .bind(activity_id)
        .execute(&session_state.db_pool)
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response = json!({
                    "status": "error",
                    "message": "Activity not found"
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            } else {
                let response = json!({
                    "status": "success",
                    "message": "Activity deleted successfully"
                });
                Ok(Json(response))
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to delete activity: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get activity participations
pub async fn get_activity_participations(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Path(activity_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user can view participations (activity creator or admin)
    let activity_check = sqlx::query("SELECT created_by FROM activities WHERE id = $1")
        .bind(&activity_id)
        .fetch_one(&session_state.db_pool)
        .await;

    let can_view = match activity_check {
        Ok(activity) => {
            activity.get::<Uuid, _>("created_by") == user.user_id
                || user
                    .permissions
                    .iter()
                    .any(|p| p.contains("ManageActivities") || p.contains("ViewParticipations"))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check activity"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    if !can_view {
        let error_response = json!({
            "status": "error",
            "message": "Access denied: You don't have permission to view participations"
        });
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    let status_filter = params.get("status");

    let mut query = r#"
        SELECT 
            p.id,
            p.user_id,
            p.status,
            p.registered_at,
            p.checked_in_at,
            p.checked_out_at,
            p.notes,
            u.first_name || ' ' || u.last_name as user_name,
            u.student_id,
            u.email,
            d.name as department_name
        FROM participations p
        JOIN users u ON p.user_id = u.id
        LEFT JOIN departments d ON u.department_id = d.id
        WHERE p.activity_id = $1
    "#
    .to_string();

    if let Some(_status) = status_filter {
        query.push_str(" AND p.status = $2");
    }

    query.push_str(" ORDER BY p.registered_at DESC");

    let participations_result = if let Some(status) = status_filter {
        sqlx::query(&query)
            .bind(activity_id)
            .bind(status)
            .fetch_all(&session_state.db_pool)
            .await
    } else {
        sqlx::query(&query)
            .bind(activity_id)
            .fetch_all(&session_state.db_pool)
            .await
    };

    match participations_result {
        Ok(rows) => {
            let mut participations_with_users = Vec::new();

            for row in rows {
                let participation = ParticipationWithUser {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    user_name: row.get("user_name"),
                    student_id: row.get("student_id"),
                    email: row.get("email"),
                    department_name: row.get::<Option<String>, _>("department_name"),
                    status: row.get::<ParticipationStatus, _>("status"),
                    registered_at: row.get("registered_at"),
                    checked_in_at: row.get::<Option<DateTime<Utc>>, _>("checked_in_at"),
                    checked_out_at: row.get::<Option<DateTime<Utc>>, _>("checked_out_at"),
                    notes: row.get::<Option<String>, _>("notes"),
                };

                participations_with_users.push(participation);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "participations": participations_with_users,
                    "total_count": participations_with_users.len()
                },
                "message": "Participations retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve participations"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Participate in activity (register)
pub async fn participate(
    State(session_state): State<SessionState>,
    user: SessionUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if activity exists and get details
    let activity = sqlx::query(
        "SELECT id, title, status, start_time, max_participants FROM activities WHERE id = $1",
    )
    .bind(&activity_id)
    .fetch_one(&session_state.db_pool)
    .await;

    let activity = match activity {
        Ok(activity) => activity,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check activity"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Check if activity is open for registration
    let status: String = activity.get("status");
    if status != "published" && status != "ongoing" {
        let error_response = json!({
            "status": "error",
            "message": "Activity is not open for registration"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Check if user is already registered
    let existing_participation =
        sqlx::query("SELECT id FROM participations WHERE user_id = $1 AND activity_id = $2")
            .bind(&user.user_id)
            .bind(&activity_id)
            .fetch_optional(&session_state.db_pool)
            .await;

    match existing_participation {
        Ok(Some(_)) => {
            let error_response = json!({
                "status": "error",
                "message": "You are already registered for this activity"
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check existing participation"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Check if activity has reached max participants
    let max_participants: Option<i32> = activity.get("max_participants");
    if let Some(max_participants) = max_participants {
        let current_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM participations WHERE activity_id = $1",
        )
        .bind(activity_id)
        .fetch_one(&session_state.db_pool)
        .await
        .unwrap_or(0);

        if current_count >= max_participants as i64 {
            let error_response = json!({
                "status": "error",
                "message": "Activity has reached maximum number of participants"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Create participation
    let create_result = sqlx::query_as::<_, Participation>(
        r#"
        INSERT INTO participations (user_id, activity_id, status)
        VALUES ($1, $2, 'registered')
        RETURNING id, user_id, activity_id, status, registered_at, checked_in_at, checked_out_at, notes
        "#
    )
    .bind(user.user_id)
    .bind(activity_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match create_result {
        Ok(participation) => {
            let response = json!({
                "status": "success",
                "data": participation,
                "message": "Successfully registered for activity"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to register for activity: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Scan QR code for check-in/check-out
pub async fn scan_qr(
    State(session_state): State<SessionState>,
    _admin: AdminUser, // Only admins or activity creators can scan QR codes
    Path(activity_id): Path<Uuid>,
    Json(request): Json<QrScanRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Parse QR data
    let qr_data: Value = match serde_json::from_str(&request.qr_data) {
        Ok(data) => data,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid QR code format"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let user_id = match qr_data
        .get("user_id")
        .and_then(|id| id.as_str())
        .and_then(|id| Uuid::parse_str(id).ok())
    {
        Some(id) => id,
        None => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid user ID in QR code"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let qr_secret = match qr_data.get("secret").and_then(|s| s.as_str()) {
        Some(secret) => secret,
        None => {
            let error_response = json!({
                "status": "error",
                "message": "Missing secret in QR code"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Verify QR secret
    let user_check =
        sqlx::query("SELECT student_id, first_name, last_name, qr_secret FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&session_state.db_pool)
            .await;

    let user_data = match user_check {
        Ok(user) => {
            if user.get::<String, _>("qr_secret") != qr_secret {
                let error_response = json!({
                    "status": "error",
                    "message": "Invalid QR code secret"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }
            user
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to verify user"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Check if user is registered for this activity
    let participation = sqlx::query(
        "SELECT id, status FROM participations WHERE user_id = $1 AND activity_id = $2",
    )
    .bind(&user_id)
    .bind(&activity_id)
    .fetch_one(&session_state.db_pool)
    .await;

    let participation = match participation {
        Ok(p) => p,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "User is not registered for this activity"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check participation"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Determine next status based on current status
    let status: String = participation.get("status");
    let (new_status, field_to_update) = match status.as_str() {
        "registered" => ("checked_in", "checked_in_at"),
        "checked_in" => ("checked_out", "checked_out_at"),
        "checked_out" => ("completed", ""), // No additional field to update
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid participation status for QR scan"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Update participation status
    let update_query = if field_to_update.is_empty() {
        "UPDATE participations SET status = $1 WHERE id = $2".to_string()
    } else {
        format!(
            "UPDATE participations SET status = $1, {} = NOW() WHERE id = $2",
            field_to_update
        )
    };

    let update_result = sqlx::query(&update_query)
        .bind(new_status)
        .bind(participation.get::<Uuid, _>("id"))
        .execute(&session_state.db_pool)
        .await;

    match update_result {
        Ok(_) => {
            let response_data = QrScanResponse {
                success: true,
                message: format!("Successfully {} for activity", new_status.replace("_", " ")),
                participation_status: Some(match new_status {
                    "checked_in" => ParticipationStatus::CheckedIn,
                    "checked_out" => ParticipationStatus::CheckedOut,
                    "completed" => ParticipationStatus::Completed,
                    _ => ParticipationStatus::Registered,
                }),
                user_name: Some(format!(
                    "{} {}",
                    user_data.get::<String, _>("first_name"),
                    user_data.get::<String, _>("last_name")
                )),
                student_id: Some(user_data.get::<String, _>("student_id")),
            };

            let response = json!({
                "status": "success",
                "data": response_data,
                "message": "QR code scanned successfully"
            });

            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update participation: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
