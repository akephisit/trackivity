use std::time::Duration;
use tokio_stream::StreamExt;
use chrono::{Timelike, Utc};

use crate::handlers::sse_enhanced::*;
use crate::middleware::session::SessionState;

// Enhanced background tasks

// Background task for cleaning up inactive SSE connections
pub async fn sse_cleanup_task(
    session_state: SessionState, 
    sse_manager: SseConnectionManager
) {
    let cleanup_interval = Duration::from_secs(300); // 5 minutes
    let mut interval = tokio::time::interval(cleanup_interval);

    tracing::info!("Starting SSE cleanup task with {}s interval", cleanup_interval.as_secs());

    loop {
        interval.tick().await;

        match sse_manager.cleanup_inactive_connections(&session_state).await {
            Ok(cleaned_count) => {
                if cleaned_count > 0 {
                    tracing::info!("Cleaned up {} inactive SSE connections", cleaned_count);
                }
            }
            Err(e) => {
                tracing::error!("Failed to cleanup SSE connections: {}", e);
            }
        }

        let connection_count = sse_manager.connection_count().await;
        tracing::debug!("Active SSE connections: {}", connection_count);
        
        // Log connection stats periodically
        if connection_count > 0 {
            let stats = sse_manager.get_connection_stats().await;
            tracing::debug!(
                "SSE Stats - Total: {}, By Faculty: {:?}, Stale: {}",
                stats.total_connections,
                stats.connections_by_faculty,
                stats.stale_connections
            );
        }
    }
}

// Background task for sending heartbeats
pub async fn sse_heartbeat_task(sse_manager: SseConnectionManager, config: SseConfig) {
    let mut interval = tokio::time::interval(config.heartbeat_interval);

    tracing::info!(
        "Starting SSE heartbeat task with {}s interval",
        config.heartbeat_interval.as_secs()
    );

    loop {
        interval.tick().await;

        match sse_manager.send_heartbeat().await {
            Ok(sent_count) => {
                if sent_count > 0 {
                    tracing::debug!("Sent heartbeat to {} connections", sent_count);
                }
            }
            Err(e) => {
                tracing::error!("Failed to send heartbeat: {}", e);
            }
        }
    }
}

