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
    pub is_active: bool,    // Whether admin has active login session
    pub is_enabled: bool,   // Whether admin account is enabled (can login)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminActivityInfo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub start_time_only: Option<chrono::NaiveTime>,
    pub end_time_only: Option<chrono::NaiveTime>,
    pub activity_type: Option<String>,
    pub max_participants: Option<i32>,
    pub status: ActivityStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub academic_year: Option<String>,
    pub organizer: Option<String>,
    pub eligible_faculties: Option<serde_json::Value>,
    pub hours: Option<i32>,
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
            ar.is_enabled,
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
                        is_enabled: row.get::<Option<bool>, _>("is_enabled").unwrap_or(true),
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
                    is_enabled: row.get::<Option<bool>, _>("is_enabled").unwrap_or(true),
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
            a.start_date,
            a.end_date,
            a.start_time_only,
            a.end_time_only,
            a.activity_type,
            a.max_participants,
            a.status,
            a.created_at,
            a.updated_at,
            a.academic_year,
            a.organizer,
            a.eligible_faculties,
            a.hours
        FROM activities a
    "#
    .to_string();

    let mut count_query = r#"
        SELECT COUNT(*) 
        FROM activities a
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

    // No grouping needed since no aggregate
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
                    start_date: row.get::<Option<chrono::NaiveDate>, _>("start_date"),
                    end_date: row.get::<Option<chrono::NaiveDate>, _>("end_date"),
                    start_time_only: row.get::<Option<chrono::NaiveTime>, _>("start_time_only"),
                    end_time_only: row.get::<Option<chrono::NaiveTime>, _>("end_time_only"),
                    activity_type: row.get::<Option<String>, _>("activity_type"),
                    max_participants: row.get::<Option<i32>, _>("max_participants"),
                    status: row.get::<ActivityStatus, _>("status"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    academic_year: row.get::<Option<String>, _>("academic_year"),
                    organizer: row.get::<Option<String>, _>("organizer"),
                    eligible_faculties: row.get::<Option<serde_json::Value>, _>("eligible_faculties"),
                    hours: row.get::<Option<i32>, _>("hours"),
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

    // Create admin role (with is_enabled = true by default)
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions, is_enabled)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, admin_level, faculty_id, permissions, is_enabled, created_at, updated_at
        "#
    )
    .bind(user.id)
    .bind(&request.admin_level)
    .bind(request.faculty_id)
    .bind(&request.permissions)
    .bind(true) // is_enabled = true by default
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
    pub is_enabled: bool,  // Changed from is_active to is_enabled
}

