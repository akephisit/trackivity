use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use redis::{AsyncCommands, Client};
use serde_json::Value;
use uuid::Uuid;

use crate::models::session::{Session, SessionValidation, CreateSession};

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
            ip_address: create_req.ip_address,
            user_agent: create_req.user_agent,
            device_info: create_req.device_info,
            is_active: true,
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
        conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64).await?;
        
        // Add to user's active sessions set
        let user_sessions_key = format!("user_sessions:{}", create_req.user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, &session_id).await?;
        conn.expire::<_, ()>(&user_sessions_key, ttl_seconds as i64).await?;

        // Track session in global active sessions
        let active_sessions_key = "active_sessions";
        conn.zadd::<_, _, _, ()>(active_sessions_key, &session_id, ttl_seconds).await?;

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
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64).await?;
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
        
        // Get session to find user_id for cleanup
        if let Some(session) = self.get_session(session_id).await? {
            let user_sessions_key = format!("user_sessions:{}", session.user_id);
            conn.srem::<_, _, ()>(&user_sessions_key, session_id).await?;
        }
        
        // Remove session data
        let session_key = format!("session:{}", session_id);
        conn.del::<_, ()>(&session_key).await?;
        
        // Remove from active sessions
        let active_sessions_key = "active_sessions";
        conn.zrem::<_, _, ()>(active_sessions_key, session_id).await?;
        
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

    pub async fn extend_session(&self, session_id: &str, new_expiry: DateTime<Utc>) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let session_key = format!("session:{}", session_id);
        
        if let Some(mut session) = self.get_session(session_id).await? {
            session.expires_at = new_expiry;
            session.last_accessed = Utc::now();
            
            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (new_expiry - Utc::now()).num_seconds();
            
            if ttl_seconds > 0 {
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64).await?;
                
                // Update user sessions expiry
                let user_sessions_key = format!("user_sessions:{}", session.user_id);
                conn.expire::<_, ()>(&user_sessions_key, ttl_seconds as i64).await?;
                
                // Update active sessions score
                let active_sessions_key = "active_sessions";
                conn.zadd::<_, _, _, ()>(active_sessions_key, session_id, ttl_seconds).await?;
                
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
        conn.zrembyscore::<_, _, _, ()>(active_sessions_key, 0, now).await?;
        
        // Get active session IDs
        let session_ids: Vec<String> = match limit {
            Some(n) => conn.zrange(active_sessions_key, 0, (n as isize) - 1).await?,
            None => conn.zrange(active_sessions_key, 0, -1).await?,
        };
        
        Ok(session_ids)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let active_sessions_key = "active_sessions";
        
        let now = Utc::now().timestamp();
        
        // Get expired session IDs
        let expired_sessions: Vec<String> = conn
            .zrangebyscore(active_sessions_key, 0, now)
            .await?;
        
        let count = expired_sessions.len();
        
        // Clean up expired sessions
        for session_id in expired_sessions {
            Box::pin(self.delete_session(&session_id)).await?;
        }
        
        Ok(count)
    }

    pub async fn revoke_session(&self, session_id: &str, reason: Option<String>) -> Result<bool> {
        let session_key = format!("session:{}", session_id);
        let mut conn = self.get_connection().await?;
        
        if let Some(mut session) = self.get_session(session_id).await? {
            session.is_active = false;
            session.last_accessed = Utc::now();
            
            // Add revocation reason to device_info if provided
            if let Some(reason) = reason {
                session.device_info.insert("revocation_reason".to_string(), Value::String(reason));
                session.device_info.insert("revoked_at".to_string(), Value::String(Utc::now().to_rfc3339()));
            }
            
            let session_data = serde_json::to_string(&session)?;
            let ttl_seconds = (session.expires_at - Utc::now()).num_seconds();
            
            if ttl_seconds > 0 {
                // Keep revoked session for audit purposes but mark as inactive
                conn.set_ex::<_, _, ()>(&session_key, session_data, ttl_seconds as u64).await?;
            }
            
            // Remove from active sessions
            let active_sessions_key = "active_sessions";
            conn.zrem::<_, _, ()>(active_sessions_key, session_id).await?;
            
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
            default_expiry_hours: 24,        // 24 hours default
            max_sessions_per_user: 5,        // Max 5 concurrent sessions
            remember_me_expiry_days: 30,     // 30 days for remember me
            cleanup_interval_minutes: 15,    // Cleanup every 15 minutes
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