// Background task for Redis PubSub message handling
pub async fn redis_pubsub_task(
    sse_manager: SseConnectionManager,
    redis_client: redis::Client,
    channel: String,
) {
    
    tracing::info!("Starting Redis PubSub task for channel: {}", channel);

    loop {
        #[allow(deprecated)]
        match redis_client.get_async_connection().await {
            Ok(conn) => {
                let mut pubsub = conn.into_pubsub();
                
                if let Err(e) = pubsub.subscribe(&channel).await {
                    tracing::error!("Failed to subscribe to Redis channel {}: {}", channel, e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }

                tracing::info!("Successfully subscribed to Redis channel: {}", channel);
                
                loop {
                    match pubsub.on_message().next().await {
                        Some(msg) => {
                            let payload: String = match msg.get_payload() {
                                Ok(p) => p,
                                Err(e) => {
                                    tracing::warn!("Failed to get Redis message payload: {}", e);
                                    continue;
                                }
                            };

                            match serde_json::from_str::<SseMessage>(&payload) {
                                Ok(sse_message) => {
                                    match sse_manager.broadcast(sse_message).await {
                                        Ok(sent_count) => {
                                            tracing::debug!(
                                                "Broadcast Redis message to {} connections",
                                                sent_count
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to broadcast Redis message: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse Redis SSE message: {}", e);
                                }
                            }
                        }
                        None => {
                            tracing::warn!("Redis PubSub stream ended, reconnecting...");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to connect to Redis for PubSub: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// Background task for monitoring connection health
pub async fn sse_health_monitor_task(
    sse_manager: SseConnectionManager,
    config: SseConfig,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(60)); // 1 minute
    let mut last_stats = sse_manager.get_connection_stats().await;

    tracing::info!("Starting SSE health monitor task");

    loop {
        interval.tick().await;

        let current_stats = sse_manager.get_connection_stats().await;
        
        // Log significant changes
        if current_stats.total_connections != last_stats.total_connections {
            let change = current_stats.total_connections as i32 - last_stats.total_connections as i32;
            tracing::info!(
                "SSE connection count changed: {} -> {} ({}{})",
                last_stats.total_connections,
                current_stats.total_connections,
                if change > 0 { "+" } else { "" },
                change
            );
        }

        // Alert on high stale connection count
        if current_stats.stale_connections > 0 {
            let stale_percentage = (current_stats.stale_connections as f64 / 
                current_stats.total_connections.max(1) as f64) * 100.0;
            
            if stale_percentage > 20.0 {
                tracing::warn!(
                    "High percentage of stale connections: {}/{} ({:.1}%)",
                    current_stats.stale_connections,
                    current_stats.total_connections,
                    stale_percentage
                );
            }
        }

        // Alert on very high connection count
        if current_stats.total_connections > config.max_connections_per_user as usize * 100 {
            tracing::warn!(
                "Very high connection count: {} (may indicate resource leak)",
                current_stats.total_connections
            );
        }

        last_stats = current_stats;
    }
}

// Background task for periodic statistics logging
pub async fn sse_stats_logger_task(sse_manager: SseConnectionManager) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

    tracing::info!("Starting SSE statistics logging task");

    loop {
        interval.tick().await;

        let stats = sse_manager.get_connection_stats().await;
        
        if stats.total_connections > 0 {
            tracing::info!(
                "SSE Statistics - Total: {}, Avg Duration: {}s, Stale: {}, By Role: {:?}",
                stats.total_connections,
                stats.average_connection_duration,
                stats.stale_connections,
                stats.connections_by_role
            );

            if !stats.connections_by_faculty.is_empty() {
                tracing::debug!("SSE connections by faculty: {:?}", stats.connections_by_faculty);
            }
        }
    }
}

// Background task for automatic system announcements
pub async fn sse_system_announcements_task(
    sse_manager: SseConnectionManager,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour

    tracing::info!("Starting SSE system announcements task");

    loop {
        interval.tick().await;

        // Check for scheduled maintenance notifications
        if should_send_maintenance_reminder() {
            let announcement = create_system_announcement(
                "Scheduled Maintenance",
                "System maintenance is scheduled for tonight at 2:00 AM. The system may be unavailable for up to 30 minutes.",
                AnnouncementSeverity::Important,
                vec![] // Send to all users
            );

            match announcement {
                Ok(message) => {
                    match sse_manager.broadcast(message).await {
                        Ok(sent_count) => {
                            tracing::info!("Sent maintenance reminder to {} users", sent_count);
                        }
                        Err(e) => {
                            tracing::error!("Failed to send maintenance reminder: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create maintenance announcement: {}", e);
                }
            }
        }

        // Check for subscription expiry warnings
        // This would integrate with your subscription system
        // send_subscription_expiry_warnings(&sse_manager).await;
    }
}

// Helper function to determine if maintenance reminder should be sent
fn should_send_maintenance_reminder() -> bool {
    let now = Utc::now();
    let hour = now.hour();
    
    // Send reminder at 6 PM if maintenance is scheduled for 2 AM
    hour == 18 && now.minute() < 5
}

// Background task to handle connection overflow
pub async fn sse_connection_overflow_handler(
    sse_manager: SseConnectionManager,
    config: SseConfig,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(30)); // 30 seconds

    tracing::info!("Starting SSE connection overflow handler");

    loop {
        interval.tick().await;

        let stats = sse_manager.get_connection_stats().await;
        let max_total_connections = config.max_connections_per_user as usize * 200; // Reasonable limit

        if stats.total_connections > max_total_connections {
            tracing::warn!(
                "Connection overflow detected: {} connections (max: {})",
                stats.total_connections,
                max_total_connections
            );

            // Send alert to admin users
            match sse_manager.send_admin_alert(
                "SSE Connection Overflow",
                &format!(
                    "SSE connection count has exceeded the safe limit: {} connections active",
                    stats.total_connections
                ),
                AnnouncementSeverity::Critical,
            ).await {
                Ok(_) => {
                    tracing::info!("Sent connection overflow alert to admins");
                }
                Err(e) => {
                    tracing::error!("Failed to send connection overflow alert: {}", e);
                }
            }

            // Force cleanup to help resolve the issue
            // This would be handled by the cleanup task, but we can trigger it manually
        }

        // Also check for memory usage patterns that might indicate leaks
        if stats.stale_connections > ((stats.total_connections / 4) as u32) {
            tracing::warn!(
                "High stale connection ratio: {}/{} may indicate connection leaks",
                stats.stale_connections,
                stats.total_connections
            );
        }
    }
}

// Utility function to spawn all SSE background tasks
pub fn spawn_sse_background_tasks(
    session_state: SessionState,
    sse_manager: SseConnectionManager,
    config: SseConfig,
    redis_client: redis::Client,
) -> Vec<tokio::task::JoinHandle<()>> {
    let mut handles = Vec::new();

    // Cleanup task
    handles.push(tokio::spawn({
        let session_state = session_state.clone();
        let sse_manager = sse_manager.clone();
        async move {
            sse_cleanup_task(session_state, sse_manager).await;
        }
    }));

    // Heartbeat task
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        let config = config.clone();
        async move {
            sse_heartbeat_task(sse_manager, config).await;
        }
    }));

    // Redis PubSub task
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        let redis_client = redis_client.clone();
        let channel = config.redis_pubsub_channel.clone();
        async move {
            redis_pubsub_task(sse_manager, redis_client, channel).await;
        }
    }));

    // Health monitor task
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        let config = config.clone();
        async move {
            sse_health_monitor_task(sse_manager, config).await;
        }
    }));

    // Statistics logger task
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        async move {
            sse_stats_logger_task(sse_manager).await;
        }
    }));

    // System announcements task
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        async move {
            sse_system_announcements_task(sse_manager).await;
        }
    }));

    // Connection overflow handler
    handles.push(tokio::spawn({
        let sse_manager = sse_manager.clone();
        let config = config.clone();
        async move {
            sse_connection_overflow_handler(sse_manager, config).await;
        }
    }));

    tracing::info!("Spawned {} SSE background tasks", handles.len());
    handles
}