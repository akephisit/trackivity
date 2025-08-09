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

use crate::middleware::session::{AdminUser, SessionState, SuperAdminUser, FacultyAdminUser};
use crate::models::{
    activity::ActivityStatus,
    admin_role::{AdminLevel, AdminRole},
    session::AdminSessionInfo,
    user::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_activities: i64,
    pub ongoing_activities: i64,
    pub total_participations: i64,
    pub active_sessions: i64,
    pub recent_activities: Vec<ActivitySummary>,
    pub user_registrations_today: i64,
    pub popular_activities: Vec<ActivitySummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub id: Uuid,
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub participant_count: i64,
    pub status: ActivityStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserInfo {
    pub id: Uuid,
    pub student_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
    pub admin_role: Option<AdminRole>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminActivityInfo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub current_participants: i64,
    pub status: ActivityStatus,
    pub created_by_name: String,
    pub faculty_name: Option<String>,
    pub department_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Get admin dashboard statistics
pub async fn get_dashboard(
    State(session_state): State<SessionState>,
    admin: AdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check admin level and get appropriate statistics
    match admin.admin_role.admin_level {
        AdminLevel::SuperAdmin => {
            get_super_admin_dashboard_stats(session_state).await
        }
        AdminLevel::FacultyAdmin | AdminLevel::RegularAdmin => {
            get_faculty_admin_dashboard_stats(session_state, admin.admin_role.faculty_id).await
        }
    }
}

/// Get dashboard statistics for SuperAdmin (system-wide)
async fn get_super_admin_dashboard_stats(
    session_state: SessionState,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get total users count
    let total_users_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&session_state.db_pool)
        .await;

    // Get total activities count
    let total_activities_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activities")
        .fetch_one(&session_state.db_pool)
        .await;

    // Get ongoing activities count
    let ongoing_activities_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activities WHERE status = 'ongoing'")
            .fetch_one(&session_state.db_pool)
            .await;

    // Get total participations count
    let total_participations_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM participations")
            .fetch_one(&session_state.db_pool)
            .await;

    // Get active sessions count from Redis
    let active_sessions = match session_state.redis_store.get_session_count().await {
        Ok(count) => count as i64,
        Err(_) => 0,
    };

    // Get user registrations today
    let user_registrations_today_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE created_at >= CURRENT_DATE")
            .fetch_one(&session_state.db_pool)
            .await;

    // Get recent activities (last 5)
    let recent_activities_result = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.title,
            a.start_time,
            a.status,
            COALESCE(COUNT(p.id), 0) as participant_count
        FROM activities a
        LEFT JOIN participations p ON a.id = p.activity_id
        WHERE a.created_at >= NOW() - INTERVAL '7 days'
        GROUP BY a.id, a.title, a.start_time, a.status
        ORDER BY a.created_at DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&session_state.db_pool)
    .await;

    // Get popular activities (most participants)
    let popular_activities_result = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.title,
            a.start_time,
            a.status,
            COALESCE(COUNT(p.id), 0) as participant_count
        FROM activities a
        LEFT JOIN participations p ON a.id = p.activity_id
        GROUP BY a.id, a.title, a.start_time, a.status
        ORDER BY participant_count DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&session_state.db_pool)
    .await;

    // Process results
    match (
        total_users_result,
        total_activities_result,
        ongoing_activities_result,
        total_participations_result,
        user_registrations_today_result,
        recent_activities_result,
        popular_activities_result,
    ) {
        (
            Ok(total_users),
            Ok(total_activities),
            Ok(ongoing_activities),
            Ok(total_participations),
            Ok(user_registrations_today),
            Ok(recent_activities_data),
            Ok(popular_activities_data),
        ) => {
            let recent_activities = recent_activities_data
                .into_iter()
                .map(|row| ActivitySummary {
                    id: row.get("id"),
                    title: row.get("title"),
                    start_time: row.get("start_time"),
                    participant_count: row.get::<i64, _>("participant_count"),
                    status: row.get("status"),
                })
                .collect();

            let popular_activities = popular_activities_data
                .into_iter()
                .map(|row| ActivitySummary {
                    id: row.get("id"),
                    title: row.get("title"),
                    start_time: row.get("start_time"),
                    participant_count: row.get::<i64, _>("participant_count"),
                    status: row.get("status"),
                })
                .collect();

            let dashboard_stats = DashboardStats {
                total_users,
                total_activities,
                ongoing_activities,
                total_participations,
                active_sessions,
                recent_activities,
                user_registrations_today,
                popular_activities,
            };

            let response = json!({
                "status": "success",
                "data": dashboard_stats,
                "message": "Dashboard statistics retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve dashboard statistics"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get dashboard statistics for Faculty Admin (faculty-specific)
async fn get_faculty_admin_dashboard_stats(
    session_state: SessionState,
    faculty_id: Option<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Ensure faculty_id is provided for FacultyAdmin
    let faculty_id = match faculty_id {
        Some(id) => id,
        None => {
            let error_response = json!({
                "status": "error",
                "message": "Faculty ID is required for faculty admin"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Get faculty-specific user count (users in departments belonging to this faculty)
    let total_users_result = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT u.id)
        FROM users u
        JOIN departments d ON u.department_id = d.id
        WHERE d.faculty_id = $1
        "#
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get faculty-specific activities count
    let total_activities_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM activities WHERE faculty_id = $1"
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get faculty-specific ongoing activities count
    let ongoing_activities_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM activities WHERE faculty_id = $1 AND status = 'ongoing'"
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get faculty-specific participations count
    let total_participations_result = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT p.id)
        FROM participations p
        JOIN activities a ON p.activity_id = a.id
        WHERE a.faculty_id = $1
        "#
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get active sessions count from users in this faculty
    // Note: This is tricky with Redis, so we'll approximate or use all sessions for now
    let active_sessions = match session_state.redis_store.get_session_count().await {
        Ok(count) => count as i64,
        Err(_) => 0,
    };

    // Get faculty-specific user registrations today
    let user_registrations_today_result = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT u.id)
        FROM users u
        JOIN departments d ON u.department_id = d.id
        WHERE d.faculty_id = $1 AND u.created_at >= CURRENT_DATE
        "#
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get recent activities for this faculty (last 5)
    let recent_activities_result = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.title,
            a.start_time,
            a.status,
            COALESCE(COUNT(p.id), 0) as participant_count
        FROM activities a
        LEFT JOIN participations p ON a.id = p.activity_id
        WHERE a.faculty_id = $1 AND a.created_at >= NOW() - INTERVAL '7 days'
        GROUP BY a.id, a.title, a.start_time, a.status
        ORDER BY a.created_at DESC
        LIMIT 5
        "#,
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    // Get popular activities for this faculty (most participants)
    let popular_activities_result = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.title,
            a.start_time,
            a.status,
            COALESCE(COUNT(p.id), 0) as participant_count
        FROM activities a
        LEFT JOIN participations p ON a.id = p.activity_id
        WHERE a.faculty_id = $1
        GROUP BY a.id, a.title, a.start_time, a.status
        ORDER BY participant_count DESC
        LIMIT 5
        "#,
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    // Process results
    match (
        total_users_result,
        total_activities_result,
        ongoing_activities_result,
        total_participations_result,
        user_registrations_today_result,
        recent_activities_result,
        popular_activities_result,
    ) {
        (
            Ok(total_users),
            Ok(total_activities),
            Ok(ongoing_activities),
            Ok(total_participations),
            Ok(user_registrations_today),
            Ok(recent_activities_data),
            Ok(popular_activities_data),
        ) => {
            let recent_activities = recent_activities_data
                .into_iter()
                .map(|row| ActivitySummary {
                    id: row.get("id"),
                    title: row.get("title"),
                    start_time: row.get("start_time"),
                    participant_count: row.get::<i64, _>("participant_count"),
                    status: row.get("status"),
                })
                .collect();

            let popular_activities = popular_activities_data
                .into_iter()
                .map(|row| ActivitySummary {
                    id: row.get("id"),
                    title: row.get("title"),
                    start_time: row.get("start_time"),
                    participant_count: row.get::<i64, _>("participant_count"),
                    status: row.get("status"),
                })
                .collect();

            let dashboard_stats = DashboardStats {
                total_users,
                total_activities,
                ongoing_activities,
                total_participations,
                active_sessions,
                recent_activities,
                user_registrations_today,
                popular_activities,
            };

            let response = json!({
                "status": "success",
                "data": dashboard_stats,
                "message": "Faculty dashboard statistics retrieved successfully"
            });

            return Ok(Json(response));
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve faculty dashboard statistics"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    }
}

/// Get admin users list with detailed information
pub async fn get_admin_users(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
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

    let mut query = r#"
        SELECT DISTINCT
            u.id,
            u.student_id,
            u.email,
            u.first_name,
            u.last_name,
            u.department_id,
            u.created_at,
            ar.id as admin_role_id,
            ar.admin_level,
            ar.faculty_id,
            ar.permissions,
            ar.created_at as role_created_at,
            ar.updated_at as role_updated_at,
            (SELECT MAX(s.last_accessed) FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) as last_login,
            CASE WHEN EXISTS(SELECT 1 FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) THEN true ELSE false END as is_active
        FROM users u
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        WHERE ar.id IS NOT NULL
    "#
    .to_string();

    let mut count_query = "SELECT COUNT(*) FROM users u LEFT JOIN admin_roles ar ON u.id = ar.user_id WHERE ar.id IS NOT NULL".to_string();

    if let Some(_search_term) = &search {
        let where_clause = " AND (u.first_name ILIKE $3 OR u.last_name ILIKE $3 OR u.email ILIKE $3 OR u.student_id ILIKE $3)";
        query.push_str(where_clause);
        count_query.push_str(where_clause);
    }

    query.push_str(" ORDER BY u.created_at DESC LIMIT $1 OFFSET $2");

    let users_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .bind(search_pattern.clone())
            .fetch_all(&session_state.db_pool)
            .await
    } else {
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&session_state.db_pool)
            .await
    };

    let total_count_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query_scalar::<_, i64>(&count_query)
            .bind(search_pattern)
            .fetch_one(&session_state.db_pool)
            .await
    } else {
        sqlx::query_scalar::<_, i64>(&count_query)
            .fetch_one(&session_state.db_pool)
            .await
    };

    match (users_result, total_count_result) {
        (Ok(rows), Ok(total_count)) => {
            let mut admin_users = Vec::new();

            for row in rows {
                let admin_role = match row.get::<Option<Uuid>, _>("admin_role_id") {
                    Some(role_id) => Some(AdminRole {
                        id: role_id,
                        user_id: row.get("id"),
                        admin_level: row
                            .get::<Option<AdminLevel>, _>("admin_level")
                            .unwrap_or(AdminLevel::RegularAdmin),
                        faculty_id: row.get::<Option<Uuid>, _>("faculty_id"),
                        permissions: row
                            .get::<Option<Vec<String>>, _>("permissions")
                            .unwrap_or_else(|| vec![]),
                        created_at: row.get::<Option<DateTime<Utc>>, _>("role_created_at"),
                        updated_at: row.get::<Option<DateTime<Utc>>, _>("role_updated_at"),
                    }),
                    None => None,
                };

                let admin_user = AdminUserInfo {
                    id: row.get("id"),
                    student_id: row.get("student_id"),
                    email: row.get("email"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    department_id: row.get::<Option<Uuid>, _>("department_id"),
                    admin_role,
                    created_at: row.get::<Option<DateTime<Utc>>, _>("created_at"),
                    last_login: row.get::<Option<DateTime<Utc>>, _>("last_login"),
                    is_active: row.get::<Option<bool>, _>("is_active").unwrap_or(false),
                };

                admin_users.push(admin_user);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "users": admin_users,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset
                },
                "message": "Admin users retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve admin users"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get admin activities with enhanced information
pub async fn get_admin_activities(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
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

    let status_filter = params.get("status");
    let search = params.get("search").cloned();

    let mut query = r#"
        SELECT 
            a.id,
            a.title,
            a.description,
            a.location,
            a.start_time,
            a.end_time,
            a.max_participants,
            a.status,
            a.created_at,
            u.first_name || ' ' || u.last_name as created_by_name,
            f.name as faculty_name,
            d.name as department_name,
            COALESCE(COUNT(p.id), 0) as current_participants
        FROM activities a
        LEFT JOIN users u ON a.created_by = u.id
        LEFT JOIN faculties f ON a.faculty_id = f.id
        LEFT JOIN departments d ON a.department_id = d.id
        LEFT JOIN participations p ON a.id = p.activity_id
    "#
    .to_string();

    let mut count_query = r#"
        SELECT COUNT(DISTINCT a.id) 
        FROM activities a
        LEFT JOIN users u ON a.created_by = u.id
        LEFT JOIN faculties f ON a.faculty_id = f.id
        LEFT JOIN departments d ON a.department_id = d.id
    "#
    .to_string();

    let mut conditions = Vec::new();
    let mut param_count = 3;

    if let Some(_status) = status_filter {
        conditions.push(format!("a.status = ${}", param_count));
        param_count += 1;
    }

    if search.is_some() {
        conditions.push(format!(
            "(a.title ILIKE ${} OR a.description ILIKE ${})",
            param_count, param_count
        ));
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        query.push_str(&where_clause);
        count_query.push_str(&where_clause);
    }

    query.push_str(" GROUP BY a.id, a.title, a.description, a.location, a.start_time, a.end_time, a.max_participants, a.status, a.created_at, u.first_name, u.last_name, f.name, d.name");
    query.push_str(" ORDER BY a.created_at DESC LIMIT $1 OFFSET $2");

    let mut query_builder = sqlx::query(&query).bind(limit).bind(offset);

    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(status) = status_filter {
        query_builder = query_builder.bind(status);
        count_query_builder = count_query_builder.bind(status);
    }

    if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        query_builder = query_builder.bind(search_pattern.clone());
        count_query_builder = count_query_builder.bind(search_pattern);
    }

    let activities_result = query_builder.fetch_all(&session_state.db_pool).await;
    let total_count_result = count_query_builder.fetch_one(&session_state.db_pool).await;

    match (activities_result, total_count_result) {
        (Ok(rows), Ok(total_count)) => {
            let mut admin_activities = Vec::new();

            for row in rows {
                let admin_activity = AdminActivityInfo {
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
                    created_by_name: row.get("created_by_name"),
                    faculty_name: row.get::<Option<String>, _>("faculty_name"),
                    department_name: row.get::<Option<String>, _>("department_name"),
                    created_at: row.get("created_at"),
                };

                admin_activities.push(admin_activity);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "activities": admin_activities,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset
                },
                "message": "Admin activities retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve admin activities"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get admin sessions information
pub async fn get_admin_sessions(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let search = params.get("search").cloned();

    // Get active session IDs from Redis
    let session_ids_result = session_state
        .redis_store
        .get_active_sessions(Some(limit * 2))
        .await;

    match session_ids_result {
        Ok(session_ids) => {
            let mut admin_sessions = Vec::new();

            for session_id in &session_ids {
                if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
                    // Get user info
                    if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                        // Apply search filter
                        if let Some(search_term) = &search {
                            let search_lower = search_term.to_lowercase();
                            let user_match = user.first_name.to_lowercase().contains(&search_lower)
                                || user.last_name.to_lowercase().contains(&search_lower)
                                || user.email.to_lowercase().contains(&search_lower)
                                || user.student_id.to_lowercase().contains(&search_lower);

                            if !user_match {
                                continue;
                            }
                        }

                        // Get admin role
                        let admin_role = get_user_admin_role(&session_state, user.id)
                            .await
                            .ok()
                            .flatten();

                        let session_info = AdminSessionInfo {
                            session_id: session.id,
                            user_id: user.id,
                            user_name: format!("{} {}", user.first_name, user.last_name),
                            student_id: user.student_id,
                            email: user.email,
                            admin_level: admin_role.as_ref().map(|r| r.admin_level.clone()),
                            faculty_name: None, // TODO: Join with faculty table if needed
                            department_name: None, // TODO: Join with department table
                            session_type: session.session_type,
                            login_method: session.login_method,
                            device_info: session.device_info,
                            ip_address: session.ip_address,
                            user_agent: session.user_agent,
                            created_at: session.created_at,
                            last_accessed: session.last_accessed,
                            expires_at: session.expires_at,
                            permissions: session.permissions,
                            sse_connections: session.sse_connections,
                            is_active: session.is_active,
                            recent_activities: session.activity_log,
                        };

                        admin_sessions.push(session_info);

                        if admin_sessions.len() >= limit {
                            break;
                        }
                    }
                }
            }

            let total_count = session_state
                .redis_store
                .get_session_count()
                .await
                .map_err(|_| {
                    let error_response = json!({
                        "status": "error",
                        "message": "Failed to get session count"
                    });
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                })?;

            let response = json!({
                "status": "success",
                "data": {
                    "sessions": admin_sessions,
                    "total_count": total_count,
                    "limit": limit,
                    "filtered_count": admin_sessions.len()
                },
                "message": "Admin sessions retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve session information from Redis"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

// Helper functions
async fn get_user_by_id(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<User>, anyhow::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    Ok(user)
}

async fn get_user_admin_role(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<AdminRole>, anyhow::Error> {
    let admin_role = sqlx::query_as::<_, AdminRole>("SELECT * FROM admin_roles WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    Ok(admin_role)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdminRequest {
    pub student_id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
    pub admin_level: AdminLevel,
    pub faculty_id: Option<Uuid>,
    pub permissions: Vec<String>,
}

/// Create new admin account (user + admin role)
pub async fn create_admin(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Json(request): Json<CreateAdminRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user already exists
    let existing_user = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1 OR student_id = $2",
    )
    .bind(&request.email)
    .bind(&request.student_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match existing_user {
        Ok(count) if count > 0 => {
            let error_response = json!({
                "status": "error",
                "message": "User with this email or student ID already exists"
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check existing user"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match bcrypt::hash(&request.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to hash password"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Generate QR secret
    let qr_secret = Uuid::new_v4().to_string();

    // Start transaction
    let mut tx = match session_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to start transaction"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Create user
    let user_result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (student_id, email, password_hash, first_name, last_name, qr_secret, department_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, student_id, email, password_hash, first_name, last_name, qr_secret, department_id, created_at, updated_at
        "#
    )
    .bind(&request.student_id)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(&qr_secret)
    .bind(request.department_id)
    .fetch_one(&mut *tx)
    .await;

    let user = match user_result {
        Ok(user) => user,
        Err(e) => {
            let _ = tx.rollback().await;
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create user: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Create admin role
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
        "#
    )
    .bind(user.id)
    .bind(&request.admin_level)
    .bind(request.faculty_id)
    .bind(&request.permissions)
    .fetch_one(&mut *tx)
    .await;

    let admin_role = match admin_role_result {
        Ok(role) => role,
        Err(e) => {
            let _ = tx.rollback().await;
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create admin role: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Commit transaction
    if let Err(_) = tx.commit().await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to commit transaction"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "data": {
            "user": {
                "id": user.id,
                "student_id": user.student_id,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "department_id": user.department_id,
                "created_at": user.created_at,
                "updated_at": user.updated_at
            },
            "admin_role": {
                "id": admin_role.id,
                "admin_level": admin_role.admin_level,
                "faculty_id": admin_role.faculty_id,
                "permissions": admin_role.permissions,
                "created_at": admin_role.created_at,
                "updated_at": admin_role.updated_at
            }
        },
        "message": "Admin account created successfully"
    });

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleAdminStatusRequest {
    pub is_active: bool,
}

/// Toggle admin status by managing permissions
pub async fn toggle_admin_status(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Path(admin_role_id): Path<Uuid>,
    Json(request): Json<ToggleAdminStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get the admin role first
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        "SELECT * FROM admin_roles WHERE id = $1"
    )
    .bind(admin_role_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let admin_role = match admin_role_result {
        Ok(Some(role)) => role,
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "Admin role not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to fetch admin role"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Define default permissions based on admin level
    let default_permissions = match admin_role.admin_level {
        AdminLevel::SuperAdmin => vec![
            "ManageUsers".to_string(),
            "ManageAdmins".to_string(),
            "ManageActivities".to_string(),
            "ViewDashboard".to_string(),
            "ManageFaculties".to_string(),
            "ManageSessions".to_string(),
        ],
        AdminLevel::FacultyAdmin => vec![
            "ViewDashboard".to_string(),
            "ManageActivities".to_string(),
            "ManageUsers".to_string(),
        ],
        AdminLevel::RegularAdmin => vec![
            "ViewDashboard".to_string(),
            "ManageActivities".to_string(),
        ],
    };

    // Set permissions based on active status
    let new_permissions = if request.is_active {
        default_permissions
    } else {
        vec![] // Empty permissions means inactive
    };

    // Update the admin role permissions
    let update_result = sqlx::query_as::<_, AdminRole>(
        r#"
        UPDATE admin_roles 
        SET permissions = $1, updated_at = NOW() 
        WHERE id = $2
        RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
        "#
    )
    .bind(&new_permissions)
    .bind(admin_role_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match update_result {
        Ok(updated_role) => {
            // If deactivating, also revoke any active sessions for this user
            if !request.is_active {
                // Note: For now we skip session revocation as we'd need to implement
                // a method to find all sessions for a user and revoke them individually
                // This is a future enhancement
            }

            let response = json!({
                "status": "success",
                "data": {
                    "admin_role": updated_role,
                    "is_active": request.is_active,
                    "action": if request.is_active { "activated" } else { "deactivated" }
                },
                "message": format!("Admin status {} successfully", 
                    if request.is_active { "activated" } else { "deactivated" })
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error", 
                "message": format!("Failed to update admin status: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapAdminRequest {
    pub student_id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

/// Bootstrap initial super admin account (only works when no super admin exists)
pub async fn bootstrap_admin(
    State(session_state): State<SessionState>,
    Json(request): Json<BootstrapAdminRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if any super admin already exists
    let super_admin_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM admin_roles WHERE admin_level = 'super_admin'",
    )
    .fetch_one(&session_state.db_pool)
    .await;

    match super_admin_count {
        Ok(count) if count > 0 => {
            let error_response = json!({
                "status": "error",
                "message": "Super admin already exists. Bootstrap is disabled."
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check existing super admins"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Check if user already exists
    let existing_user = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1 OR student_id = $2",
    )
    .bind(&request.email)
    .bind(&request.student_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match existing_user {
        Ok(count) if count > 0 => {
            let error_response = json!({
                "status": "error",
                "message": "User with this email or student ID already exists"
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check existing user"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match bcrypt::hash(&request.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to hash password"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Generate QR secret
    let qr_secret = Uuid::new_v4().to_string();

    // Start transaction
    let mut tx = match session_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to start transaction"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Create super admin user (with null department_id)
    let user_result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (student_id, email, password_hash, first_name, last_name, qr_secret, department_id)
        VALUES ($1, $2, $3, $4, $5, $6, NULL)
        RETURNING id, student_id, email, password_hash, first_name, last_name, qr_secret, department_id, created_at, updated_at
        "#
    )
    .bind(&request.student_id)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(&qr_secret)
    .fetch_one(&mut *tx)
    .await;

    let user = match user_result {
        Ok(user) => user,
        Err(e) => {
            let _ = tx.rollback().await;
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create super admin user: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Define super admin permissions
    let super_admin_permissions = vec![
        "ViewSystemReports".to_string(),
        "ManageAllFaculties".to_string(), 
        "ManageUsers".to_string(),
        "ManageActivities".to_string(),
        "ManageAdmins".to_string(),
        "ManageSessions".to_string(),
        "ViewAllReports".to_string(),
    ];

    // Create super admin role (with null faculty_id)
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions)
        VALUES ($1, 'super_admin', NULL, $2)
        RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
        "#
    )
    .bind(user.id)
    .bind(&super_admin_permissions)
    .fetch_one(&mut *tx)
    .await;

    let admin_role = match admin_role_result {
        Ok(role) => role,
        Err(e) => {
            let _ = tx.rollback().await;
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create super admin role: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Commit transaction
    if let Err(_) = tx.commit().await {
        let error_response = json!({
            "status": "error",
            "message": "Failed to commit transaction"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "data": {
            "user": {
                "id": user.id,
                "student_id": user.student_id,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "department_id": user.department_id,
                "created_at": user.created_at,
                "updated_at": user.updated_at
            },
            "admin_role": {
                "id": admin_role.id,
                "admin_level": admin_role.admin_level,
                "faculty_id": admin_role.faculty_id,
                "permissions": admin_role.permissions,
                "created_at": admin_role.created_at,
                "updated_at": admin_role.updated_at
            }
        },
        "message": "Super admin bootstrap completed successfully"
    });

    Ok(Json(response))
}