/// Toggle admin enabled status (not login activity status)
pub async fn toggle_admin_status(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Path(admin_role_id): Path<Uuid>,
    Json(request): Json<ToggleAdminStatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get the admin role first to verify it exists
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        "SELECT * FROM admin_roles WHERE id = $1"
    )
    .bind(admin_role_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let _admin_role = match admin_role_result {
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

    // Update the is_enabled field directly (don't modify permissions)
    let update_result = sqlx::query_as::<_, AdminRole>(
        r#"
        UPDATE admin_roles 
        SET is_enabled = $1, updated_at = NOW() 
        WHERE id = $2
        RETURNING id, user_id, admin_level, faculty_id, permissions, is_enabled, created_at, updated_at
        "#
    )
    .bind(request.is_enabled)
    .bind(admin_role_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match update_result {
        Ok(updated_role) => {
            // If disabling admin, optionally revoke active sessions for this user
            if !request.is_enabled {
                // TODO: Implement session revocation for disabled admins
                // This could be done by calling redis_session.revoke_user_sessions(user_id)
                // For now, we'll leave sessions active but the middleware should check is_enabled
            }

            let response = json!({
                "status": "success",
                "data": {
                    "admin_role": updated_role,
                    "is_enabled": request.is_enabled,
                    "action": if request.is_enabled { "enabled" } else { "disabled" }
                },
                "message": format!("Admin account {} successfully", 
                    if request.is_enabled { "enabled" } else { "disabled" })
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

    // Create super admin role (with null faculty_id and is_enabled = true)
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions, is_enabled)
        VALUES ($1, 'super_admin', NULL, $2, $3)
        RETURNING id, user_id, admin_level, faculty_id, permissions, is_enabled, created_at, updated_at
        "#
    )
    .bind(user.id)
    .bind(&super_admin_permissions)
    .bind(true) // is_enabled = true for super admin
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

/// Get admins in a faculty with proper authorization
/// FacultyAdmin+ for their faculty, SuperAdmin for any
pub async fn get_faculty_admins(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    admin: FacultyAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check authorization - FacultyAdmin can only access their own faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only access admins in your faculty"
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
            ar.is_enabled,
            ar.created_at as role_created_at,
            ar.updated_at as role_updated_at,
            (SELECT MAX(s.last_accessed) FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) as last_login,
            CASE WHEN EXISTS(SELECT 1 FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) THEN true ELSE false END as is_active
        FROM users u
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        WHERE ar.id IS NOT NULL AND ar.faculty_id = $3
    "#
    .to_string();

    let mut count_query = "SELECT COUNT(*) FROM users u LEFT JOIN admin_roles ar ON u.id = ar.user_id WHERE ar.id IS NOT NULL AND ar.faculty_id = $1".to_string();

    if let Some(_search_term) = &search {
        let where_clause = " AND (u.first_name ILIKE $4 OR u.last_name ILIKE $4 OR u.email ILIKE $4 OR u.student_id ILIKE $4)";
        query.push_str(where_clause);
        count_query.push_str(" AND (u.first_name ILIKE $2 OR u.last_name ILIKE $2 OR u.email ILIKE $2 OR u.student_id ILIKE $2)");
    }

    query.push_str(" ORDER BY u.created_at DESC LIMIT $1 OFFSET $2");

    let users_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .bind(faculty_id)
            .bind(search_pattern.clone())
            .fetch_all(&session_state.db_pool)
            .await
    } else {
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .bind(faculty_id)
            .fetch_all(&session_state.db_pool)
            .await
    };

    let total_count_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query_scalar::<_, i64>(&count_query)
            .bind(faculty_id)
            .bind(search_pattern)
            .fetch_one(&session_state.db_pool)
            .await
    } else {
        sqlx::query_scalar::<_, i64>(&count_query)
            .bind(faculty_id)
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
                        is_enabled: row.get::<Option<bool>, _>("is_enabled").unwrap_or(true),
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
                    is_enabled: row.get::<Option<bool>, _>("is_enabled").unwrap_or(true),
                };

                admin_users.push(admin_user);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "admins": admin_users,
                    "faculty_id": faculty_id,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset
                },
                "message": "Faculty admins retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve faculty admins"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get users in a faculty with proper authorization
/// FacultyAdmin+ for their faculty, SuperAdmin for any
pub async fn get_faculty_users(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    admin: FacultyAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check authorization - FacultyAdmin can only access their own faculty
    if admin.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin.admin_role.faculty_id != Some(faculty_id) {
            let error_response = json!({
                "status": "error",
                "message": "Access denied: You can only access users in your faculty"
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

    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50);

    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0);

    let search = params.get("search").cloned();
    let include_admins = params
        .get("include_admins")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let mut query = r#"
        SELECT DISTINCT
            u.id,
            u.student_id,
            u.email,
            u.first_name,
            u.last_name,
            u.department_id,
            u.created_at,
            u.updated_at,
            d.name as department_name,
            f.id as faculty_id,
            f.name as faculty_name,
            f.code as faculty_code,
            (SELECT MAX(s.last_accessed) FROM sessions s WHERE s.user_id = u.id) as last_login,
            CASE WHEN EXISTS(SELECT 1 FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) THEN true ELSE false END as is_active,
            ar.id as admin_role_id,
            ar.admin_level,
            ar.permissions
        FROM users u
        JOIN departments d ON u.department_id = d.id
        LEFT JOIN faculties f ON d.faculty_id = f.id
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        WHERE d.faculty_id = $3
    "#
    .to_string();

    let mut count_query = r#"
        SELECT COUNT(DISTINCT u.id)
        FROM users u
        JOIN departments d ON u.department_id = d.id
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        WHERE d.faculty_id = $1
    "#.to_string();

    // Filter out admin users if not requested
    if !include_admins {
        let admin_filter = " AND ar.id IS NULL";
        query.push_str(admin_filter);
        count_query.push_str(admin_filter);
    }

    if let Some(_search_term) = &search {
        let where_clause = " AND (u.first_name ILIKE $4 OR u.last_name ILIKE $4 OR u.email ILIKE $4 OR u.student_id ILIKE $4)";
        query.push_str(where_clause);
        count_query.push_str(" AND (u.first_name ILIKE $2 OR u.last_name ILIKE $2 OR u.email ILIKE $2 OR u.student_id ILIKE $2)");
    }

    query.push_str(" ORDER BY u.last_name, u.first_name LIMIT $1 OFFSET $2");

    let users_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .bind(faculty_id)
            .bind(search_pattern.clone())
            .fetch_all(&session_state.db_pool)
            .await
    } else {
        sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .bind(faculty_id)
            .fetch_all(&session_state.db_pool)
            .await
    };

    let total_count_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query_scalar::<_, i64>(&count_query)
            .bind(faculty_id)
            .bind(search_pattern)
            .fetch_one(&session_state.db_pool)
            .await
    } else {
        sqlx::query_scalar::<_, i64>(&count_query)
            .bind(faculty_id)
            .fetch_one(&session_state.db_pool)
            .await
    };

    match (users_result, total_count_result) {
        (Ok(rows), Ok(total_count)) => {
            let mut users = Vec::new();

            for row in rows {
                let user_info = json!({
                    "id": row.get::<Uuid, _>("id"),
                    "student_id": row.get::<String, _>("student_id"),
                    "email": row.get::<String, _>("email"),
                    "first_name": row.get::<String, _>("first_name"),
                    "last_name": row.get::<String, _>("last_name"),
                    "department_id": row.get::<Option<Uuid>, _>("department_id"),
                    "department_name": row.get::<Option<String>, _>("department_name"),
                    "faculty_id": row.get::<Option<Uuid>, _>("faculty_id"),
                    "faculty_name": row.get::<Option<String>, _>("faculty_name"),
                    "faculty_code": row.get::<Option<String>, _>("faculty_code"),
                    "created_at": row.get::<Option<DateTime<Utc>>, _>("created_at"),
                    "updated_at": row.get::<Option<DateTime<Utc>>, _>("updated_at"),
                    "last_login": row.get::<Option<DateTime<Utc>>, _>("last_login"),
                    "is_active": row.get::<Option<bool>, _>("is_active").unwrap_or(false),
                    "is_admin": row.get::<Option<Uuid>, _>("admin_role_id").is_some(),
                    "admin_level": row.get::<Option<AdminLevel>, _>("admin_level"),
                    "permissions": row.get::<Option<Vec<String>>, _>("permissions").unwrap_or_else(|| vec![])
                });

                users.push(user_info);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "users": users,
                    "faculty_id": faculty_id,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset,
                    "include_admins": include_admins
                },
                "message": "Faculty users retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve faculty users"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create admin in faculty (SuperAdmin can create any, FacultyAdmin can create RegularAdmin in their faculty)
pub async fn create_faculty_admin(
    State(session_state): State<SessionState>,
    Path(faculty_id): Path<Uuid>,
    admin: FacultyAdminUser,
    Json(mut request): Json<CreateAdminRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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

    // Force the faculty_id to match the path parameter
    request.faculty_id = Some(faculty_id);

    // Check authorization based on admin levels
    match admin.admin_role.admin_level {
        AdminLevel::SuperAdmin => {
            // SuperAdmin can create any faculty admin in any faculty
        }
        AdminLevel::FacultyAdmin => {
            // FacultyAdmin can only create RegularAdmin in their own faculty
            if admin.admin_role.faculty_id != Some(faculty_id) {
                let error_response = json!({
                    "status": "error",
                    "message": "Faculty Admin can only create admins in their own faculty"
                });
                return Err((StatusCode::FORBIDDEN, Json(error_response)));
            }
            if request.admin_level != AdminLevel::RegularAdmin {
                let error_response = json!({
                    "status": "error",
                    "message": "Faculty Admin can only create Regular Admin accounts"
                });
                return Err((StatusCode::FORBIDDEN, Json(error_response)));
            }
        }
        AdminLevel::RegularAdmin => {
            // Regular admins cannot create other admins
            let error_response = json!({
                "status": "error",
                "message": "Regular Admin does not have permission to create admin accounts"
            });
            return Err((StatusCode::FORBIDDEN, Json(error_response)));
        }
    }

    // Validate admin level is appropriate for faculty
    match request.admin_level {
        AdminLevel::SuperAdmin => {
            let error_response = json!({
                "status": "error",
                "message": "Cannot create SuperAdmin with faculty assignment"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
        AdminLevel::FacultyAdmin | AdminLevel::RegularAdmin => {
            // These are valid for faculty assignment
        }
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

    // Create admin role (with is_enabled = true by default)
    let admin_role_result = sqlx::query_as::<_, AdminRole>(
        r#"
        INSERT INTO admin_roles (user_id, admin_level, faculty_id, permissions, is_enabled)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, admin_level, faculty_id, permissions, is_enabled, created_at, updated_at
        "#
    )
    .bind(user.id)
    .bind(&request.admin_level)
    .bind(request.faculty_id)
    .bind(&request.permissions)
    .bind(true) // is_enabled = true by default
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
        "message": "Faculty admin created successfully"
    });

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAdminRoleRequest {
    pub admin_level: Option<AdminLevel>,
    pub faculty_id: Option<Option<Uuid>>, // Option<Option<T>> to distinguish between not provided vs explicitly setting to null
    pub permissions: Option<Vec<String>>,
}

/// Update admin role (SuperAdmin only)
pub async fn update_admin_role(
    State(session_state): State<SessionState>,
    Path(admin_role_id): Path<Uuid>,
    _admin: SuperAdminUser,
    Json(request): Json<UpdateAdminRoleRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get the current admin role
    let current_role = sqlx::query_as::<_, AdminRole>(
        "SELECT * FROM admin_roles WHERE id = $1"
    )
    .bind(admin_role_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    let current_role = match current_role {
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

    // Validate the update request
    if let Some(new_level) = &request.admin_level {
        match new_level {
            AdminLevel::SuperAdmin => {
                // If changing to SuperAdmin, faculty_id should be null
                if let Some(Some(_)) = request.faculty_id {
                    let error_response = json!({
                        "status": "error",
                        "message": "SuperAdmin cannot be assigned to a specific faculty"
                    });
                    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
                }
            }
            AdminLevel::FacultyAdmin | AdminLevel::RegularAdmin => {
                // Faculty admins should have a faculty_id
                let final_faculty_id = match &request.faculty_id {
                    Some(Some(id)) => Some(*id),
                    Some(None) => None,
                    None => current_role.faculty_id,
                };
                
                if final_faculty_id.is_none() {
                    let error_response = json!({
                        "status": "error",
                        "message": "Faculty and Regular admins must be assigned to a faculty"
                    });
                    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
                }
            }
        }
    }

    // Build dynamic update query
    let mut query = "UPDATE admin_roles SET updated_at = NOW()".to_string();
    let mut param_count = 1;
    let mut query_builder = sqlx::query_as::<_, AdminRole>("");

    if let Some(_) = &request.admin_level {
        query.push_str(&format!(", admin_level = ${}", param_count));
        param_count += 1;
    }

    if let Some(_) = &request.faculty_id {
        query.push_str(&format!(", faculty_id = ${}", param_count));
        param_count += 1;
    }

    if let Some(_) = &request.permissions {
        query.push_str(&format!(", permissions = ${}", param_count));
        param_count += 1;
    }

    query.push_str(&format!(
        " WHERE id = ${} RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at",
        param_count
    ));

    query_builder = sqlx::query_as::<_, AdminRole>(&query);

    if let Some(admin_level) = &request.admin_level {
        query_builder = query_builder.bind(admin_level);
    }

    if let Some(faculty_id) = &request.faculty_id {
        query_builder = query_builder.bind(faculty_id);
    }

    if let Some(permissions) = &request.permissions {
        query_builder = query_builder.bind(permissions);
    }

    query_builder = query_builder.bind(admin_role_id);

    match query_builder.fetch_one(&session_state.db_pool).await {
        Ok(updated_role) => {
            let response = json!({
                "status": "success",
                "data": {
                    "admin_role": updated_role,
                    "updated_fields": {
                        "admin_level_changed": request.admin_level.is_some(),
                        "faculty_id_changed": request.faculty_id.is_some(),
                        "permissions_changed": request.permissions.is_some()
                    }
                },
                "message": "Admin role updated successfully"
            });
            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Admin role not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update admin role: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get all system admins with faculty grouping (SuperAdmin only)
pub async fn get_all_system_admins(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
            ar.is_enabled,
            ar.created_at as role_created_at,
            ar.updated_at as role_updated_at,
            f.name as faculty_name,
            f.code as faculty_code,
            d.name as department_name,
            d.code as department_code,
            (SELECT MAX(s.last_accessed) FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) as last_login,
            CASE WHEN EXISTS(SELECT 1 FROM sessions s WHERE s.user_id = u.id AND s.is_active = true) THEN true ELSE false END as is_active
        FROM users u
        JOIN admin_roles ar ON u.id = ar.user_id
        LEFT JOIN faculties f ON ar.faculty_id = f.id
        LEFT JOIN departments d ON u.department_id = d.id
    "#.to_string();

    if let Some(_search_term) = &search {
        let where_clause = " WHERE (u.first_name ILIKE $1 OR u.last_name ILIKE $1 OR u.email ILIKE $1 OR u.student_id ILIKE $1 OR f.name ILIKE $1)";
        query.push_str(where_clause);
    }

    query.push_str(" ORDER BY ar.admin_level, f.name NULLS FIRST, u.first_name");

    let users_result = if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        sqlx::query(&query)
            .bind(search_pattern)
            .fetch_all(&session_state.db_pool)
            .await
    } else {
        sqlx::query(&query)
            .fetch_all(&session_state.db_pool)
            .await
    };

    match users_result {
        Ok(rows) => {
            let mut grouped_admins: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
            let mut super_admins = Vec::new();
            let mut total_count = 0;

            for row in rows {
                total_count += 1;
                let admin_level: AdminLevel = row.get("admin_level");
                let faculty_id: Option<Uuid> = row.get("faculty_id");

                let admin_info = json!({
                    "id": row.get::<Uuid, _>("id"),
                    "student_id": row.get::<String, _>("student_id"),
                    "email": row.get::<String, _>("email"),
                    "first_name": row.get::<String, _>("first_name"),
                    "last_name": row.get::<String, _>("last_name"),
                    "department_id": row.get::<Option<Uuid>, _>("department_id"),
                    "department_name": row.get::<Option<String>, _>("department_name"),
                    "department_code": row.get::<Option<String>, _>("department_code"),
                    "created_at": row.get::<Option<DateTime<Utc>>, _>("created_at"),
                    "admin_role": {
                        "id": row.get::<Uuid, _>("admin_role_id"),
                        "admin_level": admin_level,
                        "faculty_id": faculty_id,
                        "permissions": row.get::<Vec<String>, _>("permissions"),
                        "is_enabled": row.get::<bool, _>("is_enabled"),
                        "created_at": row.get::<Option<DateTime<Utc>>, _>("role_created_at"),
                        "updated_at": row.get::<Option<DateTime<Utc>>, _>("role_updated_at")
                    },
                    "last_login": row.get::<Option<DateTime<Utc>>, _>("last_login"),
                    "is_active": row.get::<bool, _>("is_active"),
                    "is_enabled": row.get::<bool, _>("is_enabled")
                });

                match admin_level {
                    AdminLevel::SuperAdmin => {
                        super_admins.push(admin_info);
                    },
                    _ => {
                        let faculty_key = if let Some(faculty_id) = faculty_id {
                            format!("{}|{}", 
                                faculty_id, 
                                row.get::<Option<String>, _>("faculty_name").unwrap_or("Unknown Faculty".to_string())
                            )
                        } else {
                            "no_faculty|No Faculty Assigned".to_string()
                        };
                        
                        grouped_admins.entry(faculty_key).or_insert_with(Vec::new).push(admin_info);
                    }
                }
            }

            // Build the response
            let mut faculty_groups = Vec::new();
            for (key, admins) in grouped_admins {
                let parts: Vec<&str> = key.split('|').collect();
                let faculty_id_str = parts[0];
                let faculty_name = parts[1];
                
                let faculty_group = json!({
                    "faculty_id": if faculty_id_str == "no_faculty" { serde_json::Value::Null } else { json!(faculty_id_str) },
                    "faculty_name": faculty_name,
                    "admins": admins,
                    "admin_count": admins.len()
                });
                faculty_groups.push(faculty_group);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "super_admins": super_admins,
                    "faculty_groups": faculty_groups,
                    "total_count": total_count
                },
                "message": "All system admins retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to retrieve system admins: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Bulk admin operations (SuperAdmin only)
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkAdminOperationRequest {
    pub operation: String, // "activate", "deactivate", "delete", "update_faculty"
    pub admin_role_ids: Vec<Uuid>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

pub async fn bulk_admin_operations(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Json(request): Json<BulkAdminOperationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if request.admin_role_ids.is_empty() {
        let error_response = json!({
            "status": "error",
            "message": "No admin role IDs provided"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let mut tx = match session_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to start transaction: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let mut results = Vec::new();

    match request.operation.as_str() {
        "activate" => {
            for role_id in &request.admin_role_ids {
                let result = sqlx::query_as::<_, AdminRole>(
                    r#"
                    UPDATE admin_roles 
                    SET permissions = CASE 
                        WHEN admin_level = 'super_admin' THEN $2
                        WHEN admin_level = 'faculty_admin' THEN $3
                        ELSE $4
                    END, updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
                    "#
                )
                .bind(role_id)
                .bind(&vec!["ManageUsers".to_string(), "ManageAdmins".to_string(), "ManageActivities".to_string(), "ViewDashboard".to_string(), "ManageFaculties".to_string(), "ManageSessions".to_string()])
                .bind(&vec!["ViewDashboard".to_string(), "ManageActivities".to_string(), "ManageUsers".to_string()])
                .bind(&vec!["ViewDashboard".to_string(), "ManageActivities".to_string()])
                .fetch_one(&mut *tx)
                .await;

                match result {
                    Ok(admin_role) => results.push(json!({
                        "role_id": role_id,
                        "status": "success",
                        "message": "Admin activated successfully",
                        "admin_role": admin_role
                    })),
                    Err(e) => results.push(json!({
                        "role_id": role_id,
                        "status": "error",
                        "message": format!("Failed to activate admin: {}", e)
                    }))
                }
            }
        }
        "deactivate" => {
            for role_id in &request.admin_role_ids {
                let result = sqlx::query_as::<_, AdminRole>(
                    r#"
                    UPDATE admin_roles 
                    SET permissions = '{}', updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
                    "#
                )
                .bind(role_id)
                .fetch_one(&mut *tx)
                .await;

                match result {
                    Ok(admin_role) => results.push(json!({
                        "role_id": role_id,
                        "status": "success",
                        "message": "Admin deactivated successfully",
                        "admin_role": admin_role
                    })),
                    Err(e) => results.push(json!({
                        "role_id": role_id,
                        "status": "error",
                        "message": format!("Failed to deactivate admin: {}", e)
                    }))
                }
            }
        }
        "update_faculty" => {
            let new_faculty_id = request.parameters
                .as_ref()
                .and_then(|p| p.get("faculty_id"))
                .and_then(|v| v.as_str())
                .and_then(|s| if s == "null" { None } else { Some(Uuid::parse_str(s).ok()) })
                .flatten();

            for role_id in &request.admin_role_ids {
                let result = sqlx::query_as::<_, AdminRole>(
                    r#"
                    UPDATE admin_roles 
                    SET faculty_id = $2, updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, user_id, admin_level, faculty_id, permissions, created_at, updated_at
                    "#
                )
                .bind(role_id)
                .bind(new_faculty_id)
                .fetch_one(&mut *tx)
                .await;

                match result {
                    Ok(admin_role) => results.push(json!({
                        "role_id": role_id,
                        "status": "success", 
                        "message": "Faculty assignment updated successfully",
                        "admin_role": admin_role
                    })),
                    Err(e) => results.push(json!({
                        "role_id": role_id,
                        "status": "error",
                        "message": format!("Failed to update faculty assignment: {}", e)
                    }))
                }
            }
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid bulk operation"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    if let Err(e) = tx.commit().await {
        let error_response = json!({
            "status": "error",
            "message": format!("Failed to commit bulk operation: {}", e)
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let successful_count = results.iter().filter(|r| r["status"] == "success").count();
    let failed_count = results.len() - successful_count;

    let response = json!({
        "status": "success",
        "data": {
            "operation": request.operation,
            "results": results,
            "summary": {
                "total_attempted": request.admin_role_ids.len(),
                "successful": successful_count,
                "failed": failed_count
            }
        },
        "message": format!("Bulk {} operation completed: {} successful, {} failed", 
            request.operation, successful_count, failed_count)
    });

    Ok(Json(response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdminActivityRequest {
    pub activity_name: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub start_time: String,
    pub end_time: String,
    pub activity_type: String,
    pub location: String,
    pub max_participants: Option<i32>,
    pub organizer: String,
    pub eligible_faculties: Vec<Uuid>,
    pub academic_year: String,
    pub hours: i32,
}

/// Create new activity via admin interface with enhanced fields
pub async fn create_admin_activity(
    State(session_state): State<SessionState>,
    admin: FacultyAdminUser,
    Json(request): Json<CreateAdminActivityRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate activity type
    let valid_activity_types = vec!["Academic", "Sports", "Cultural", "Social", "Other"];
    if !valid_activity_types.contains(&request.activity_type.as_str()) {
        let error_response = json!({
            "status": "error",
            "message": "ประเภทกิจกรรมไม่ถูกต้อง"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Parse and validate dates
    let start_date = match chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "รูปแบบวันที่เริ่มต้นไม่ถูกต้อง"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let end_date = match chrono::NaiveDate::parse_from_str(&request.end_date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "รูปแบบวันที่สิ้นสุดไม่ถูกต้อง"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Parse and validate times
    let start_time = match chrono::NaiveTime::parse_from_str(&request.start_time, "%H:%M") {
        Ok(time) => time,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "รูปแบบเวลาเริ่มต้นไม่ถูกต้อง"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let end_time = match chrono::NaiveTime::parse_from_str(&request.end_time, "%H:%M") {
        Ok(time) => time,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "รูปแบบเวลาสิ้นสุดไม่ถูกต้อง"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Combine date and time for start_time and end_time
    let start_datetime = start_date.and_time(start_time).and_utc();
    let end_datetime = end_date.and_time(end_time).and_utc();

    // Validate time range
    if start_datetime >= end_datetime {
        let error_response = json!({
            "status": "error",
            "message": "เวลาเริ่มต้นต้องน้อยกว่าเวลาสิ้นสุด"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Validate required textual fields non-empty
    if request.activity_name.trim().is_empty()
        || request.activity_type.trim().is_empty()
        || request.academic_year.trim().is_empty()
        || request.organizer.trim().is_empty()
        || request.location.trim().is_empty()
    {
        let error_response = json!({
            "status": "error",
            "message": "ข้อมูลไม่ครบถ้วน"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Validate hours
    if request.hours <= 0 {
        let error_response = json!({
            "status": "error",
            "message": "ชั่วโมงกิจกรรมต้องมากกว่า 0"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // For faculty admins, set faculty_id to their own faculty
    // For super admins, we'll need to determine faculty based on eligible_faculties
    let faculty_id = match admin.admin_role.admin_level {
        crate::models::admin_role::AdminLevel::SuperAdmin => {
            // For super admin, use the first eligible faculty if any
            request.eligible_faculties.first().copied()
        }
        _ => admin.admin_role.faculty_id
    };

    // Convert eligible_faculties to JSONB format
    let eligible_faculties_json = serde_json::to_value(&request.eligible_faculties)
        .map_err(|_| {
            let error_response = json!({
                "status": "error",
                "message": "รูปแบบคณะที่มีสิทธิ์ไม่ถูกต้อง"
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    // Get user_id from admin
    let user_id = admin.session_user.user_id;

    // Create the activity with enhanced fields
    let create_result = sqlx::query(
        r#"
        INSERT INTO activities (
            title, description, location, max_participants, 
            faculty_id, created_by, academic_year, organizer, 
            eligible_faculties, activity_type, start_date, end_date, 
            start_time_only, end_time_only, hours
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id, title, description, location,
                  ((start_date::timestamp + start_time_only) AT TIME ZONE 'UTC') as start_time,
                  ((end_date::timestamp + end_time_only) AT TIME ZONE 'UTC') as end_time,
                  max_participants, 
                  status, faculty_id, created_by, created_at, updated_at,
                  academic_year, organizer, eligible_faculties, activity_type, hours
        "#
    )
    .bind(&request.activity_name)  // title
    .bind(request.description.unwrap_or_default())  // description
    .bind(&request.location)  // location
    .bind(request.max_participants)  // max_participants
    .bind(faculty_id)  // faculty_id
    .bind(user_id)  // created_by
    .bind(&request.academic_year)  // academic_year
    .bind(&request.organizer)  // organizer
    .bind(&eligible_faculties_json)  // eligible_faculties
    .bind(&request.activity_type)  // activity_type
    .bind(start_date)  // start_date
    .bind(end_date)  // end_date
    .bind(start_time)  // start_time_only
    .bind(end_time)  // end_time_only
    .bind(request.hours) // hours
    .fetch_one(&session_state.db_pool)
    .await;

    match create_result {
        Ok(row) => {
            let activity = json!({
                "id": row.get::<Uuid, _>("id"),
                "title": row.get::<String, _>("title"),
                "description": row.get::<String, _>("description"),
                "location": row.get::<String, _>("location"),
                "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                "end_time": row.get::<DateTime<Utc>, _>("end_time"),
                "max_participants": row.get::<Option<i32>, _>("max_participants"),
                "status": row.get::<String, _>("status"),
                "faculty_id": row.get::<Option<Uuid>, _>("faculty_id"),
                "created_by": row.get::<Uuid, _>("created_by"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                "updated_at": row.get::<DateTime<Utc>, _>("updated_at"),
                "academic_year": row.get::<Option<String>, _>("academic_year"),
                "organizer": row.get::<Option<String>, _>("organizer"),
                "eligible_faculties": row.get::<Option<serde_json::Value>, _>("eligible_faculties"),
                "activity_type": row.get::<Option<String>, _>("activity_type"),
                "hours": row.get::<Option<i32>, _>("hours")
            });

            let response = json!({
                "status": "success",
                "data": activity,
                "message": "สร้างกิจกรรมสำเร็จ"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": "เกิดข้อผิดพลาดในการสร้างกิจกรรม"
            });
            eprintln!("Database error creating activity: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
