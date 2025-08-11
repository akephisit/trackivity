use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive},
    response::Sse,
};
use chrono::Utc;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use uuid::Uuid;
use redis::AsyncCommands;
use std::net::IpAddr;
use std::cmp::Ordering;

use crate::middleware::session::SessionState;
use crate::models::session::{Permission, SessionUser};

// Enhanced SSE Connection Manager with Redis PubSub and comprehensive features
#[derive(Clone)]
pub struct SseConnectionManager {
    // Map of session_id -> connection info
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    // Redis connection for PubSub
    redis_client: redis::Client,
    // Rate limiting store (session_id -> (count, last_reset))
    rate_limits: Arc<RwLock<HashMap<String, (u32, chrono::DateTime<Utc>)>>>,
    // Configuration
    config: SseConfig,
}

#[derive(Clone)]
struct ConnectionInfo {
    sender: broadcast::Sender<SseMessage>,
    session_user: SessionUser,
    connected_at: chrono::DateTime<Utc>,
    last_heartbeat: chrono::DateTime<Utc>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
}

#[derive(Clone)]
pub struct SseConfig {
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
    pub max_connections_per_user: u32,
    pub rate_limit_per_minute: u32,
    pub channel_buffer_size: usize,
    pub redis_pubsub_channel: String,
    pub enable_compression: bool,
}

// Enhanced SSE Event Types with type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SseEventType {
    ActivityCheckedIn,
    NewActivityCreated,
    SubscriptionExpiryWarning,
    SystemAnnouncement,
    AdminAssignment,
    PermissionUpdated,
    SessionRevoked,
    AdminPromoted,
    Heartbeat,
    ConnectionStatus,
    Custom(String),
}

impl std::fmt::Display for SseEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SseEventType::ActivityCheckedIn => "activity_checked_in",
            SseEventType::NewActivityCreated => "new_activity_created",
            SseEventType::SubscriptionExpiryWarning => "subscription_expiry_warning",
            SseEventType::SystemAnnouncement => "system_announcement",
            SseEventType::AdminAssignment => "admin_assignment",
            SseEventType::PermissionUpdated => "permission_updated",
            SseEventType::SessionRevoked => "session_revoked",
            SseEventType::AdminPromoted => "admin_promoted",
            SseEventType::Heartbeat => "heartbeat",
            SseEventType::ConnectionStatus => "connection_status",
            SseEventType::Custom(name) => name,
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseMessage {
    pub event_type: SseEventType,
    pub data: Value,
    pub timestamp: chrono::DateTime<Utc>,
    pub message_id: String,
    pub target_permissions: Option<Vec<String>>, // Filter by permissions
    pub target_user_id: Option<Uuid>,            // Target specific user
    pub target_faculty_id: Option<Uuid>,         // Target faculty members
    pub target_sessions: Option<Vec<String>>,    // Target specific sessions
    pub priority: MessagePriority,
    pub ttl_seconds: Option<u32>,
    pub retry_count: u32,
    pub broadcast_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl PartialOrd for MessagePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessagePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_val = match self {
            MessagePriority::Low => 0,
            MessagePriority::Normal => 1,
            MessagePriority::High => 2,
            MessagePriority::Critical => 3,
        };
        let other_val = match other {
            MessagePriority::Low => 0,
            MessagePriority::Normal => 1,
            MessagePriority::High => 2,
            MessagePriority::Critical => 3,
        };
        self_val.cmp(&other_val)
    }
}

