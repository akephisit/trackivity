use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use redis::{AsyncCommands, Client};
use serde_json::Value;
use uuid::Uuid;

use crate::models::admin_role::AdminLevel;
use crate::models::session::{
    BatchSessionRevocationRequest, BatchSessionRevocationResponse, CreateSession, LoginMethod, Session, SessionActivity,
    SessionActivityType, SessionValidation, SessionType, ForceLogoutUserRequest, ForceLogoutFacultyRequest,
};

pub struct RedisSessionStore {
    client: Client,
}

impl RedisSessionStore {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    // Session management methods
    pub async fn create_session(&self, create_req: CreateSession) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            user_id: create_req.user_id,
            expires_at: create_req.expires_at,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ip_address: create_req.ip_address.clone(),
            user_agent: create_req.user_agent.clone(),
            device_info: create_req.device_info,
            is_active: true,
            // Initialize new fields with defaults
            session_type: SessionType::Student, // Will be updated by caller
            admin_level: None,
            faculty_id: None,
            permissions: Vec::new(),
            revoked_by: None,
            revoked_at: None,
            revocation_reason: None,
            login_method: LoginMethod::StudentId, // Will be updated by caller
            sse_connections: Vec::new(),
            activity_log: vec![SessionActivity {
                timestamp: Utc::now(),
                activity_type: SessionActivityType::Login,
                details: Some("Session created".to_string()),
                ip_address: create_req.ip_address.clone(),
                user_agent: create_req.user_agent.clone(),
            }],
        };

        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);
        let session_data = serde_json::to_string(&session)?;

        // Calculate TTL in seconds
        let ttl_seconds = (create_req.expires_at - Utc::now()).num_seconds();
        if ttl_seconds <= 0 {
            return Err(anyhow::anyhow!("Session expiry time is in the past"));
        }

        // Set session data with expiration
        conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
            .await?;

        // Add to user's active sessions set
        let user_sessions_key = format!("user_sessions:{}", create_req.user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, &session_id)
            .await?;
        conn.expire::<_, ()>(&user_sessions_key, ttl_seconds as i64)
            .await?;

        // Track session in global active sessions
        let active_sessions_key = "active_sessions";
        conn.zadd::<_, _, _, ()>(active_sessions_key, &session_id, ttl_seconds)
            .await?;

        Ok(session)
    }

    // Create admin session with enhanced tracking
    pub async fn create_admin_session(
        &self,
        create_req: CreateSession,
        session_type: SessionType,
        admin_level: AdminLevel,
        faculty_id: Option<Uuid>,
        permissions: Vec<String>,
        login_method: LoginMethod,
    ) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            user_id: create_req.user_id,
            expires_at: create_req.expires_at,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ip_address: create_req.ip_address.clone(),
            user_agent: create_req.user_agent.clone(),
            device_info: create_req.device_info,
            is_active: true,
            // Admin-specific fields
            session_type,
            admin_level: Some(admin_level.clone()),
            faculty_id,
            permissions,
            revoked_by: None,
            revoked_at: None,
            revocation_reason: None,
            login_method,
            sse_connections: Vec::new(),
            activity_log: vec![SessionActivity {
                timestamp: Utc::now(),
                activity_type: SessionActivityType::Login,
                details: Some(format!(
                    "Admin session created with level: {:?}",
                    admin_level
                )),
                ip_address: create_req.ip_address.clone(),
                user_agent: create_req.user_agent.clone(),
            }],
        };

        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);
        let session_data = serde_json::to_string(&session)?;

        // Calculate TTL in seconds
        let ttl_seconds = (create_req.expires_at - Utc::now()).num_seconds();
        if ttl_seconds <= 0 {
            return Err(anyhow::anyhow!("Session expiry time is in the past"));
        }

        // Set session data with expiration
        conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
            .await?;

        // Add to user's active sessions set
        let user_sessions_key = format!("user_sessions:{}", create_req.user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, &session_id)
            .await?;
        conn.expire::<_, ()>(&user_sessions_key, ttl_seconds as i64)
            .await?;

        // Track session in global active sessions
        let active_sessions_key = "active_sessions";
        conn.zadd::<_, _, _, ()>(active_sessions_key, &session_id, ttl_seconds)
            .await?;

        // Track admin session separately
        self.track_admin_session(&session_id, &admin_level, create_req.expires_at)
            .await?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);

        let session_data: Option<String> = conn.get(&session_key).await?;

        match session_data {
            Some(data) => {
                let session: Session = serde_json::from_str(&data)?;

                // Check if session is expired
                if session.expires_at <= Utc::now() {
                    // Clean up expired session
                    Box::pin(self.delete_session(session_id)).await?;
                    return Ok(None);
                }

                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);

        // Get current session
        if let Some(mut session) = self.get_session(session_id).await? {
            session.last_accessed = Utc::now();

            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();

            if ttl_seconds > 0 {
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn validate_session(&self, session_id: &str) -> Result<SessionValidation> {
        match self.get_session(session_id).await? {
            Some(session) => {
                if !session.is_active {
                    return Ok(SessionValidation::Revoked);
                }

                if session.expires_at <= Utc::now() {
                    self.delete_session(session_id).await?;
                    return Ok(SessionValidation::Expired);
                }

                // Update last accessed time
                self.update_session_activity(session_id).await?;

                // Note: SessionUser creation would need database queries for user details
                // This is handled in the session service layer
                Ok(SessionValidation::Invalid) // Placeholder - actual user data needed
            }
            None => Ok(SessionValidation::Invalid),
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Get session to find user_id and admin_level for cleanup
        if let Some(session) = self.get_session(session_id).await? {
            let user_sessions_key = format!("user_sessions:{}", session.user_id);
            conn.srem::<_, _, ()>(&user_sessions_key, session_id)
                .await?;

            // Remove admin session tracking if it's an admin session
            if let Some(admin_level) = &session.admin_level {
                self.untrack_admin_session(session_id, admin_level).await?;
            }

            // Log session deletion activity
            self.add_session_activity(
                session_id,
                SessionActivityType::Logout,
                Some("Session deleted".to_string()),
                None,
                None,
            )
            .await
            .unwrap_or_default(); // Ignore errors since session is being deleted
        }

        // Remove session data
        let session_key = format!("session:{}", session_id);
        conn.del::<_, ()>(&session_key).await?;

        // Remove from active sessions
        let active_sessions_key = "active_sessions";
        conn.zrem::<_, _, ()>(active_sessions_key, session_id)
            .await?;

        Ok(())
    }

    pub async fn delete_user_sessions(&self, user_id: Uuid) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let user_sessions_key = format!("user_sessions:{}", user_id);

        // Get all user sessions
        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;

        // Delete each session
        for session_id in &session_ids {
            Box::pin(self.delete_session(session_id)).await?;
        }

        // Clear user sessions set
        conn.del::<_, ()>(&user_sessions_key).await?;

        Ok(session_ids)
    }

    pub async fn extend_session(
        &self,
        session_id: &str,
        new_expiry: DateTime<Utc>,
    ) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);

        if let Some(mut session) = self.get_session(session_id).await? {
            session.expires_at = new_expiry;
            session.last_accessed = Utc::now();

            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (new_expiry - Utc::now()).num_seconds();

            if ttl_seconds > 0 {
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                    .await?;

                // Update user sessions expiry
                let user_sessions_key = format!("user_sessions:{}", session.user_id);
                conn.expire::<_, ()>(&user_sessions_key, ttl_seconds as i64)
                    .await?;

                // Update active sessions score
                let active_sessions_key = "active_sessions";
                conn.zadd::<_, _, _, ()>(active_sessions_key, session_id, ttl_seconds)
                    .await?;

                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let mut conn = self.get_connection().await?;
        let user_sessions_key = format!("user_sessions:{}", user_id);

        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;
        let mut sessions = Vec::new();

        for session_id in session_ids {
            if let Some(session) = self.get_session(&session_id).await? {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    pub async fn get_active_sessions(&self, limit: Option<usize>) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";

        let now = Utc::now().timestamp();

        // Remove expired sessions from sorted set
        conn.zrembyscore::<_, _, _, ()>(active_sessions_key, 0, now)
            .await?;

        // Get active session IDs
        let session_ids: Vec<String> = match limit {
            Some(n) => {
                conn.zrange(active_sessions_key, 0, (n as isize) - 1)
                    .await?
            }
            None => conn.zrange(active_sessions_key, 0, -1).await?,
        };

        Ok(session_ids)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";

        let now = Utc::now().timestamp();

        // Get expired session IDs
        let expired_sessions: Vec<String> = conn.zrangebyscore(active_sessions_key, 0, now).await?;

        let count = expired_sessions.len();

        // Clean up expired sessions
        for session_id in expired_sessions {
            Box::pin(self.delete_session(&session_id)).await?;
        }

        Ok(count)
    }

    pub async fn revoke_session(&self, session_id: &str, reason: Option<String>) -> Result<bool> {
        self.revoke_session_by_admin(session_id, reason, None).await
    }

    // Enhanced revoke session with admin tracking
    pub async fn revoke_session_by_admin(
        &self,
        session_id: &str,
        reason: Option<String>,
        revoked_by_admin_id: Option<Uuid>,
    ) -> Result<bool> {
        let session_key = format!("session:{}", session_id);
        let mut conn = self.get_connection().await?;

        if let Some(mut session) = self.get_session(session_id).await? {
            session.is_active = false;
            session.last_accessed = Utc::now();
            session.revoked_at = Some(Utc::now());
            session.revoked_by = revoked_by_admin_id;
            session.revocation_reason = reason.clone();

            // Add revocation reason to device_info if provided (legacy support)
            if let Some(ref reason) = reason {
                session.device_info.insert(
                    "revocation_reason".to_string(),
                    Value::String(reason.clone()),
                );
                session.device_info.insert(
                    "revoked_at".to_string(),
                    Value::String(Utc::now().to_rfc3339()),
                );
            }

            // Add revocation activity to log
            session.activity_log.push(SessionActivity {
                timestamp: Utc::now(),
                activity_type: SessionActivityType::ForceLogout,
                details: reason
                    .clone()
                    .or_else(|| Some("Session revoked".to_string())),
                ip_address: None,
                user_agent: None,
            });

            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();

            if ttl_seconds > 0 {
                // Keep revoked session for audit purposes but mark as inactive
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                    .await?;
            }

            // Remove from active sessions
            let active_sessions_key = "active_sessions";
            conn.zrem::<_, _, ()>(active_sessions_key, session_id)
                .await?;

            // Remove from admin session tracking if it's an admin session
            if let Some(admin_level) = &session.admin_level {
                self.untrack_admin_session(session_id, admin_level).await?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_session_count(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";

        let now = Utc::now().timestamp();

        // Count only non-expired sessions
        let count: usize = conn.zcount(active_sessions_key, now + 1, "+inf").await?;

        Ok(count)
    }

    pub async fn get_user_session_count(&self, user_id: Uuid) -> Result<usize> {
        let user_sessions = self.get_user_sessions(user_id).await?;
        Ok(user_sessions.len())
    }

    // ========== ADMIN SESSION MANAGEMENT METHODS ==========

    // Get all admin sessions with detailed information
    pub async fn get_admin_sessions(
        &self,
        admin_level_filter: Option<AdminLevel>,
    ) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let admin_sessions_key = "admin_sessions";

        let now = Utc::now().timestamp();

        // Clean up expired admin sessions
        conn.zrembyscore::<_, _, _, ()>(admin_sessions_key, 0, now)
            .await?;

        // Get active admin session IDs
        let session_ids: Vec<String> = conn.zrange(admin_sessions_key, 0, -1).await?;

        // Filter by admin level if specified
        if let Some(filter_level) = admin_level_filter {
            let mut filtered_sessions = Vec::new();

            for session_id in session_ids {
                if let Some(session) = self.get_session(&session_id).await? {
                    if let Some(session_admin_level) = &session.admin_level {
                        if session_admin_level == &filter_level {
                            filtered_sessions.push(session_id);
                        }
                    }
                }
            }

            return Ok(filtered_sessions);
        }

        Ok(session_ids)
    }

    // Track admin session in separate sorted set
    pub async fn track_admin_session(
        &self,
        session_id: &str,
        admin_level: &AdminLevel,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let admin_sessions_key = "admin_sessions";
        let level_sessions_key = format!("admin_sessions:{:?}", admin_level);

        let expiry_timestamp = expires_at.timestamp();

        // Add to general admin sessions
        conn.zadd::<_, _, _, ()>(admin_sessions_key, session_id, expiry_timestamp)
            .await?;

        // Add to level-specific admin sessions
        conn.zadd::<_, _, _, ()>(&level_sessions_key, session_id, expiry_timestamp)
            .await?;

        Ok(())
    }

    // Remove admin session tracking
    pub async fn untrack_admin_session(
        &self,
        session_id: &str,
        admin_level: &AdminLevel,
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let admin_sessions_key = "admin_sessions";
        let level_sessions_key = format!("admin_sessions:{:?}", admin_level);

        // Remove from general admin sessions
        conn.zrem::<_, _, ()>(admin_sessions_key, session_id)
            .await?;

        // Remove from level-specific admin sessions
        conn.zrem::<_, _, ()>(&level_sessions_key, session_id)
            .await?;

        Ok(())
    }

    // Get session count by admin level
    pub async fn get_admin_session_count_by_level(
        &self,
        admin_level: &AdminLevel,
    ) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let level_sessions_key = format!("admin_sessions:{:?}", admin_level);

        let now = Utc::now().timestamp();

        // Count only non-expired sessions
        let count: usize = conn.zcount(&level_sessions_key, now + 1, "+inf").await?;

        Ok(count)
    }

    // Batch revoke sessions
    pub async fn batch_revoke_sessions(
        &self,
        request: BatchSessionRevocationRequest,
    ) -> Result<BatchSessionRevocationResponse> {
        let mut revoked_sessions = Vec::new();
        let mut failed_sessions = Vec::new();
        let mut errors = Vec::new();

        for session_id in &request.session_ids {
            match self
                .revoke_session(session_id, request.reason.clone())
                .await
            {
                Ok(true) => revoked_sessions.push(session_id.clone()),
                Ok(false) => {
                    failed_sessions.push(session_id.clone());
                    errors.push(format!("Session {} not found", session_id));
                }
                Err(e) => {
                    failed_sessions.push(session_id.clone());
                    errors.push(format!("Failed to revoke session {}: {}", session_id, e));
                }
            }
        }

        let total_revoked = revoked_sessions.len();
        let success = failed_sessions.is_empty();

        Ok(BatchSessionRevocationResponse {
            success,
            revoked_sessions,
            failed_sessions,
            total_revoked,
            errors,
        })
    }

    // Force logout all sessions for a user
    pub async fn force_logout_user(&self, request: ForceLogoutUserRequest) -> Result<Vec<String>> {
        let user_sessions = self.get_user_sessions(request.user_id).await?;
        let mut revoked_sessions = Vec::new();

        for session in user_sessions {
            // Skip current session if requested
            if request.exclude_current_session {
                // This would need the current session ID to be passed in the request
                // For now, we'll revoke all sessions
            }

            if self
                .revoke_session(&session.id, request.reason.clone())
                .await?
            {
                revoked_sessions.push(session.id);
            }
        }

        Ok(revoked_sessions)
    }

    // Force logout all sessions for a faculty
    pub async fn force_logout_faculty(
        &self,
        request: ForceLogoutFacultyRequest,
    ) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";

        let now = Utc::now().timestamp();

        // Get all active session IDs
        let session_ids: Vec<String> = conn
            .zrangebyscore(active_sessions_key, now + 1, "+inf")
            .await?;
        let mut revoked_sessions = Vec::new();

        for session_id in session_ids {
            if let Some(session) = self.get_session(&session_id).await? {
                // Check if session belongs to the faculty
                if session.faculty_id == Some(request.faculty_id) {
                    // Check admin level filter if specified
                    if let Some(filter_level) = &request.admin_level_filter {
                        if let Some(session_level) = &session.admin_level {
                            if session_level != filter_level {
                                continue;
                            }
                        } else {
                            continue; // Skip non-admin sessions if admin level filter is specified
                        }
                    }

                    if self
                        .revoke_session(&session_id, request.reason.clone())
                        .await?
                    {
                        revoked_sessions.push(session_id);
                    }
                }
            }
        }

        Ok(revoked_sessions)
    }

    // Add session activity log
    pub async fn add_session_activity(
        &self,
        session_id: &str,
        activity_type: SessionActivityType,
        details: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            let activity = SessionActivity {
                timestamp: Utc::now(),
                activity_type,
                details,
                ip_address,
                user_agent,
            };

            session.activity_log.push(activity);

            // Keep only last 50 activities to prevent memory bloat
            if session.activity_log.len() > 50 {
                session
                    .activity_log
                    .drain(0..session.activity_log.len() - 50);
            }

            // Update session in Redis
            let mut conn = self.get_connection().await?;
            let session_key = format!("session:{}", session_id);
            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();

            if ttl_seconds > 0 {
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                    .await?;
            }
        }

        Ok(())
    }

    // Add SSE connection to session
    pub async fn add_sse_connection(
        &self,
        session_id: &str,
        sse_connection_id: String,
    ) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            if !session.sse_connections.contains(&sse_connection_id) {
                session.sse_connections.push(sse_connection_id);

                // Update session in Redis
                let mut conn = self.get_connection().await?;
                let session_key = format!("session:{}", session_id);
                let session_data = serde_json::to_string(&session)?;
                let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();

                if ttl_seconds > 0 {
                    conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                        .await?;
                }

                // Log SSE connection activity
                self.add_session_activity(
                    session_id,
                    SessionActivityType::SseConnected,
                    Some("SSE connection established".to_string()),
                    None,
                    None,
                )
                .await?;
            }
        }

        Ok(())
    }

    // Remove SSE connection from session
    pub async fn remove_sse_connection(
        &self,
        session_id: &str,
        sse_connection_id: &str,
    ) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.sse_connections.retain(|id| id != sse_connection_id);

            // Update session in Redis
            let mut conn = self.get_connection().await?;
            let session_key = format!("session:{}", session_id);
            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();

            if ttl_seconds > 0 {
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64)
                    .await?;
            }

            // Log SSE disconnection activity
            self.add_session_activity(
                session_id,
                SessionActivityType::SseDisconnected,
                Some("SSE connection closed".to_string()),
                None,
                None,
            )
            .await?;
        }

        Ok(())
    }

    // Get sessions with active SSE connections
    pub async fn get_sessions_with_sse(&self) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";

        let now = Utc::now().timestamp();
        let session_ids: Vec<String> = conn
            .zrangebyscore(active_sessions_key, now + 1, "+inf")
            .await?;
        let mut sessions_with_sse = Vec::new();

        for session_id in session_ids {
            if let Some(session) = self.get_session(&session_id).await? {
                if !session.sse_connections.is_empty() {
                    sessions_with_sse.push(session_id);
                }
            }
        }

        Ok(sessions_with_sse)
    }
}

// Session configuration constants
#[derive(Clone)]
pub struct SessionConfig {
    pub default_expiry_hours: i64,
    pub max_sessions_per_user: usize,
    pub remember_me_expiry_days: i64,
    pub cleanup_interval_minutes: i64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_expiry_hours: 2,      // 2 hours default for inactivity timeout
            max_sessions_per_user: 5,     // Max 5 concurrent sessions
            remember_me_expiry_days: 30,  // 30 days for remember me
            cleanup_interval_minutes: 5,  // Cleanup every 5 minutes for faster cleanup
        }
    }
}

impl SessionConfig {
    pub fn get_session_expiry(&self, remember_me: bool) -> DateTime<Utc> {
        if remember_me {
            Utc::now() + Duration::days(self.remember_me_expiry_days)
        } else {
            Utc::now() + Duration::hours(self.default_expiry_hours)
        }
    }
}
