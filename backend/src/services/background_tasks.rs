use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

use crate::middleware::session::SessionState;

// Background task manager
pub struct BackgroundTaskManager {
    session_state: SessionState,
}

impl BackgroundTaskManager {
    pub fn new(session_state: SessionState) -> Self { Self { session_state } }

    // Start all background tasks
    pub async fn start_all_tasks(&self) {
        let session_state = self.session_state.clone();

        // Session cleanup task
        let cleanup_session_state = session_state.clone();
        tokio::spawn(async move {
            session_cleanup_task(cleanup_session_state).await;
        });

        // Database session sync task
        let sync_session_state = session_state.clone();
        tokio::spawn(async move {
            database_session_sync_task(sync_session_state).await;
        });


        // Session activity monitoring task
        let monitoring_session_state = session_state.clone();
        tokio::spawn(async move {
            session_activity_monitoring_task(monitoring_session_state).await;
        });

        // Admin session audit task
        let audit_session_state = session_state.clone();
        tokio::spawn(async move {
            admin_session_audit_task(audit_session_state).await;
        });

        tracing::info!("All background tasks started successfully");
    }
}

// Task 1: Clean up expired sessions from Redis
async fn session_cleanup_task(session_state: SessionState) {
    let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

    tracing::info!("Started session cleanup task (every 5 minutes for 2-hour inactivity timeout)");

    loop {
        interval.tick().await;

        match session_state.redis_store.cleanup_expired_sessions().await {
            Ok(cleaned_count) => {
                if cleaned_count > 0 {
                    tracing::info!("Cleaned up {} expired sessions", cleaned_count);
                }
            }
            Err(e) => {
                tracing::error!("Failed to cleanup expired sessions: {}", e);
            }
        }
    }
}

// Task 2: Sync session metadata between Redis and Database
async fn database_session_sync_task(session_state: SessionState) {
    let mut interval = interval(Duration::from_secs(600)); // Every 10 minutes

    tracing::info!("Started database session sync task");

    loop {
        interval.tick().await;

        match sync_database_sessions(&session_state).await {
            Ok(synced_count) => {
                if synced_count > 0 {
                    tracing::debug!("Synced {} sessions with database", synced_count);
                }
            }
            Err(e) => {
                tracing::error!("Failed to sync sessions with database: {}", e);
            }
        }
    }
}


// Task 5: Monitor session activity patterns
async fn session_activity_monitoring_task(session_state: SessionState) {
    let mut interval = interval(Duration::from_secs(3600)); // Every hour

    tracing::info!("Started session activity monitoring task");

    loop {
        interval.tick().await;

        match monitor_session_activity(&session_state).await {
            Ok(stats) => {
                tracing::info!(
                    "Session activity stats - Total: {}, Admin: {}, Active: {}",
                    stats.total_sessions,
                    stats.admin_sessions,
                    stats.active_sessions
                );
            }
            Err(e) => {
                tracing::error!("Failed to monitor session activity: {}", e);
            }
        }
    }
}

// Task 6: Audit admin session activities
async fn admin_session_audit_task(session_state: SessionState) {
    let mut interval = interval(Duration::from_secs(1800)); // Every 30 minutes

    tracing::info!("Started admin session audit task");

    loop {
        interval.tick().await;

        match audit_admin_sessions(&session_state).await {
            Ok(audit_count) => {
                if audit_count > 0 {
                    tracing::debug!("Audited {} admin sessions", audit_count);
                }
            }
            Err(e) => {
                tracing::error!("Failed to audit admin sessions: {}", e);
            }
        }
    }
}

// Helper functions

async fn sync_database_sessions(session_state: &SessionState) -> Result<usize, anyhow::Error> {
    // Get active sessions from Redis
    let active_session_ids = session_state.redis_store.get_active_sessions(None).await?;

    let mut synced_count = 0;

    for session_id in &active_session_ids {
        if let Some(session) = session_state.redis_store.get_session(session_id).await? {
            // Update database record
            let result = sqlx::query(
                r#"
                UPDATE sessions 
                SET last_accessed = $1, expires_at = $2, is_active = $3
                WHERE id = $4
                "#,
            )
            .bind(session.last_accessed)
            .bind(session.expires_at)
            .bind(session.is_active)
            .bind(&session.id)
            .execute(&session_state.db_pool)
            .await?;

            if result.rows_affected() > 0 {
                synced_count += 1;
            }
        }
    }

    // Mark inactive sessions in database
    sqlx::query(
        r#"
        UPDATE sessions 
        SET is_active = false 
        WHERE id NOT IN (
            SELECT UNNEST($1::text[])
        ) AND is_active = true
        "#,
    )
    .bind(&active_session_ids)
    .execute(&session_state.db_pool)
    .await?;

    Ok(synced_count)
}


