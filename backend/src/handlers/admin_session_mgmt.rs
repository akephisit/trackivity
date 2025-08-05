use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::middleware::session::{AdminUser, FacultyAdminUser, SessionState, SuperAdminUser};
use crate::models::{
    admin_role::AdminLevel,
    faculty::Faculty,
    session::{
        AdminSessionInfo, AdminSessionMonitor, BatchSessionRevocationRequest,
        BatchSessionRevocationResponse, ForceLogoutFacultyRequest, ForceLogoutUserRequest,
        Permission, SessionAnalytics, SessionRevocationRequest, SessionType, SuspiciousActivity,
    },
    user::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSessionResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
    pub total_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMonitorResponse {
    pub success: bool,
    pub monitor_data: AdminSessionMonitor,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAnalyticsResponse {
    pub success: bool,
    pub analytics: SessionAnalytics,
    pub message: String,
}

// Get all active admin sessions (Super Admin only)
pub async fn get_active_admin_sessions(
    State(session_state): State<SessionState>,
    _super_admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminSessionResponse>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let admin_level_filter = params
        .get("admin_level")
        .and_then(|level| match level.as_str() {
            "SuperAdmin" => Some(AdminLevel::SuperAdmin),
            "FacultyAdmin" => Some(AdminLevel::FacultyAdmin),
            "RegularAdmin" => Some(AdminLevel::RegularAdmin),
            _ => None,
        });

    let admin_session_ids = session_state
        .redis_store
        .get_admin_sessions(admin_level_filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sessions = Vec::new();
    let mut processed = 0;

    for session_id in &admin_session_ids {
        if processed >= limit {
            break;
        }

        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            // Get user info
            if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                // Get faculty name if applicable
                let faculty_name = if let Some(faculty_id) = session.faculty_id {
                    get_faculty_name(&session_state, faculty_id)
                        .await
                        .unwrap_or_default()
                } else {
                    None
                };

                let session_info = AdminSessionInfo {
                    session_id: session.id,
                    user_id: user.id,
                    user_name: format!("{} {}", user.first_name, user.last_name),
                    student_id: user.student_id,
                    email: user.email,
                    admin_level: session.admin_level.clone(),
                    faculty_name,
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

                sessions.push(session_info);
                processed += 1;
            }
        }
    }

    let total_count = session_state
        .redis_store
        .get_session_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminSessionResponse {
        success: true,
        data: Some(serde_json::json!({
            "sessions": sessions,
            "admin_session_count": admin_session_ids.len(),
        })),
        message: "Admin sessions retrieved successfully".to_string(),
        total_count: Some(total_count),
    }))
}

