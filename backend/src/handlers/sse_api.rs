use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::handlers::sse_enhanced::*;
use crate::middleware::session::SessionState;
use crate::models::session::{Permission, SessionUser};

// Enhanced notification API endpoints

// Send notification to user
pub async fn send_notification(
    State(session_state): State<SessionState>,
    session_user: SessionUser,
    Json(notification): Json<NotificationMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let message = SseMessage {
        event_type: SseEventType::Custom("notification".to_string()),
        data: serde_json::to_value(&notification)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("notif_{}", uuid::Uuid::new_v4()),
        target_permissions: None,
        target_user_id: Some(session_user.user_id),
        target_faculty_id: None,
        target_sessions: None,
        priority: match notification.notification_type {
            NotificationType::Error => MessagePriority::High,
            NotificationType::Warning => MessagePriority::Normal,
            _ => MessagePriority::Normal,
        },
        ttl_seconds: if let Some(expires_at) = notification.expires_at {
            Some((expires_at - Utc::now()).num_seconds() as u32)
        } else {
            Some(3600) // 1 hour default
        },
        retry_count: 0,
        broadcast_id: None,
    };

    match sse_manager
        .send_to_session(&session_user.session_id, message)
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Notification sent successfully"
        }))),
        Err(SseError::SessionNotFound) => {
            tracing::warn!("Session {} not found for notification", session_user.session_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(SseError::ConnectionClosed) => {
            tracing::warn!("Connection closed for session {}", session_user.session_id);
            Err(StatusCode::GONE)
        }
        Err(e) => {
            tracing::error!("Failed to send notification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Send activity check-in notification
pub async fn send_activity_checked_in(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
    Json(payload): Json<ActivityCheckedInMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = SseMessage {
        event_type: SseEventType::ActivityCheckedIn,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("checkin_{}", uuid::Uuid::new_v4()),
        target_permissions: Some(vec![
            "ViewAssignedActivities".to_string(),
            "ManageFacultyActivities".to_string(),
        ]),
        target_user_id: None,
        target_faculty_id: None, // Will be filtered by permissions
        target_sessions: None,
        priority: MessagePriority::Normal,
        ttl_seconds: Some(3600),
        retry_count: 0,
        broadcast_id: Some(format!("activity_checkin_{}", payload.activity_id)),
    };

    match sse_manager.broadcast(message).await {
        Ok(sent_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Activity check-in notification sent",
            "recipients": sent_count
        }))),
        Err(e) => {
            tracing::error!("Failed to send activity check-in notification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Send new activity notification
pub async fn send_new_activity_created(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
    Json(payload): Json<NewActivityMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = SseMessage {
        event_type: SseEventType::NewActivityCreated,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("new_activity_{}", uuid::Uuid::new_v4()),
        target_permissions: None, // Send to all users
        target_user_id: None,
        target_faculty_id: payload.faculty_id,
        target_sessions: None,
        priority: MessagePriority::Normal,
        ttl_seconds: Some(86400), // 24 hours
        retry_count: 0,
        broadcast_id: Some(format!("new_activity_{}", payload.activity_id)),
    };

    match sse_manager.broadcast(message).await {
        Ok(sent_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "New activity notification sent",
            "recipients": sent_count
        }))),
        Err(e) => {
            tracing::error!("Failed to send new activity notification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Send subscription expiry warning
pub async fn send_subscription_expiry_warning(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
    Json(payload): Json<SubscriptionExpiryMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = SseMessage {
        event_type: SseEventType::SubscriptionExpiryWarning,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("sub_expiry_{}", uuid::Uuid::new_v4()),
        target_permissions: None,
        target_user_id: Some(payload.user_id),
        target_faculty_id: None,
        target_sessions: None,
        priority: MessagePriority::High,
        ttl_seconds: Some(604800), // 7 days
        retry_count: 0,
        broadcast_id: None,
    };

    let user_sessions = sse_manager.get_user_connections(&payload.user_id).await;
    let mut sent_count = 0u32;

    for session_id in user_sessions {
        if sse_manager.send_to_session(&session_id, message.clone()).await.is_ok() {
            sent_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Subscription expiry warning sent",
        "recipients": sent_count
    })))
}

// Send system announcement
pub async fn send_system_announcement(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
    Json(payload): Json<SystemAnnouncementMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let target_permissions = if payload.target_audience.is_empty() {
        None
    } else {
        Some(payload.target_audience.clone())
    };

    let message = SseMessage {
        event_type: SseEventType::SystemAnnouncement,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("announcement_{}", uuid::Uuid::new_v4()),
        target_permissions,
        target_user_id: None,
        target_faculty_id: None,
        target_sessions: None,
        priority: match payload.severity {
            AnnouncementSeverity::Critical => MessagePriority::Critical,
            AnnouncementSeverity::Important => MessagePriority::High,
            _ => MessagePriority::Normal,
        },
        ttl_seconds: if let Some(display_until) = payload.display_until {
            Some((display_until - Utc::now()).num_seconds() as u32)
        } else {
            Some(86400) // 24 hours default
        },
        retry_count: 0,
        broadcast_id: Some(format!("announcement_{}", payload.announcement_id)),
    };

    match sse_manager.broadcast(message).await {
        Ok(sent_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "System announcement sent",
            "recipients": sent_count
        }))),
        Err(e) => {
            tracing::error!("Failed to send system announcement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Enhanced admin notification endpoints

// Admin: Send notification to all users with specific permission
pub async fn admin_send_notification_by_permission(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser, // Must be admin
    Path(permission_str): Path<String>,
    Json(notification): Json<NotificationMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        
    // Parse permission string to Permission enum
    let permission = match permission_str.as_str() {
        "ViewSystemReports" => Permission::ViewSystemReports,
        "ManageAllFaculties" => Permission::ManageAllFaculties,
        "ManageFacultyStudents" => Permission::ManageFacultyStudents,
        "ManageFacultyActivities" => Permission::ManageFacultyActivities,
        "ScanQrCodes" => Permission::ScanQrCodes,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let message = SseMessage {
        event_type: SseEventType::Custom("notification".to_string()),
        data: serde_json::to_value(&notification)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("perm_notif_{}", uuid::Uuid::new_v4()),
        target_permissions: Some(vec![format!("{:?}", permission)]),
        target_user_id: None,
        target_faculty_id: None,
        target_sessions: None,
        priority: match notification.notification_type {
            NotificationType::Error => MessagePriority::High,
            NotificationType::Warning => MessagePriority::Normal,
            _ => MessagePriority::Normal,
        },
        ttl_seconds: if let Some(expires_at) = notification.expires_at {
            Some((expires_at - Utc::now()).num_seconds() as u32)
        } else {
            Some(3600)
        },
        retry_count: 0,
        broadcast_id: Some(format!("perm_notif_{}", permission_str)),
    };

    match sse_manager.send_to_permission(&permission, message).await {
        Ok(sent_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Notification sent to users with permission",
            "permission": permission_str,
            "recipients": sent_count
        }))),
        Err(e) => {
            tracing::error!("Failed to send permission-based notification: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Enhanced admin control endpoints

// Admin: Force logout notification
pub async fn admin_send_force_logout(
    State(session_state): State<SessionState>,
    admin_user: SessionUser,
    Path(target_session_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        
    // Extract reason from payload
    let reason_str = payload
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("Administrative action");
    
    let force_all_devices = payload
        .get("force_all_devices")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Get target user ID from session
    let target_user_id = if let Ok(Some(session)) = session_state.redis_store.get_session(&target_session_id).await {
        session.user_id
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    if force_all_devices {
        // Force disconnect all user sessions
        match sse_manager
            .force_disconnect_user(
                &target_user_id,
                SessionRevocationReason::AdminAction,
                reason_str,
                Some(admin_user.user_id),
            )
            .await
        {
            Ok(disconnected_count) => Ok(Json(serde_json::json!({
                "success": true,
                "message": "Force logout sent to all user sessions",
                "sessions_disconnected": disconnected_count
            }))),
            Err(e) => {
                tracing::error!("Failed to force logout user sessions: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Force logout specific session
        let logout_message = SessionRevokedMessage {
            session_id: target_session_id.clone(),
            user_id: target_user_id,
            reason: SessionRevocationReason::AdminAction,
            message: reason_str.to_string(),
            revoked_by: Some(admin_user.user_id),
            force_logout_all_devices: false,
        };

        let message = SseMessage {
            event_type: SseEventType::SessionRevoked,
            data: serde_json::to_value(&logout_message)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            timestamp: Utc::now(),
            message_id: format!("force_logout_{}", uuid::Uuid::new_v4()),
            target_permissions: None,
            target_user_id: Some(target_user_id),
            target_faculty_id: None,
            target_sessions: Some(vec![target_session_id.clone()]),
            priority: MessagePriority::Critical,
            ttl_seconds: None,
            retry_count: 0,
            broadcast_id: None,
        };

        match sse_manager.send_to_session(&target_session_id, message).await {
            Ok(()) => {
                // Also remove the connection
                sse_manager.remove_connection(&target_session_id).await.ok();
                
                Ok(Json(serde_json::json!({
                    "success": true,
                    "message": "Force logout notification sent",
                    "session_id": target_session_id
                })))
            }
            Err(SseError::SessionNotFound) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                tracing::error!("Failed to send force logout notification: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

// Admin: Send permission update notification
pub async fn admin_send_permission_updated(
    State(session_state): State<SessionState>,
    admin_user: SessionUser,
    Json(payload): Json<PermissionUpdatedMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = SseMessage {
        event_type: SseEventType::PermissionUpdated,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("perm_update_{}", uuid::Uuid::new_v4()),
        target_permissions: None,
        target_user_id: Some(payload.user_id),
        target_faculty_id: None,
        target_sessions: None,
        priority: MessagePriority::High,
        ttl_seconds: Some(3600),
        retry_count: 0,
        broadcast_id: None,
    };

    let user_sessions = sse_manager.get_user_connections(&payload.user_id).await;
    let mut sent_count = 0u32;

    for session_id in user_sessions {
        if sse_manager.send_to_session(&session_id, message.clone()).await.is_ok() {
            sent_count += 1;
        }
    }

    // If requires re-login, force disconnect after sending notification
    if payload.requires_re_login {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; // Give time for notification
        sse_manager
            .force_disconnect_user(
                &payload.user_id,
                SessionRevocationReason::PermissionChange,
                "Your permissions have been updated. Please log in again.",
                Some(admin_user.user_id),
            )
            .await
            .ok();
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Permission update notification sent",
        "recipients": sent_count,
        "requires_re_login": payload.requires_re_login
    })))
}

// Admin: Send admin promotion notification
pub async fn admin_send_promotion_notification(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
    Json(payload): Json<AdminPromotedMessage>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = SseMessage {
        event_type: SseEventType::AdminPromoted,
        data: serde_json::to_value(&payload)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("promotion_{}", uuid::Uuid::new_v4()),
        target_permissions: None,
        target_user_id: Some(payload.user_id),
        target_faculty_id: None,
        target_sessions: None,
        priority: MessagePriority::High,
        ttl_seconds: Some(86400), // 24 hours
        retry_count: 0,
        broadcast_id: None,
    };

    let user_sessions = sse_manager.get_user_connections(&payload.user_id).await;
    let mut sent_count = 0u32;

    for session_id in user_sessions {
        if sse_manager.send_to_session(&session_id, message.clone()).await.is_ok() {
            sent_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Admin promotion notification sent",
        "recipients": sent_count
    })))
}

// Get SSE connection statistics (Admin only)
pub async fn admin_get_sse_stats(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
) -> Result<Json<ConnectionStats>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = sse_manager.get_connection_stats().await;
    Ok(Json(stats))
}

// Force cleanup inactive connections (Admin only)
pub async fn admin_cleanup_connections(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    match sse_manager.cleanup_inactive_connections(&session_state).await {
        Ok(cleaned_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Connection cleanup completed",
            "cleaned_connections": cleaned_count
        }))),
        Err(e) => {
            tracing::error!("Failed to cleanup connections: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Send manual heartbeat (Admin only)
pub async fn admin_send_heartbeat(
    State(session_state): State<SessionState>,
    _admin_user: SessionUser,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    match sse_manager.send_heartbeat().await {
        Ok(sent_count) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Heartbeat sent to all connections",
            "recipients": sent_count
        }))),
        Err(e) => {
            tracing::error!("Failed to send heartbeat: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Send test notification (Admin only)
pub async fn admin_send_test_notification(
    State(session_state): State<SessionState>,
    admin_user: SessionUser,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let title = payload.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Test Notification");
    
    let message_text = payload.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("This is a test notification from the SSE system.");

    let notification_type = match payload.get("type").and_then(|v| v.as_str()) {
        Some("error") => NotificationType::Error,
        Some("warning") => NotificationType::Warning,
        Some("success") => NotificationType::Success,
        _ => NotificationType::Info,
    };

    let target_user = payload.get("target_user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Uuid>().ok());

    let notification = NotificationMessage {
        title: title.to_string(),
        message: message_text.to_string(),
        notification_type,
        action_url: None,
        expires_at: Some(Utc::now() + chrono::Duration::minutes(30)),
        read_receipt_required: false,
        sound_enabled: true,
        category: NotificationCategory::System,
    };

    let sse_message = SseMessage {
        event_type: SseEventType::Custom("test_notification".to_string()),
        data: serde_json::to_value(&notification)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        timestamp: Utc::now(),
        message_id: format!("test_{}", uuid::Uuid::new_v4()),
        target_permissions: None,
        target_user_id: target_user.or(Some(admin_user.user_id)), // Send to self if no target
        target_faculty_id: None,
        target_sessions: None,
        priority: MessagePriority::Normal,
        ttl_seconds: Some(1800), // 30 minutes
        retry_count: 0,
        broadcast_id: None,
    };

    if let Some(user_id) = target_user {
        // Send to specific user
        match sse_manager.send_to_user(user_id, sse_message).await {
            Ok(sent_count) => Ok(Json(serde_json::json!({
                "success": true,
                "message": "Test notification sent to specific user",
                "target_user": user_id,
                "recipients": sent_count
            }))),
            Err(e) => {
                tracing::error!("Failed to send test notification: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Send to self
        match sse_manager.send_to_session(&admin_user.session_id, sse_message).await {
            Ok(()) => Ok(Json(serde_json::json!({
                "success": true,
                "message": "Test notification sent to current session",
                "recipients": 1
            }))),
            Err(e) => {
                tracing::error!("Failed to send test notification: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}