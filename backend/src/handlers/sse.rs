use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::Sse,
    response::sse::{Event, KeepAlive},
};
use chrono::Utc;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt as _;
use uuid::Uuid;

use crate::models::session::{SessionUser, Permission};
use crate::middleware::session::SessionState;

// SSE Connection Manager
#[derive(Clone)]
pub struct SseConnectionManager {
    // Map of session_id -> broadcast sender
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<SseMessage>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseMessage {
    pub event_type: String,
    pub data: Value,
    pub timestamp: chrono::DateTime<Utc>,
    pub target_permissions: Option<Vec<String>>, // Filter by permissions
    pub target_user_id: Option<Uuid>,           // Target specific user
    pub target_faculty_id: Option<Uuid>,        // Target faculty members
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub title: String,
    pub message: String,
    pub notification_type: String, // "info", "warning", "error", "success"
    pub action_url: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUpdateMessage {
    pub session_id: String,
    pub action: String, // "force_logout", "permission_changed", "extended"
    pub reason: Option<String>,
    pub new_expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityUpdateMessage {
    pub activity_id: Uuid,
    pub title: String,
    pub update_type: String, // "created", "updated", "cancelled", "started"
    pub message: String,
}

impl SseConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Add new SSE connection
    pub async fn add_connection(&self, session_id: String) -> broadcast::Receiver<SseMessage> {
        let mut connections = self.connections.write().await;
        
        // Remove old connection if exists
        if connections.contains_key(&session_id) {
            connections.remove(&session_id);
        }
        
        // Create new broadcast channel
        let (tx, rx) = broadcast::channel(100);
        connections.insert(session_id.clone(), tx);
        
        tracing::info!("Added SSE connection for session: {}", session_id);
        rx
    }

    // Remove SSE connection
    pub async fn remove_connection(&self, session_id: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(session_id).is_some() {
            tracing::info!("Removed SSE connection for session: {}", session_id);
        }
    }

    // Send message to specific session
    pub async fn send_to_session(&self, session_id: &str, message: SseMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        
        if let Some(tx) = connections.get(session_id) {
            if let Err(_) = tx.send(message) {
                return Err("Failed to send message - connection closed".to_string());
            }
        } else {
            return Err("Session not connected".to_string());
        }
        
        Ok(())
    }

    // Broadcast message to all connections
    pub async fn broadcast(&self, message: SseMessage) {
        let connections = self.connections.read().await;
        
        for (session_id, tx) in connections.iter() {
            if let Err(_) = tx.send(message.clone()) {
                tracing::warn!("Failed to send broadcast message to session: {}", session_id);
            }
        }
    }

    // Send message to users with specific permission
    pub async fn send_to_permission(
        &self, 
        session_state: &SessionState,
        permission: &Permission,
        message: SseMessage
    ) {
        let connections = self.connections.read().await;
        
        for (session_id, tx) in connections.iter() {
            // Check if session user has the required permission
            if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
                // Get user permissions (this would need to be optimized in production)
                if let Ok(Some(user)) = get_user_by_id(session_state, session.user_id).await {
                    if let Ok(Some(admin_role)) = get_user_admin_role(session_state, user.id).await {
                        let user_permissions = Permission::from_admin_level(&admin_role.admin_level, admin_role.faculty_id);
                        
                        if user_permissions.contains(permission) {
                            let _ = tx.send(message.clone());
                        }
                    }
                }
            }
        }
    }

    // Send message to faculty members
    pub async fn send_to_faculty(
        &self,
        session_state: &SessionState,
        faculty_id: Uuid,
        message: SseMessage
    ) {
        let connections = self.connections.read().await;
        
        for (session_id, tx) in connections.iter() {
            if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
                if let Ok(Some(admin_role)) = get_user_admin_role(session_state, session.user_id).await {
                    if admin_role.faculty_id == Some(faculty_id) {
                        let _ = tx.send(message.clone());
                    }
                }
            }
        }
    }

    // Get active connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    // Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self, session_state: &SessionState) {
        let mut connections = self.connections.write().await;
        let mut to_remove = Vec::new();
        
        for session_id in connections.keys() {
            // Check if session still exists in Redis
            if let Ok(None) = session_state.redis_store.get_session(session_id).await {
                to_remove.push(session_id.clone());
            }
        }
        
        for session_id in to_remove {
            connections.remove(&session_id);
            tracing::info!("Cleaned up inactive SSE connection: {}", session_id);
        }
    }
}