// Get session monitor dashboard (Super Admin only)
pub async fn get_session_monitor(
    State(session_state): State<SessionState>,
    _super_admin: SuperAdminUser,
) -> Result<Json<SessionMonitorResponse>, StatusCode> {
    // Get session counts by admin level
    let super_admin_count = session_state
        .redis_store
        .get_admin_session_count_by_level(&AdminLevel::SuperAdmin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let faculty_admin_count = session_state
        .redis_store
        .get_admin_session_count_by_level(&AdminLevel::FacultyAdmin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let regular_admin_count = session_state
        .redis_store
        .get_admin_session_count_by_level(&AdminLevel::RegularAdmin)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_sessions = session_state
        .redis_store
        .get_session_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_admin_sessions = super_admin_count + faculty_admin_count + regular_admin_count;
    let active_student_sessions = total_sessions.saturating_sub(active_admin_sessions);

    // Get recent admin logins (last 10)
    let admin_session_ids = session_state
        .redis_store
        .get_admin_sessions(None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut recent_logins = Vec::new();
    let mut processed = 0;

    for session_id in admin_session_ids.iter().take(10) {
        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                let faculty_name = if let Some(faculty_id) = session.faculty_id {
                    get_faculty_name(&session_state, faculty_id)
                        .await
                        .unwrap_or_default()
                } else {
                    None
                };

                let session_info = AdminSessionInfo {
                    session_id: session.id,
                    user_id: user.id,
                    user_name: format!("{} {}", user.first_name, user.last_name),
                    student_id: user.student_id,
                    email: user.email,
                    admin_level: session.admin_level.clone(),
                    faculty_name,
                    department_name: None,
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

                recent_logins.push(session_info);
                processed += 1;
            }
        }
    }

    let mut sessions_by_level = HashMap::new();
    sessions_by_level.insert("SuperAdmin".to_string(), super_admin_count);
    sessions_by_level.insert("FacultyAdmin".to_string(), faculty_admin_count);
    sessions_by_level.insert("RegularAdmin".to_string(), regular_admin_count);

    let monitor_data = AdminSessionMonitor {
        total_sessions,
        active_admin_sessions,
        active_student_sessions,
        sessions_by_level,
        sessions_by_faculty: HashMap::new(), // TODO: Implement faculty grouping
        recent_logins,
        suspicious_activities: Vec::new(), // TODO: Implement suspicious activity detection
    };

    Ok(Json(SessionMonitorResponse {
        success: true,
        monitor_data,
        message: "Session monitor data retrieved successfully".to_string(),
    }))
}

// Force logout specific session (Admin only)
pub async fn force_logout_session(
    State(session_state): State<SessionState>,
    admin_user: AdminUser,
    Path(session_id): Path<String>,
    Json(req): Json<SessionRevocationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if admin has permission to revoke this session
    let target_session = session_state
        .redis_store
        .get_session(&session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(target_session) = target_session {
        // Super admin can revoke any session
        if admin_user.admin_role.admin_level == AdminLevel::SuperAdmin {
            // Allow
        }
        // Faculty admin can only revoke sessions within their faculty
        else if admin_user.admin_role.admin_level == AdminLevel::FacultyAdmin {
            if target_session.faculty_id != admin_user.admin_role.faculty_id {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        // Regular admin cannot revoke other sessions
        else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let reason = req
        .reason
        .unwrap_or_else(|| "Force logout by admin".to_string());

    let success = session_state
        .redis_store
        .revoke_session_by_admin(
            &session_id,
            Some(reason.clone()),
            Some(admin_user.session_user.user_id),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if success {
        // Send SSE notification to the affected user if connected
        if let Some(sse_manager) = &session_state.sse_manager {
            let notification = crate::handlers::sse::SessionUpdateMessage {
                session_id: session_id.clone(),
                action: "force_logout".to_string(),
                reason: Some(reason),
                new_expires_at: None,
            };

            let sse_message = crate::handlers::sse::SseMessage {
                event_type: "session_update".to_string(),
                data: serde_json::to_value(notification).unwrap(),
                timestamp: Utc::now(),
                target_permissions: None,
                target_user_id: None,
                target_faculty_id: None,
            };

            let _ = sse_manager.send_to_session(&session_id, sse_message).await;
        }

        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Session revoked successfully"
        })))
    } else {
        Ok(Json(serde_json::json!({
            "success": false,
            "message": "Session not found"
        })))
    }
}

// Batch force logout sessions (Super Admin only)
pub async fn batch_force_logout_sessions(
    State(session_state): State<SessionState>,
    _super_admin: SuperAdminUser,
    Json(req): Json<BatchSessionRevocationRequest>,
) -> Result<Json<BatchSessionRevocationResponse>, StatusCode> {
    let response = session_state
        .redis_store
        .batch_revoke_sessions(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Send SSE notifications for successfully revoked sessions
    if let Some(sse_manager) = &session_state.sse_manager {
        for session_id in &response.revoked_sessions {
            let notification = crate::handlers::sse::SessionUpdateMessage {
                session_id: session_id.clone(),
                action: "force_logout".to_string(),
                reason: response.errors.first().cloned(),
                new_expires_at: None,
            };

            let sse_message = crate::handlers::sse::SseMessage {
                event_type: "session_update".to_string(),
                data: serde_json::to_value(notification).unwrap(),
                timestamp: Utc::now(),
                target_permissions: None,
                target_user_id: None,
                target_faculty_id: None,
            };

            let _ = sse_manager.send_to_session(session_id, sse_message).await;
        }
    }

    Ok(Json(response))
}

// Force logout all sessions for a user (Admin only)
pub async fn force_logout_user_sessions(
    State(session_state): State<SessionState>,
    admin_user: AdminUser,
    Path(user_id): Path<Uuid>,
    Json(mut req): Json<ForceLogoutUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Override the user_id from path
    req.user_id = user_id;

    // Check permission: Faculty admin can only logout users in their faculty
    if admin_user.admin_role.admin_level != AdminLevel::SuperAdmin {
        // TODO: Check if target user belongs to admin's faculty
        // For now, allow faculty admin to logout users in their faculty
    }

    let revoked_sessions = session_state
        .redis_store
        .force_logout_user(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Revoked {} sessions", revoked_sessions.len()),
        "revoked_sessions": revoked_sessions
    })))
}

// Force logout all sessions for a faculty (Faculty Admin or Super Admin)
pub async fn force_logout_faculty_sessions(
    State(session_state): State<SessionState>,
    admin_user: FacultyAdminUser,
    Path(faculty_id): Path<Uuid>,
    Json(mut req): Json<ForceLogoutFacultyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Override the faculty_id from path
    req.faculty_id = faculty_id;

    // Check permission: Faculty admin can only logout sessions in their own faculty
    if admin_user.admin_role.admin_level != AdminLevel::SuperAdmin {
        if admin_user.faculty_id != Some(faculty_id) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let revoked_sessions = session_state
        .redis_store
        .force_logout_faculty(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Revoked {} faculty sessions", revoked_sessions.len()),
        "revoked_sessions": revoked_sessions
    })))
}

// Get session analytics (Super Admin only)
pub async fn get_session_analytics(
    State(session_state): State<SessionState>,
    _super_admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SessionAnalyticsResponse>, StatusCode> {
    let _days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(7);

    // TODO: Implement comprehensive analytics
    // For now, return basic session counts
    let current_active_sessions = session_state
        .redis_store
        .get_session_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = SessionAnalytics {
        current_active_sessions,
        today_logins: 0,                          // TODO: Track daily logins
        today_admin_logins: 0,                    // TODO: Track daily admin logins
        failed_login_attempts: 0,                 // TODO: Track failed login attempts
        average_session_duration: 0.0,            // TODO: Calculate average duration
        most_active_hours: Vec::new(),            // TODO: Track hourly activity
        device_distribution: HashMap::new(),      // TODO: Analyze device types
        browser_distribution: HashMap::new(),     // TODO: Analyze browsers
        top_active_users: Vec::new(),             // TODO: Find most active users
        recent_suspicious_activities: Vec::new(), // TODO: Detect suspicious activities
    };

    Ok(Json(SessionAnalyticsResponse {
        success: true,
        analytics,
        message: "Session analytics retrieved successfully".to_string(),
    }))
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

async fn get_faculty_name(
    session_state: &SessionState,
    faculty_id: Uuid,
) -> Result<Option<String>, anyhow::Error> {
    let faculty = sqlx::query_as::<_, Faculty>("SELECT * FROM faculties WHERE id = $1")
        .bind(faculty_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    Ok(faculty.map(|f| f.name))
}