#[derive(Debug)]
struct SessionActivityStats {
    total_sessions: usize,
    admin_sessions: usize,
    active_sessions: usize,
    device_breakdown: std::collections::HashMap<String, usize>,
    faculty_breakdown: std::collections::HashMap<String, usize>,
}

async fn monitor_session_activity(
    session_state: &SessionState,
) -> Result<SessionActivityStats, anyhow::Error> {
    let session_ids = session_state.redis_store.get_active_sessions(None).await?;

    let mut stats = SessionActivityStats {
        total_sessions: session_ids.len(),
        admin_sessions: 0,
        active_sessions: 0,
        device_breakdown: std::collections::HashMap::new(),
        faculty_breakdown: std::collections::HashMap::new(),
    };

    for session_id in session_ids {
        if let Some(session) = session_state.redis_store.get_session(&session_id).await? {
            // Check if session is active (accessed within last 30 minutes)
            let last_activity = Utc::now().timestamp() - session.last_accessed.timestamp();
            if last_activity <= 1800 {
                // 30 minutes
                stats.active_sessions += 1;
            }

            // Check if user is admin
            if let Ok(Some(admin_role)) = get_user_admin_role(session_state, session.user_id).await
            {
                stats.admin_sessions += 1;

                // Track faculty breakdown for admins
                if let Some(faculty_id) = admin_role.faculty_id {
                    if let Ok(Some(faculty)) = get_faculty_by_id(session_state, faculty_id).await {
                        *stats.faculty_breakdown.entry(faculty.name).or_insert(0) += 1;
                    }
                }
            }

            // Track device breakdown
            let device_type = session
                .device_info
                .get("device_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            *stats.device_breakdown.entry(device_type).or_insert(0) += 1;
        }
    }

    // Log detailed stats
    tracing::info!("Session Activity Monitoring:");
    tracing::info!("  Total Sessions: {}", stats.total_sessions);
    tracing::info!("  Active Sessions: {}", stats.active_sessions);
    tracing::info!("  Admin Sessions: {}", stats.admin_sessions);
    tracing::info!("  Device Breakdown: {:?}", stats.device_breakdown);
    tracing::info!("  Faculty Breakdown: {:?}", stats.faculty_breakdown);

    Ok(stats)
}

async fn audit_admin_sessions(session_state: &SessionState) -> Result<usize, anyhow::Error> {
    // Get all admin sessions
    let session_ids = session_state.redis_store.get_active_sessions(None).await?;
    let mut audit_count = 0;

    for session_id in session_ids {
        if let Some(session) = session_state.redis_store.get_session(&session_id).await? {
            // Check if user is admin
            if let Ok(Some(admin_role)) = get_user_admin_role(session_state, session.user_id).await
            {
                // Log admin session activity
                tracing::info!(
                    "Admin Session Audit - User: {}, Level: {:?}, Faculty: {:?}, IP: {:?}, Last Active: {}",
                    session.user_id,
                    admin_role.admin_level,
                    admin_role.faculty_id,
                    session.ip_address,
                    session.last_accessed
                );

                // Store audit record in database (optional)
                let audit_record = sqlx::query(
                    r#"
                    INSERT INTO session_audit_log (session_id, user_id, admin_level, faculty_id, ip_address, user_agent, audit_timestamp)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (session_id, audit_timestamp) DO NOTHING
                    "#
                )
                .bind(&session.id)
                .bind(session.user_id)
                .bind(format!("{:?}", admin_role.admin_level))
                .bind(admin_role.faculty_id)
                .bind(&session.ip_address)
                .bind(&session.user_agent)
                .bind(Utc::now().date_naive())
                .execute(&session_state.db_pool)
                .await;

                if audit_record.is_ok() {
                    audit_count += 1;
                }
            }
        }
    }

    Ok(audit_count)
}

// Helper functions (would need to be implemented or imported)
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

async fn get_faculty_by_id(
    session_state: &SessionState,
    faculty_id: Uuid,
) -> Result<Option<crate::models::faculty::Faculty>, anyhow::Error> {
    let faculty = sqlx::query_as::<_, crate::models::faculty::Faculty>(
        "SELECT * FROM faculties WHERE id = $1",
    )
    .bind(faculty_id)
    .fetch_optional(&session_state.db_pool)
    .await?;

    Ok(faculty)
}
