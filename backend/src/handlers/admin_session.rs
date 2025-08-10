use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
// use sqlx::Row; // Removed unused import

use crate::middleware::session::{AdminUser, FacultyAdminUser, SessionState, SuperAdminUser};
use crate::models::{
    admin_role::{AdminLevel, AdminRole},
    faculty::Faculty,
    user::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSessionListResponse {
    pub sessions: Vec<EnhancedAdminSessionInfo>,
    pub total_count: usize,
    pub active_count: usize,
    pub faculty_breakdown: HashMap<String, usize>,
    pub admin_level_breakdown: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedAdminSessionInfo {
    pub session_id: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub student_id: String,
    pub email: String,
    pub admin_level: Option<AdminLevel>,
    pub faculty_id: Option<Uuid>,
    pub faculty_name: Option<String>,
    pub department_name: Option<String>,
    pub device_info: HashMap<String, Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_current_session: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatsResponse {
    pub total_active_sessions: usize,
    pub admin_sessions: usize,
    pub student_sessions: usize,
    pub sessions_by_level: HashMap<String, usize>,
    pub sessions_by_faculty: HashMap<String, usize>,
    pub sessions_by_device: HashMap<String, usize>,
    pub top_active_users: Vec<TopActiveUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopActiveUser {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub session_count: usize,
    pub admin_level: Option<AdminLevel>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSessionActionRequest {
    pub session_ids: Vec<String>,
    pub action: String, // "revoke", "extend"
    pub reason: Option<String>,
    pub extend_hours: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSessionActionResponse {
    pub success: bool,
    pub processed_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAuditLogResponse {
    pub logs: Vec<SessionAuditEntry>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub session_id: String,
    pub user_id: Uuid,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub reason: Option<String>,
    pub details: HashMap<String, Value>,
}

// Super Admin: Get comprehensive session overview
pub async fn get_admin_sessions_overview(
    State(session_state): State<SessionState>,
    admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminSessionListResponse>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100);

    let faculty_filter = params
        .get("faculty_id")
        .and_then(|f| f.parse::<Uuid>().ok());

    let admin_level_filter = params.get("admin_level");

    // Get all active sessions
    let session_ids = session_state
        .redis_store
        .get_active_sessions(Some(limit * 2))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sessions = Vec::new();
    let mut faculty_breakdown = HashMap::new();
    let mut admin_level_breakdown = HashMap::new();

    for session_id in &session_ids {
        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            // Get detailed user info
            if let Ok(Some(enhanced_info)) =
                get_enhanced_session_info(&session_state, &admin.session_user.session_id, &session)
                    .await
            {
                // Apply filters
                if let Some(faculty_id) = faculty_filter {
                    if enhanced_info.faculty_id != Some(faculty_id) {
                        continue;
                    }
                }

                if let Some(level_filter) = admin_level_filter {
                    let session_level = enhanced_info
                        .admin_level
                        .as_ref()
                        .map(|l| format!("{:?}", l))
                        .unwrap_or_else(|| "Student".to_string());

                    if session_level != *level_filter {
                        continue;
                    }
                }

                // Update breakdowns
                if let Some(faculty_name) = &enhanced_info.faculty_name {
                    *faculty_breakdown.entry(faculty_name.clone()).or_insert(0) += 1;
                } else {
                    *faculty_breakdown
                        .entry("No Faculty".to_string())
                        .or_insert(0) += 1;
                }

                let level_name = enhanced_info
                    .admin_level
                    .as_ref()
                    .map(|l| format!("{:?}", l))
                    .unwrap_or_else(|| "Student".to_string());
                *admin_level_breakdown.entry(level_name).or_insert(0) += 1;

                sessions.push(enhanced_info);

                if sessions.len() >= limit {
                    break;
                }
            }
        }
    }

    let total_count = session_state
        .redis_store
        .get_session_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_count = sessions.len();
    Ok(Json(AdminSessionListResponse {
        sessions,
        active_count,
        total_count,
        faculty_breakdown,
        admin_level_breakdown,
    }))
}

// Faculty Admin: Get sessions within their faculty
pub async fn get_faculty_sessions(
    State(session_state): State<SessionState>,
    admin: FacultyAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminSessionListResponse>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    // Faculty admins can only see sessions from their faculty
    let faculty_id = match admin.faculty_id {
        Some(id) => id,
        None => return Err(StatusCode::FORBIDDEN),
    };

    let session_ids = session_state
        .redis_store
        .get_active_sessions(None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sessions = Vec::new();
    let mut admin_level_breakdown = HashMap::new();

    for session_id in &session_ids {
        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            if let Ok(Some(enhanced_info)) =
                get_enhanced_session_info(&session_state, &admin.session_user.session_id, &session)
                    .await
            {
                // Only include sessions from the same faculty
                if enhanced_info.faculty_id == Some(faculty_id) {
                    let level_name = enhanced_info
                        .admin_level
                        .as_ref()
                        .map(|l| format!("{:?}", l))
                        .unwrap_or_else(|| "Student".to_string());
                    *admin_level_breakdown.entry(level_name).or_insert(0) += 1;

                    sessions.push(enhanced_info);

                    if sessions.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    let session_count = sessions.len();

    let mut faculty_breakdown = HashMap::new();
    if let Ok(Some(faculty)) = get_faculty_by_id(&session_state, faculty_id).await {
        faculty_breakdown.insert(faculty.name, session_count);
    }

    let active_count = session_count;
    let total_count = active_count;
    Ok(Json(AdminSessionListResponse {
        sessions,
        active_count,
        total_count,
        faculty_breakdown,
        admin_level_breakdown,
    }))
}

// Get session statistics
pub async fn get_session_stats(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
) -> Result<Json<SessionStatsResponse>, StatusCode> {
    let session_ids = session_state
        .redis_store
        .get_active_sessions(None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut admin_sessions = 0;
    let mut student_sessions = 0;
    let mut sessions_by_level = HashMap::new();
    let mut sessions_by_faculty = HashMap::new();
    let mut sessions_by_device = HashMap::new();
    let mut user_session_counts: HashMap<Uuid, (User, AdminRole, usize, DateTime<Utc>)> =
        HashMap::new();

    for session_id in &session_ids {
        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            // Get user info
            if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                // Get admin role
                let admin_role = get_user_admin_role(&session_state, user.id)
                    .await
                    .ok()
                    .flatten();

                // Count by admin status
                if admin_role.is_some() {
                    admin_sessions += 1;
                } else {
                    student_sessions += 1;
                }

                // Count by admin level
                let level_name = admin_role
                    .as_ref()
                    .map(|r| format!("{:?}", r.admin_level))
                    .unwrap_or_else(|| "Student".to_string());
                *sessions_by_level.entry(level_name).or_insert(0) += 1;

                // Count by faculty
                if let Some(ref role) = admin_role {
                    if let Some(faculty_id) = role.faculty_id {
                        if let Ok(Some(faculty)) =
                            get_faculty_by_id(&session_state, faculty_id).await
                        {
                            *sessions_by_faculty.entry(faculty.name).or_insert(0) += 1;
                        }
                    }
                } else {
                    *sessions_by_faculty
                        .entry("No Faculty".to_string())
                        .or_insert(0) += 1;
                }

                // Count by device type
                let device_type = session
                    .device_info
                    .get("device_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                *sessions_by_device
                    .entry(device_type.to_string())
                    .or_insert(0) += 1;

                // Track user session counts
                let user_id = user.id;
                let entry = user_session_counts.entry(user_id).or_insert((
                    user,
                    admin_role.unwrap_or_else(|| AdminRole {
                        id: Uuid::new_v4(),
                        user_id,
                        admin_level: crate::models::admin_role::AdminLevel::RegularAdmin,
                        faculty_id: None,
                        permissions: vec![],
                        is_enabled: true,
                        created_at: Some(chrono::Utc::now()),
                        updated_at: Some(chrono::Utc::now()),
                    }),
                    0,
                    session.last_accessed,
                ));
                entry.2 += 1;
                if session.last_accessed > entry.3 {
                    entry.3 = session.last_accessed;
                }
            }
        }
    }

    // Get top active users
    let mut user_counts: Vec<_> = user_session_counts.into_iter().collect();
    user_counts.sort_by(|a, b| b.1 .2.cmp(&a.1 .2));
    user_counts.truncate(10);

    let top_active_users = user_counts
        .into_iter()
        .map(
            |(user_id, (user, admin_role, count, last_activity))| TopActiveUser {
                user_id,
                name: format!("{} {}", user.first_name, user.last_name),
                email: user.email,
                session_count: count,
                admin_level: Some(admin_role.admin_level),
                last_activity,
            },
        )
        .collect();

    Ok(Json(SessionStatsResponse {
        total_active_sessions: session_ids.len(),
        admin_sessions,
        student_sessions,
        sessions_by_level,
        sessions_by_faculty,
        sessions_by_device,
        top_active_users,
    }))
}

// Bulk session actions
pub async fn bulk_session_action(
    State(session_state): State<SessionState>,
    admin: AdminUser,
    Json(req): Json<BulkSessionActionRequest>,
) -> Result<Json<BulkSessionActionResponse>, StatusCode> {
    let mut processed_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    match req.action.as_str() {
        "revoke" => {
            let reason = req
                .reason
                .unwrap_or_else(|| format!("Bulk revocation by {}", admin.session_user.email));

            for session_id in &req.session_ids {
                match session_state
                    .redis_store
                    .revoke_session(session_id, Some(reason.clone()))
                    .await
                {
                    Ok(true) => {
                        processed_count += 1;
                        // Log the action
                        let _ = log_session_action(
                            &session_state,
                            "bulk_revoke",
                            session_id,
                            &admin.session_user.user_id,
                            Some(reason.clone()),
                        )
                        .await;
                    }
                    Ok(false) => {
                        failed_count += 1;
                        errors.push(format!("Session {} not found", session_id));
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Failed to revoke session {}: {}", session_id, e));
                    }
                }
            }
        }
        "extend" => {
            let hours = req.extend_hours.unwrap_or(24);
            let new_expiry = Utc::now() + chrono::Duration::hours(hours);

            for session_id in &req.session_ids {
                match session_state
                    .redis_store
                    .extend_session(session_id, new_expiry)
                    .await
                {
                    Ok(true) => {
                        processed_count += 1;
                        // Log the action
                        let _ = log_session_action(
                            &session_state,
                            "bulk_extend",
                            session_id,
                            &admin.session_user.user_id,
                            Some(format!("Extended by {} hours", hours)),
                        )
                        .await;
                    }
                    Ok(false) => {
                        failed_count += 1;
                        errors.push(format!("Session {} not found or expired", session_id));
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Failed to extend session {}: {}", session_id, e));
                    }
                }
            }
        }
        _ => {
            return Ok(Json(BulkSessionActionResponse {
                success: false,
                processed_count: 0,
                failed_count: req.session_ids.len(),
                errors: vec!["Invalid action. Supported actions: revoke, extend".to_string()],
            }));
        }
    }

    Ok(Json(BulkSessionActionResponse {
        success: failed_count == 0,
        processed_count,
        failed_count,
        errors,
    }))
}

// Helper functions
async fn get_enhanced_session_info(
    session_state: &SessionState,
    current_session_id: &str,
    session: &crate::models::session::Session,
) -> Result<Option<EnhancedAdminSessionInfo>, anyhow::Error> {
    // Get user info
    let user = match get_user_by_id(session_state, session.user_id).await? {
        Some(user) => user,
        None => return Ok(None),
    };

    // Get admin role
    let admin_role = get_user_admin_role(session_state, user.id).await?;

    // Get faculty and department info
    let (faculty_name, department_name) = if let Some(department_id) = user.department_id {
        let department = get_department_by_id(session_state, department_id).await?;
        let faculty_name = if let Some(dept) = &department {
            get_faculty_by_id(session_state, dept.faculty_id)
                .await?
                .map(|f| f.name)
        } else {
            None
        };
        (faculty_name, department.map(|d| d.name))
    } else {
        (None, None)
    };

    Ok(Some(EnhancedAdminSessionInfo {
        session_id: session.id.clone(),
        user_id: user.id,
        user_name: format!("{} {}", user.first_name, user.last_name),
        student_id: user.student_id,
        email: user.email,
        admin_level: admin_role.as_ref().map(|r| r.admin_level.clone()),
        faculty_id: admin_role.as_ref().and_then(|r| r.faculty_id),
        faculty_name,
        department_name,
        device_info: session.device_info.clone(),
        ip_address: session.ip_address.clone(),
        user_agent: session.user_agent.clone(),
        created_at: session.created_at,
        last_accessed: session.last_accessed,
        expires_at: session.expires_at,
        is_current_session: session.id == current_session_id,
    }))
}

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

async fn get_faculty_by_id(
    session_state: &SessionState,
    faculty_id: Uuid,
) -> Result<Option<Faculty>, anyhow::Error> {
    let faculty = sqlx::query_as::<_, Faculty>("SELECT * FROM faculties WHERE id = $1")
        .bind(faculty_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    Ok(faculty)
}

async fn get_department_by_id(
    session_state: &SessionState,
    department_id: Uuid,
) -> Result<Option<crate::models::department::Department>, anyhow::Error> {
    let department = sqlx::query_as::<_, crate::models::department::Department>(
        "SELECT * FROM departments WHERE id = $1",
    )
    .bind(department_id)
    .fetch_optional(&session_state.db_pool)
    .await?;

    Ok(department)
}

// Main handlers for routes

/// Get sessions (simplified endpoint for admin_session routes)
pub async fn get_sessions(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
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
            let mut sessions = Vec::new();

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

                        let session_info = serde_json::json!({
                            "session_id": session.id,
                            "user_id": user.id,
                            "user_name": format!("{} {}", user.first_name, user.last_name),
                            "student_id": user.student_id,
                            "email": user.email,
                            "admin_level": admin_role.as_ref().map(|r| r.admin_level.clone()),
                            "device_info": session.device_info,
                            "ip_address": session.ip_address,
                            "user_agent": session.user_agent,
                            "created_at": session.created_at,
                            "last_accessed": session.last_accessed,
                            "expires_at": session.expires_at,
                            "is_active": session.is_active
                        });

                        sessions.push(session_info);

                        if sessions.len() >= limit {
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
                    let error_response = serde_json::json!({
                        "status": "error",
                        "message": "Failed to get session count"
                    });
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                })?;

            let response = serde_json::json!({
                "status": "success",
                "data": {
                    "sessions": sessions,
                    "total_count": total_count,
                    "limit": limit,
                    "filtered_count": sessions.len()
                },
                "message": "Sessions retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Failed to retrieve session information from Redis"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Revoke specific session
pub async fn revoke_session(
    State(session_state): State<SessionState>,
    admin: AdminUser,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if the session exists
    let session_exists = match session_state.redis_store.get_session(&session_id).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Failed to check session existence"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    if !session_exists {
        let error_response = serde_json::json!({
            "status": "error",
            "message": "Session not found"
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // Prevent admin from revoking their own session
    if session_id == admin.session_user.session_id {
        let error_response = serde_json::json!({
            "status": "error",
            "message": "Cannot revoke your own session"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let reason = format!("Revoked by admin: {}", admin.session_user.email);

    match session_state
        .redis_store
        .revoke_session(&session_id, Some(reason.clone()))
        .await
    {
        Ok(true) => {
            // Log the action
            let _ = log_session_action(
                &session_state,
                "revoke",
                &session_id,
                &admin.session_user.user_id,
                Some(reason),
            )
            .await;

            let response = serde_json::json!({
                "status": "success",
                "message": "Session revoked successfully"
            });
            Ok(Json(response))
        }
        Ok(false) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Session not found or already expired"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Failed to revoke session: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Cleanup expired sessions
pub async fn cleanup_expired(
    State(session_state): State<SessionState>,
    admin: SuperAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get all active sessions
    let session_ids_result = session_state.redis_store.get_active_sessions(None).await;

    match session_ids_result {
        Ok(session_ids) => {
            let mut expired_count = 0;
            let mut cleanup_errors = Vec::new();
            let current_time = Utc::now();

            for session_id in &session_ids {
                if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
                    // Check if session has expired
                    if session.expires_at <= current_time {
                        match session_state
                            .redis_store
                            .revoke_session(session_id, Some("Expired session cleanup".to_string()))
                            .await
                        {
                            Ok(true) => {
                                expired_count += 1;
                                // Log the cleanup action
                                let _ = log_session_action(
                                    &session_state,
                                    "cleanup_expired",
                                    session_id,
                                    &admin.session_user.user_id,
                                    Some("Automated cleanup of expired session".to_string()),
                                )
                                .await;
                            }
                            Ok(false) => {
                                cleanup_errors
                                    .push(format!("Session {} was already removed", session_id));
                            }
                            Err(e) => {
                                cleanup_errors.push(format!(
                                    "Failed to cleanup session {}: {}",
                                    session_id, e
                                ));
                            }
                        }
                    }
                }
            }

            // Also cleanup expired sessions from database
            let db_cleanup_result = sqlx::query(
                "UPDATE sessions SET is_active = false WHERE expires_at <= NOW() AND is_active = true"
            )
            .execute(&session_state.db_pool)
            .await;

            let db_affected_rows = match db_cleanup_result {
                Ok(result) => result.rows_affected(),
                Err(e) => {
                    cleanup_errors.push(format!("Database cleanup failed: {}", e));
                    0
                }
            };

            let response = serde_json::json!({
                "status": "success",
                "data": {
                    "redis_sessions_cleaned": expired_count,
                    "database_sessions_cleaned": db_affected_rows,
                    "total_sessions_checked": session_ids.len(),
                    "errors": cleanup_errors
                },
                "message": format!(
                    "Cleanup completed. Removed {} expired sessions from Redis and {} from database",
                    expired_count,
                    db_affected_rows
                )
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Failed to retrieve sessions for cleanup"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get active admin sessions only (filtered for admin users)
pub async fn get_active_admin_sessions(
    State(session_state): State<SessionState>,
    admin: AdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    // Get active session IDs from Redis
    let session_ids_result = session_state.redis_store.get_active_sessions(None).await;

    match session_ids_result {
        Ok(session_ids) => {
            let mut admin_sessions = Vec::new();
            let mut admin_level_breakdown = HashMap::new();
            let mut faculty_breakdown = HashMap::new();

            for session_id in &session_ids {
                if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
                    // Get user info
                    if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                        // Only include sessions with admin roles
                        if let Ok(Some(admin_role)) =
                            get_user_admin_role(&session_state, user.id).await
                        {
                            // Check if current admin can view this session
                            let can_view = match admin.admin_role.admin_level {
                                crate::models::admin_role::AdminLevel::SuperAdmin => true,
                                crate::models::admin_role::AdminLevel::FacultyAdmin => {
                                    // Faculty admins can only see sessions from their faculty or lower levels
                                    admin_role.faculty_id == admin.admin_role.faculty_id
                                        || matches!(
                                            admin_role.admin_level,
                                            crate::models::admin_role::AdminLevel::RegularAdmin
                                        )
                                }
                                crate::models::admin_role::AdminLevel::RegularAdmin => false, // Regular admins can't view other sessions
                            };

                            if !can_view {
                                continue;
                            }

                            // Get faculty info
                            let faculty_name = if let Some(faculty_id) = admin_role.faculty_id {
                                get_faculty_by_id(&session_state, faculty_id)
                                    .await
                                    .ok()
                                    .flatten()
                                    .map(|f| f.name)
                            } else {
                                None
                            };

                            let session_info = serde_json::json!({
                                "session_id": session.id,
                                "user_id": user.id,
                                "user_name": format!("{} {}", user.first_name, user.last_name),
                                "student_id": user.student_id,
                                "email": user.email,
                                "admin_level": admin_role.admin_level,
                                "faculty_id": admin_role.faculty_id,
                                "faculty_name": faculty_name,
                                "device_info": session.device_info,
                                "ip_address": session.ip_address,
                                "user_agent": session.user_agent,
                                "created_at": session.created_at,
                                "last_accessed": session.last_accessed,
                                "expires_at": session.expires_at,
                                "is_current_session": session.id == admin.session_user.session_id,
                                "is_active": session.is_active
                            });

                            admin_sessions.push(session_info);

                            // Update breakdowns
                            let level_name = format!("{:?}", admin_role.admin_level);
                            *admin_level_breakdown.entry(level_name).or_insert(0) += 1;

                            let faculty_key =
                                faculty_name.unwrap_or_else(|| "No Faculty".to_string());
                            *faculty_breakdown.entry(faculty_key).or_insert(0) += 1;

                            if admin_sessions.len() >= limit {
                                break;
                            }
                        }
                    }
                }
            }

            let response = serde_json::json!({
                "status": "success",
                "data": {
                    "admin_sessions": admin_sessions,
                    "total_admin_sessions": admin_sessions.len(),
                    "admin_level_breakdown": admin_level_breakdown,
                    "faculty_breakdown": faculty_breakdown,
                    "limit": limit
                },
                "message": "Active admin sessions retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Failed to retrieve session information from Redis"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn log_session_action(
    _session_state: &SessionState,
    action: &str,
    session_id: &str,
    admin_id: &Uuid,
    reason: Option<String>,
) -> Result<(), anyhow::Error> {
    // In a real implementation, you might want to store this in a separate audit log table
    // For now, we'll just log it to the application logs
    tracing::info!(
        action = action,
        session_id = session_id,
        admin_id = %admin_id,
        reason = reason.as_deref().unwrap_or(""),
        "Admin session action performed"
    );

    Ok(())
}