// Enhanced message structures with type safety
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub action_url: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub read_receipt_required: bool,
    pub sound_enabled: bool,
    pub category: NotificationCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationCategory {
    Activity,
    System,
    Admin,
    Security,
    Subscription,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityCheckedInMessage {
    pub activity_id: Uuid,
    pub activity_title: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub checked_in_at: chrono::DateTime<Utc>,
    pub qr_code_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewActivityMessage {
    pub activity_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
    pub faculty_id: Option<Uuid>,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionExpiryMessage {
    pub user_id: Uuid,
    pub subscription_type: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub days_remaining: i32,
    pub renewal_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAnnouncementMessage {
    pub announcement_id: Uuid,
    pub title: String,
    pub content: String,
    pub severity: AnnouncementSeverity,
    pub target_audience: Vec<String>,
    pub display_until: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnouncementSeverity {
    Info,
    Important,
    Critical,
    Maintenance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminAssignmentMessage {
    pub assignment_id: Uuid,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub task_type: String,
    pub task_description: String,
    pub due_date: Option<chrono::DateTime<Utc>>,
    pub priority: MessagePriority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionUpdatedMessage {
    pub user_id: Uuid,
    pub updated_permissions: Vec<String>,
    pub updated_by: Uuid,
    pub effective_immediately: bool,
    pub requires_re_login: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRevokedMessage {
    pub session_id: String,
    pub user_id: Uuid,
    pub reason: SessionRevocationReason,
    pub message: String,
    pub revoked_by: Option<Uuid>,
    pub force_logout_all_devices: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionRevocationReason {
    AdminAction,
    SecurityBreach,
    PolicyViolation,
    PermissionChange,
    Expired,
    UserRequested,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPromotedMessage {
    pub user_id: Uuid,
    pub user_name: String,
    pub old_role: Option<String>,
    pub new_role: String,
    pub promoted_by: Uuid,
    pub effective_date: chrono::DateTime<Utc>,
    pub congratulations_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub server_time: chrono::DateTime<Utc>,
    pub connection_count: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatusMessage {
    pub status: ConnectionStatus,
    pub message: String,
    pub reconnect_recommended: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Reconnecting,
    Disconnected,
    Error,
    RateLimited,
    Unauthorized,
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

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(300), // 5 minutes
            max_connections_per_user: 5,
            rate_limit_per_minute: 60,
            channel_buffer_size: 1000,
            redis_pubsub_channel: "sse_broadcast".to_string(),
            enable_compression: true,
        }
    }
}

impl SseConnectionManager {
    pub fn new(redis_client: redis::Client) -> Self {
        Self::with_config(redis_client, SseConfig::default())
    }

    pub fn with_config(redis_client: redis::Client, config: SseConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            redis_client,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    // Enhanced connection management with comprehensive validation
    pub async fn add_connection(
        &self,
        session_id: String,
        session_user: SessionUser,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<broadcast::Receiver<SseMessage>, SseError> {
        // Rate limiting check
        if !self.check_rate_limit(&session_id).await? {
            return Err(SseError::RateLimited);
        }

        // Check max connections per user
        self.enforce_connection_limits(&session_user.user_id).await?;

        let mut connections = self.connections.write().await;

        // Remove old connection if exists
        if let Some(_old_connection) = connections.remove(&session_id) {
            tracing::info!(
                "Replacing existing SSE connection for session: {} (user: {})",
                session_id,
                session_user.user_id
            );
        }

        // Create new broadcast channel with configured buffer size
        let (tx, rx) = broadcast::channel(self.config.channel_buffer_size);
        
        let connection_info = ConnectionInfo {
            sender: tx,
            session_user: session_user.clone(),
            connected_at: Utc::now(),
            last_heartbeat: Utc::now(),
            ip_address,
            user_agent,
        };

        connections.insert(session_id.clone(), connection_info);
        
        tracing::info!(
            "Added SSE connection for session: {} (user: {}, IP: {:?})",
            session_id,
            session_user.user_id,
            ip_address
        );

        // Send welcome message
        self.send_connection_status(
            &session_id,
            ConnectionStatus::Connected,
            "Successfully connected to real-time notifications",
        ).await.ok();

        Ok(rx)
    }

    async fn check_rate_limit(&self, session_id: &str) -> Result<bool, SseError> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();

        match rate_limits.get_mut(session_id) {
            Some((count, last_reset)) => {
                // Reset counter if a minute has passed
                if now.signed_duration_since(*last_reset).num_seconds() >= 60 {
                    *count = 1;
                    *last_reset = now;
                    Ok(true)
                } else if *count >= self.config.rate_limit_per_minute {
                    tracing::warn!("Rate limit exceeded for session: {}", session_id);
                    Ok(false)
                } else {
                    *count += 1;
                    Ok(true)
                }
            }
            None => {
                rate_limits.insert(session_id.to_string(), (1, now));
                Ok(true)
            }
        }
    }

    async fn enforce_connection_limits(&self, user_id: &Uuid) -> Result<(), SseError> {
        let connections = self.connections.read().await;
        let user_connection_count = connections
            .values()
            .filter(|info| info.session_user.user_id == *user_id)
            .count();

        if user_connection_count >= self.config.max_connections_per_user as usize {
            tracing::warn!(
                "User {} has reached maximum connections ({})",
                user_id,
                self.config.max_connections_per_user
            );
            return Err(SseError::TooManyConnections);
        }

        Ok(())
    }

    async fn send_connection_status(
        &self,
        session_id: &str,
        status: ConnectionStatus,
        message: &str,
    ) -> Result<(), SseError> {
        let status_message = ConnectionStatusMessage {
            status,
            message: message.to_string(),
            reconnect_recommended: false,
        };

        let sse_message = SseMessage {
            event_type: SseEventType::ConnectionStatus,
            data: serde_json::to_value(status_message)?,
            timestamp: Utc::now(),
            message_id: format!("conn_status_{}", uuid::Uuid::new_v4()),
            target_permissions: None,
            target_user_id: None,
            target_faculty_id: None,
            target_sessions: Some(vec![session_id.to_string()]),
            priority: MessagePriority::Normal,
            ttl_seconds: Some(30),
            retry_count: 0,
            broadcast_id: None,
        };

        self.send_to_session(session_id, sse_message).await
    }

    // Enhanced connection removal with cleanup
    pub async fn remove_connection(&self, session_id: &str) -> Result<(), SseError> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection_info) = connections.remove(session_id) {
            let duration = Utc::now()
                .signed_duration_since(connection_info.connected_at)
                .num_seconds();
            
            tracing::info!(
                "Removed SSE connection for session: {} (user: {}, duration: {}s)",
                session_id,
                connection_info.session_user.user_id,
                duration
            );

            // Clean up rate limiting data
            self.rate_limits.write().await.remove(session_id);
        }
        
        Ok(())
    }

    // Update heartbeat timestamp
    pub async fn update_heartbeat(&self, session_id: &str) -> Result<(), SseError> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection_info) = connections.get_mut(session_id) {
            connection_info.last_heartbeat = Utc::now();
        }
        
        Ok(())
    }

    // Enhanced message sending with validation and filtering
    pub async fn send_to_session(
        &self,
        session_id: &str,
        message: SseMessage,
    ) -> Result<(), SseError> {
        let connections = self.connections.read().await;

        if let Some(connection_info) = connections.get(session_id) {
            // Validate permissions if message has permission requirements
            if let Some(required_permissions) = &message.target_permissions {
                if !self.user_has_permissions(
                    &connection_info.session_user,
                    required_permissions,
                ) {
                    tracing::debug!(
                        "Message filtered out for session {} due to insufficient permissions",
                        session_id
                    );
                    return Ok(()); // Not an error, just filtered
                }
            }

            // Check target user filter
            if let Some(target_user_id) = message.target_user_id {
                if connection_info.session_user.user_id != target_user_id {
                    tracing::debug!(
                        "Message filtered out for session {} due to user ID mismatch",
                        session_id
                    );
                    return Ok(());
                }
            }

            // Check target faculty filter
            if let Some(target_faculty_id) = message.target_faculty_id {
                if connection_info.session_user.faculty_id != Some(target_faculty_id) {
                    tracing::debug!(
                        "Message filtered out for session {} due to faculty ID mismatch",
                        session_id
                    );
                    return Ok(());
                }
            }

            // Send message
            if let Err(broadcast::error::SendError(_)) = connection_info.sender.send(message.clone()) {
                tracing::warn!(
                    "Failed to send message to session {} - connection closed",
                    session_id
                );
                return Err(SseError::ConnectionClosed);
            }

            tracing::debug!(
                "Sent {} message to session {}",
                message.event_type,
                session_id
            );
        } else {
            tracing::warn!("Session {} not found for message delivery", session_id);
            return Err(SseError::SessionNotFound);
        }

        Ok(())
    }

    fn user_has_permissions(
        &self,
        session_user: &SessionUser,
        required_permissions: &[String],
    ) -> bool {
        required_permissions
            .iter()
            .all(|perm| session_user.permissions.contains(perm))
    }

    // Enhanced broadcasting with filtering and Redis PubSub
    pub async fn broadcast(&self, message: SseMessage) -> Result<u32, SseError> {
        let mut sent_count = 0u32;
        let connections = self.connections.read().await;

        for (session_id, connection_info) in connections.iter() {
            // Apply filtering logic
            if self.should_send_to_connection(&message, connection_info) {
                if connection_info.sender.send(message.clone()).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!(
                        "Failed to send broadcast message to session: {}",
                        session_id
                    );
                }
            }
        }

        // Also broadcast via Redis PubSub for multi-instance support
        if message.broadcast_id.is_some() {
            self.publish_to_redis(&message).await.ok();
        }

        tracing::info!(
            "Broadcast {} message to {} connections",
            message.event_type,
            sent_count
        );

        Ok(sent_count)
    }

    fn should_send_to_connection(
        &self,
        message: &SseMessage,
        connection_info: &ConnectionInfo,
    ) -> bool {
        // Check permissions
        if let Some(required_permissions) = &message.target_permissions {
            if !self.user_has_permissions(&connection_info.session_user, required_permissions) {
                return false;
            }
        }

        // Check target user
        if let Some(target_user_id) = message.target_user_id {
            if connection_info.session_user.user_id != target_user_id {
                return false;
            }
        }

        // Check target faculty
        if let Some(target_faculty_id) = message.target_faculty_id {
            if connection_info.session_user.faculty_id != Some(target_faculty_id) {
                return false;
            }
        }

        true
    }

    // Publish message to Redis for multi-instance support
    async fn publish_to_redis(&self, message: &SseMessage) -> Result<(), SseError> {
        #[allow(deprecated)]
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| SseError::Redis(e.to_string()))?;
        
        let serialized = serde_json::to_string(message)?;
        
        conn.publish::<_, _, ()>(&self.config.redis_pubsub_channel, serialized).await
            .map_err(|e| SseError::Redis(e.to_string()))?;
        
        Ok(())
    }

    // Enhanced permission-based messaging
    pub async fn send_to_permission(
        &self,
        permission: &Permission,
        message: SseMessage,
    ) -> Result<u32, SseError> {
        let mut sent_count = 0u32;
        let connections = self.connections.read().await;
        let permission_str = format!("{:?}", permission);

        for (session_id, connection_info) in connections.iter() {
            if connection_info.session_user.permissions.contains(&permission_str) {
                if connection_info.sender.send(message.clone()).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!(
                        "Failed to send permission-based message to session: {}",
                        session_id
                    );
                }
            }
        }

        tracing::info!(
            "Sent {} message to {} users with {} permission",
            message.event_type,
            sent_count,
            permission_str
        );

        Ok(sent_count)
    }

    // Send to multiple permissions
    pub async fn send_to_permissions(
        &self,
        permissions: &[Permission],
        message: SseMessage,
    ) -> Result<u32, SseError> {
        let mut sent_count = 0u32;
        let connections = self.connections.read().await;
        let permission_strs: Vec<String> = permissions
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();

        for (session_id, connection_info) in connections.iter() {
            // User must have at least one of the required permissions
            let has_permission = permission_strs
                .iter()
                .any(|perm| connection_info.session_user.permissions.contains(perm));

            if has_permission {
                if connection_info.sender.send(message.clone()).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!(
                        "Failed to send multi-permission message to session: {}",
                        session_id
                    );
                }
            }
        }

        Ok(sent_count)
    }

    // Enhanced faculty messaging
    pub async fn send_to_faculty(
        &self,
        faculty_id: Uuid,
        message: SseMessage,
    ) -> Result<u32, SseError> {
        let mut sent_count = 0u32;
        let connections = self.connections.read().await;

        for (session_id, connection_info) in connections.iter() {
            if connection_info.session_user.faculty_id == Some(faculty_id) {
                if connection_info.sender.send(message.clone()).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!(
                        "Failed to send faculty message to session: {}",
                        session_id
                    );
                }
            }
        }

        tracing::info!(
            "Sent {} message to {} faculty members (faculty: {})",
            message.event_type,
            sent_count,
            faculty_id
        );

        Ok(sent_count)
    }

    // Send to multiple faculties
    pub async fn send_to_faculties(
        &self,
        faculty_ids: &[Uuid],
        message: SseMessage,
    ) -> Result<u32, SseError> {
        let mut sent_count = 0u32;
        let connections = self.connections.read().await;

        for (session_id, connection_info) in connections.iter() {
            if let Some(user_faculty_id) = connection_info.session_user.faculty_id {
                if faculty_ids.contains(&user_faculty_id) {
                    if connection_info.sender.send(message.clone()).is_ok() {
                        sent_count += 1;
                    } else {
                        tracing::warn!(
                            "Failed to send multi-faculty message to session: {}",
                            session_id
                        );
                    }
                }
            }
        }

        Ok(sent_count)
    }

    // Enhanced connection statistics
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let now = Utc::now();
        
        let mut stats = ConnectionStats {
            total_connections: connections.len(),
            connections_by_faculty: HashMap::new(),
            connections_by_role: HashMap::new(),
            average_connection_duration: 0,
            stale_connections: 0,
        };

        let mut total_duration = 0i64;
        
        for connection_info in connections.values() {
            // Faculty stats
            if let Some(faculty_id) = connection_info.session_user.faculty_id {
                *stats.connections_by_faculty.entry(faculty_id).or_insert(0) += 1;
            }

            // Role stats (simplified - using admin level if available)
            let role = connection_info.session_user.admin_role
                .as_ref()
                .map(|r| format!("{:?}", r.admin_level))
                .unwrap_or_else(|| "Student".to_string());
            *stats.connections_by_role.entry(role).or_insert(0) += 1;

            // Duration calculation
            let duration = now.signed_duration_since(connection_info.connected_at).num_seconds();
            total_duration += duration;

            // Check for stale connections
            let last_heartbeat_age = now.signed_duration_since(connection_info.last_heartbeat).num_seconds();
            if last_heartbeat_age > self.config.connection_timeout.as_secs() as i64 {
                stats.stale_connections += 1;
            }
        }

        if !connections.is_empty() {
            stats.average_connection_duration = total_duration / connections.len() as i64;
        }

        stats
    }

    // Get connections by user ID
    pub async fn get_user_connections(&self, user_id: &Uuid) -> Vec<String> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter_map(|(session_id, connection_info)| {
                if connection_info.session_user.user_id == *user_id {
                    Some(session_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    // Enhanced cleanup with multiple criteria
    pub async fn cleanup_inactive_connections(
        &self,
        session_state: &SessionState,
    ) -> Result<u32, SseError> {
        let mut connections = self.connections.write().await;
        let mut to_remove = Vec::new();
        let now = Utc::now();

        for (session_id, connection_info) in connections.iter() {
            let mut should_remove = false;
            let mut reason = String::new();

            // Check if session still exists in Redis
            match session_state.redis_store.get_session(session_id).await {
                Ok(None) => {
                    should_remove = true;
                    reason = "Session expired in Redis".to_string();
                }
                Ok(Some(session)) => {
                    // Check if session is expired
                    if session.expires_at <= now {
                        should_remove = true;
                        reason = "Session expired".to_string();
                    }
                    // Check if session is inactive
                    else if !session.is_active {
                        should_remove = true;
                        reason = "Session deactivated".to_string();
                    }
                }
                Err(_) => {
                    // On Redis error, don't remove connections - be conservative
                    tracing::warn!("Failed to check session {} in Redis", session_id);
                }
            }

            // Check for stale heartbeat
            if !should_remove {
                let heartbeat_age = now.signed_duration_since(connection_info.last_heartbeat);
                if heartbeat_age > chrono::Duration::from_std(self.config.connection_timeout).unwrap() {
                    should_remove = true;
                    reason = "Stale heartbeat".to_string();
                }
            }

            if should_remove {
                to_remove.push((session_id.clone(), reason));
            }
        }

        let removed_count = to_remove.len() as u32;

        for (session_id, reason) in to_remove {
            connections.remove(&session_id);
            tracing::info!(
                "Cleaned up inactive SSE connection: {} (reason: {})",
                session_id,
                reason
            );
        }

        // Also clean up rate limiting data
        let mut rate_limits = self.rate_limits.write().await;
        let stale_cutoff = now - chrono::Duration::minutes(5);
        rate_limits.retain(|_session_id, (_, last_reset)| *last_reset > stale_cutoff);

        if removed_count > 0 {
            tracing::info!("Cleaned up {} inactive SSE connections", removed_count);
        }

        Ok(removed_count)
    }

    // Send heartbeat to all connections
    pub async fn send_heartbeat(&self) -> Result<u32, SseError> {
        let connection_count = self.connection_count().await as u32;
        
        let heartbeat_message = HeartbeatMessage {
            server_time: Utc::now(),
            connection_count,
            uptime_seconds: 0, // TODO: Implement actual uptime tracking
        };

        let sse_message = SseMessage {
            event_type: SseEventType::Heartbeat,
            data: serde_json::to_value(heartbeat_message)?,
            timestamp: Utc::now(),
            message_id: format!("heartbeat_{}", uuid::Uuid::new_v4()),
            target_permissions: None,
            target_user_id: None,
            target_faculty_id: None,
            target_sessions: None,
            priority: MessagePriority::Low,
            ttl_seconds: Some(60),
            retry_count: 0,
            broadcast_id: None,
        };

        self.broadcast(sse_message).await
    }

    // Force disconnect user sessions
    pub async fn force_disconnect_user(
        &self,
        user_id: &Uuid,
        reason: SessionRevocationReason,
        message: &str,
        revoked_by: Option<Uuid>,
    ) -> Result<u32, SseError> {
        let user_sessions = self.get_user_connections(user_id).await;
        let mut disconnected_count = 0u32;

        for session_id in user_sessions {
            let revocation_message = SessionRevokedMessage {
                session_id: session_id.clone(),
                user_id: *user_id,
                reason: reason.clone(),
                message: message.to_string(),
                revoked_by,
                force_logout_all_devices: true,
            };

            let sse_message = SseMessage {
                event_type: SseEventType::SessionRevoked,
                data: serde_json::to_value(revocation_message)?,
                timestamp: Utc::now(),
                message_id: format!("session_revoked_{}", uuid::Uuid::new_v4()),
                target_permissions: None,
                target_user_id: Some(*user_id),
                target_faculty_id: None,
                target_sessions: Some(vec![session_id.clone()]),
                priority: MessagePriority::Critical,
                ttl_seconds: None,
                retry_count: 0,
                broadcast_id: None,
            };

            // Send revocation message
            self.send_to_session(&session_id, sse_message).await.ok();
            
            // Remove connection
            self.remove_connection(&session_id).await.ok();
            
            disconnected_count += 1;
        }

        tracing::warn!(
            "Force disconnected {} sessions for user {} (reason: {:?})",
            disconnected_count,
            user_id,
            reason
        );

        Ok(disconnected_count)
    }

    // Send message to specific user by user ID (enhanced version)
    pub async fn send_to_user(&self, user_id: Uuid, message: SseMessage) -> Result<u32, SseError> {
        let user_sessions = self.get_user_connections(&user_id).await;
        let mut sent_count = 0u32;
        
        for session_id in user_sessions {
            if self.send_to_session(&session_id, message.clone()).await.is_ok() {
                sent_count += 1;
            }
        }
        
        Ok(sent_count)
    }
    
    // Broadcast message to all admin users
    pub async fn broadcast_to_admins(&self, message: SseMessage) -> Result<u32, SseError> {
        let connections = self.connections.read().await;
        let mut sent_count = 0u32;
        
        for (session_id, connection_info) in connections.iter() {
            // Check if user has admin role
            if connection_info.session_user.admin_role.is_some() {
                if connection_info.sender.send(message.clone()).is_ok() {
                    sent_count += 1;
                } else {
                    tracing::warn!("Failed to send admin message to session: {}", session_id);
                }
            }
        }
        
        Ok(sent_count)
    }
    
    // Legacy compatibility method
    pub async fn send_message(&self, message: SseMessage) -> Result<u32, SseError> {
        // If targeting specific users, send to user connections
        if let Some(target_user_id) = message.target_user_id {
            return self.send_to_user(target_user_id, message).await;
        }
        
        // Otherwise broadcast with filtering
        self.broadcast(message).await
    }
    
    // Enhanced admin alert with proper typing
    pub async fn send_admin_alert(
        &self,
        title: &str,
        message: &str,
        severity: AnnouncementSeverity,
    ) -> Result<u32, SseError> {
        let alert_message = SystemAnnouncementMessage {
            announcement_id: uuid::Uuid::new_v4(),
            title: title.to_string(),
            content: message.to_string(),
            severity: severity.clone(),
            target_audience: vec!["super_admin".to_string(), "faculty_admin".to_string()],
            display_until: Some(Utc::now() + chrono::Duration::hours(24)),
        };
        
        let sse_message = SseMessage {
            event_type: SseEventType::SystemAnnouncement,
            data: serde_json::to_value(alert_message)?,
            timestamp: Utc::now(),
            message_id: format!("admin_alert_{}", uuid::Uuid::new_v4()),
            target_permissions: Some(vec!["super_admin".to_string(), "faculty_admin".to_string()]),
            target_user_id: None,
            target_faculty_id: None,
            target_sessions: None,
            priority: match severity {
                AnnouncementSeverity::Critical => MessagePriority::Critical,
                AnnouncementSeverity::Important => MessagePriority::High,
                _ => MessagePriority::Normal,
            },
            ttl_seconds: Some(86400),
            retry_count: 0,
            broadcast_id: None,
        };
        
        self.broadcast_to_admins(sse_message).await
    }
    
    // Send system status update
    pub async fn send_system_status(
        &self,
        status_type: &str,
        status_data: serde_json::Value,
    ) -> Result<u32, SseError> {
        let message = SseMessage {
            event_type: SseEventType::Custom("system_status".to_string()),
            data: serde_json::json!({
                "status_type": status_type,
                "data": status_data,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
            message_id: format!("system_status_{}", uuid::Uuid::new_v4()),
            target_permissions: Some(vec!["super_admin".to_string()]),
            target_user_id: None,
            target_faculty_id: None,
            target_sessions: None,
            priority: MessagePriority::Normal,
            ttl_seconds: Some(3600),
            retry_count: 0,
            broadcast_id: None,
        };
        
        self.broadcast_to_admins(message).await
    }
}

// Error types for SSE operations
#[derive(Debug, thiserror::Error)]
pub enum SseError {
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Too many connections")]
    TooManyConnections,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

// Connection statistics
#[derive(Debug, Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub connections_by_faculty: HashMap<Uuid, u32>,
    pub connections_by_role: HashMap<String, u32>,
    pub average_connection_duration: i64,
    pub stale_connections: u32,
}

// Enhanced SSE endpoint handlers

// Universal SSE endpoint with role-based filtering
pub async fn sse_handler(
    State(session_state): State<SessionState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    session_user: SessionUser, // This ensures authentication
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    let sse_manager = session_state
        .sse_manager
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Verify that the session_id matches the authenticated user's session
    if session_user.session_id != session_id {
        tracing::warn!(
            "Session ID mismatch: provided {}, expected {}",
            session_id,
            session_user.session_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Extract client information
    let ip_address = extract_client_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Add connection to manager
    let rx = match sse_manager
        .add_connection(session_id.clone(), session_user.clone(), ip_address, user_agent)
        .await
    {
        Ok(rx) => rx,
        Err(SseError::RateLimited) => {
            tracing::warn!("Rate limit exceeded for session: {}", session_id);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        Err(SseError::TooManyConnections) => {
            tracing::warn!("Too many connections for user: {}", session_user.user_id);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        Err(e) => {
            tracing::error!("Failed to add SSE connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Clone session_id for cleanup
    let cleanup_session_id = session_id.clone();

    // Create the stream with enhanced error handling
    let stream = BroadcastStream::new(rx)
        .map(move |result| {
            match result {
                Ok(message) => {
                    // Check if message has expired
                    if let Some(ttl) = message.ttl_seconds {
                        let age = Utc::now()
                            .signed_duration_since(message.timestamp)
                            .num_seconds() as u32;
                        if age > ttl {
                            tracing::debug!("Dropping expired message: {}", message.message_id);
                            return Ok(Event::default().event("heartbeat").data(""));
                        }
                    }

                    let event_data = serde_json::to_string(&message)
                        .unwrap_or_else(|_| "{\"error\":\"serialization_failed\"}".to_string());

                    let mut event = Event::default()
                        .event(&message.event_type.to_string())
                        .data(event_data);

                    // Add message ID for client-side deduplication
                    event = event.id(&message.message_id);

                    // Set retry time for reconnection
                    if message.priority == MessagePriority::Critical {
                        event = event.retry(Duration::from_secs(1));
                    } else {
                        event = event.retry(Duration::from_secs(5));
                    }

                    Ok(event)
                }
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(count)) => {
                    tracing::warn!(
                        "SSE connection lagged {} messages for session: {}",
                        count,
                        cleanup_session_id
                    );
                    Ok(Event::default()
                        .event("lag_warning")
                        .data(&format!("{{\"lagged_messages\": {}}}", count)))
                }
            }
        });

    // Set up enhanced keep-alive
    let keep_alive = KeepAlive::new()
        .interval(Duration::from_secs(30))
        .text("heartbeat");

    tracing::info!(
        "SSE connection established for session: {} (user: {})",
        session_id,
        session_user.user_id
    );

    Ok(Sse::new(stream).keep_alive(keep_alive))
}

// Student-specific SSE endpoint
pub async fn sse_student_handler(
    State(session_state): State<SessionState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    session_user: SessionUser,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    // Ensure user is a student (has no admin role)
    if session_user.admin_role.is_some() {
        return Err(StatusCode::FORBIDDEN);
    }

    sse_handler(State(session_state), Path(session_id), headers, session_user).await
}

// Admin SSE endpoint with elevated access
pub async fn sse_admin_handler(
    State(session_state): State<SessionState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    session_user: SessionUser,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    // Ensure user has admin role
    if session_user.admin_role.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    sse_handler(State(session_state), Path(session_id), headers, session_user).await
}

// Helper function to extract client IP
fn extract_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // Try X-Forwarded-For header first (for proxies)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP from the comma-separated list
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }
    
    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }
    
    None
}

// Enhanced helper function to extract session ID from headers
fn extract_session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str[7..].trim();
                if !token.is_empty() && token.len() >= 32 { // Basic validation
                    return Some(token.to_string());
                }
            }
        }
    }

    // Try custom session header
    if let Some(session_header) = headers.get("x-session-id") {
        if let Ok(session_str) = session_header.to_str() {
            let session_id = session_str.trim();
            if !session_id.is_empty() && session_id.len() >= 32 {
                return Some(session_id.to_string());
            }
        }
    }

    // Try cookie parsing (enhanced)
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(equals_pos) = cookie.find('=') {
                    let (name, value) = cookie.split_at(equals_pos);
                    if name.trim() == "session_id" {
                        let session_id = value[1..].trim(); // Skip '=' character
                        if !session_id.is_empty() && session_id.len() >= 32 {
                            return Some(session_id.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

// Helper function to create message builders
pub struct SseMessageBuilder {
    event_type: SseEventType,
    data: Value,
    target_permissions: Option<Vec<String>>,
    target_user_id: Option<Uuid>,
    target_faculty_id: Option<Uuid>,
    target_sessions: Option<Vec<String>>,
    priority: MessagePriority,
    ttl_seconds: Option<u32>,
    broadcast_id: Option<String>,
}

impl SseMessageBuilder {
    pub fn new(event_type: SseEventType, data: Value) -> Self {
        Self {
            event_type,
            data,
            target_permissions: None,
            target_user_id: None,
            target_faculty_id: None,
            target_sessions: None,
            priority: MessagePriority::Normal,
            ttl_seconds: Some(3600), // Default 1 hour TTL
            broadcast_id: None,
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.target_permissions = Some(permissions);
        self
    }

    pub fn to_user(mut self, user_id: Uuid) -> Self {
        self.target_user_id = Some(user_id);
        self
    }

    pub fn to_faculty(mut self, faculty_id: Uuid) -> Self {
        self.target_faculty_id = Some(faculty_id);
        self
    }

    pub fn to_sessions(mut self, sessions: Vec<String>) -> Self {
        self.target_sessions = Some(sessions);
        self
    }

    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u32) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }

    pub fn with_broadcast_id(mut self, broadcast_id: String) -> Self {
        self.broadcast_id = Some(broadcast_id);
        self
    }

    pub fn build(self) -> SseMessage {
        SseMessage {
            event_type: self.event_type,
            data: self.data,
            timestamp: Utc::now(),
            message_id: format!("msg_{}", uuid::Uuid::new_v4()),
            target_permissions: self.target_permissions,
            target_user_id: self.target_user_id,
            target_faculty_id: self.target_faculty_id,
            target_sessions: self.target_sessions,
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            retry_count: 0,
            broadcast_id: self.broadcast_id,
        }
    }
}

// Convenience functions for common message types
pub fn create_notification_message(
    title: &str,
    message: &str,
    notification_type: NotificationType,
    category: NotificationCategory,
) -> Result<SseMessage, SseError> {
    let notification = NotificationMessage {
        title: title.to_string(),
        message: message.to_string(),
        notification_type,
        action_url: None,
        expires_at: None,
        read_receipt_required: false,
        sound_enabled: true,
        category,
    };

    Ok(SseMessageBuilder::new(
        SseEventType::Custom("notification".to_string()),
        serde_json::to_value(notification)?,
    ).build())
}

pub fn create_activity_checkin_message(
    activity_id: Uuid,
    activity_title: &str,
    user_id: Uuid,
    user_name: &str,
) -> Result<SseMessage, SseError> {
    let checkin = ActivityCheckedInMessage {
        activity_id,
        activity_title: activity_title.to_string(),
        user_id,
        user_name: user_name.to_string(),
        checked_in_at: Utc::now(),
        qr_code_id: None,
    };

    Ok(SseMessageBuilder::new(
        SseEventType::ActivityCheckedIn,
        serde_json::to_value(checkin)?,
    )
    .with_permissions(vec![
        "ViewAssignedActivities".to_string(),
        "ManageFacultyActivities".to_string(),
    ])
    .with_broadcast_id(format!("activity_checkin_{}", activity_id))
    .build())
}

pub fn create_system_announcement(
    title: &str,
    content: &str,
    severity: AnnouncementSeverity,
    target_audience: Vec<String>,
) -> Result<SseMessage, SseError> {
    let announcement = SystemAnnouncementMessage {
        announcement_id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        content: content.to_string(),
        severity: severity.clone(),
        target_audience: target_audience.clone(),
        display_until: Some(Utc::now() + chrono::Duration::hours(24)),
    };

    let priority = match severity {
        AnnouncementSeverity::Critical => MessagePriority::Critical,
        AnnouncementSeverity::Important => MessagePriority::High,
        _ => MessagePriority::Normal,
    };

    Ok(SseMessageBuilder::new(
        SseEventType::SystemAnnouncement,
        serde_json::to_value(announcement)?,
    )
    .with_permissions(if target_audience.is_empty() {
        vec![]
    } else {
        target_audience
    })
    .with_priority(priority)
    .with_ttl(86400) // 24 hours
    .build())
}

// Helper functions (would need to be implemented or imported)
async fn get_user_by_id(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<crate::models::user::User>, anyhow::Error> {
    let user = sqlx::query_as::<_, crate::models::user::User>("SELECT * FROM users WHERE id = $1")
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
        "SELECT * FROM admin_roles WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&session_state.db_pool)
    .await?;

    Ok(admin_role)
}