// SSE endpoint handler
pub async fn sse_handler(
    State(session_state): State<SessionState>,
    State(sse_manager): State<SseConnectionManager>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    session_user: SessionUser, // This ensures authentication
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    // Verify that the session_id matches the authenticated user's session
    if session_user.session_id != session_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Add connection to manager
    let rx = sse_manager.add_connection(session_id.clone()).await;
    
    // Create the stream
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .map(|result| {
            match result {
                Ok(message) => {
                    let event_data = serde_json::to_string(&message)
                        .unwrap_or_else(|_| "{}".to_string());
                    
                    Ok(Event::default()
                        .event(&message.event_type)
                        .data(event_data))
                }
                Err(_) => {
                    // Channel closed or lagged
                    Ok(Event::default()
                        .event("error")
                        .data("Connection error"))
                }
            }
        });

    // Set up keep-alive and return SSE response
    Ok(Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text")
        ))
}

// Send notification to user
pub async fn send_notification(
    State(sse_manager): State<SseConnectionManager>,
    session_user: SessionUser,
    notification: NotificationMessage,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let message = SseMessage {
        event_type: "notification".to_string(),
        data: serde_json::to_value(notification).unwrap(),
        timestamp: Utc::now(),
        target_permissions: None,
        target_user_id: Some(session_user.user_id),
        target_faculty_id: None,
    };

    if let Err(e) = sse_manager.send_to_session(&session_user.session_id, message).await {
        tracing::error!("Failed to send notification: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::Json(serde_json::json!({
        "success": true,
        "message": "Notification sent"
    })))
}

// Admin: Send notification to all users with specific permission
pub async fn admin_send_notification_by_permission(
    State(session_state): State<SessionState>,
    State(sse_manager): State<SseConnectionManager>,
    _admin_user: SessionUser, // Must be admin
    Path(permission_str): Path<String>,
    axum::Json(notification): axum::Json<NotificationMessage>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
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
        event_type: "notification".to_string(),
        data: serde_json::to_value(notification).unwrap(),
        timestamp: Utc::now(),
        target_permissions: Some(vec![format!("{:?}", permission)]),
        target_user_id: None,
        target_faculty_id: None,
    };

    sse_manager.send_to_permission(&session_state, &permission, message).await;

    Ok(axum::Json(serde_json::json!({
        "success": true,
        "message": "Notification sent to users with permission"
    })))
}

// Admin: Force logout notification
pub async fn admin_send_force_logout(
    State(sse_manager): State<SseConnectionManager>,
    _admin_user: SessionUser,
    Path(target_session_id): Path<String>,
    axum::Json(reason): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let logout_message = SessionUpdateMessage {
        session_id: target_session_id.clone(),
        action: "force_logout".to_string(),
        reason: reason.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()),
        new_expires_at: None,
    };

    let message = SseMessage {
        event_type: "session_update".to_string(),
        data: serde_json::to_value(logout_message).unwrap(),
        timestamp: Utc::now(),
        target_permissions: None,
        target_user_id: None,
        target_faculty_id: None,
    };

    if let Err(e) = sse_manager.send_to_session(&target_session_id, message).await {
        tracing::error!("Failed to send force logout notification: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::Json(serde_json::json!({
        "success": true,
        "message": "Force logout notification sent"
    })))
}

// Send activity update notification
pub async fn send_activity_update(
    State(session_state): State<SessionState>,
    State(sse_manager): State<SseConnectionManager>,
    session_user: SessionUser,
    activity_update: ActivityUpdateMessage,
) -> Result<(), String> {
    let message = SseMessage {
        event_type: "activity_update".to_string(),
        data: serde_json::to_value(activity_update).unwrap(),
        timestamp: Utc::now(),
        target_permissions: Some(vec!["ViewAssignedActivities".to_string()]),
        target_user_id: None,
        target_faculty_id: session_user.faculty_id,
    };

    // Send to users in the same faculty who can view activities
    if let Some(faculty_id) = session_user.faculty_id {
        sse_manager.send_to_faculty(&session_state, faculty_id, message).await;
    } else {
        // Send to all if no faculty restriction
        sse_manager.broadcast(message).await;
    }

    Ok(())
}

// Helper functions (would need to be implemented or imported)
async fn get_user_by_id(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<crate::models::user::User>, anyhow::Error> {
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&session_state.db_pool)
    .await?;
    
    Ok(user)
}

async fn get_user_admin_role(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<crate::models::admin_role::AdminRole>, anyhow::Error> {
    let admin_role = sqlx::query_as::<_, crate::models::admin_role::AdminRole>(
        "SELECT * FROM admin_roles WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&session_state.db_pool)
    .await?;
    
    Ok(admin_role)
}

// Background task for cleaning up inactive SSE connections
pub async fn sse_cleanup_task(
    session_state: SessionState,
    sse_manager: SseConnectionManager,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    
    loop {
        interval.tick().await;
        
        sse_manager.cleanup_inactive_connections(&session_state).await;
        
        let connection_count = sse_manager.connection_count().await;
        tracing::debug!("Active SSE connections: {}", connection_count);
    }
}