use anyhow::Result;
use chrono::{Duration, Utc};
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

use crate::models::{Session, CreateSession};

pub struct SessionService {
    redis: redis::aio::ConnectionManager,
}

impl SessionService {
    pub fn new(redis: redis::aio::ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn create_session(&mut self, data: CreateSession) -> Result<Session> {
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            user_id: data.user_id,
            expires_at: data.expires_at,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ip_address: data.ip_address,
            user_agent: data.user_agent,
            device_info: data.device_info,
            is_active: true,
        };

        let session_json = serde_json::to_string(&session)?;
        let ttl = (data.expires_at - Utc::now()).num_seconds() as u64;

        self.redis
            .set_ex::<String, String, String>(
                format!("session:{}", session_id),
                session_json,
                ttl,
            )
            .await?;

        // Also store user sessions for cleanup
        self.redis
            .sadd::<String, String, i32>(
                format!("user_sessions:{}", session.user_id),
                session_id.clone(),
            )
            .await?;

        self.redis
            .expire::<String, i64>(
                format!("user_sessions:{}", session.user_id),
                ttl as i64,
            )
            .await?;

        Ok(session)
    }

    pub async fn get_session(&mut self, session_id: &str) -> Result<Option<Session>> {
        let session_data: Option<String> = self
            .redis
            .get(format!("session:{}", session_id))
            .await?;

        match session_data {
            Some(data) => {
                let mut session: Session = serde_json::from_str(&data)?;
                
                // Update last accessed time
                session.last_accessed = Utc::now();
                let updated_json = serde_json::to_string(&session)?;
                
                // Get current TTL and update the session
                let ttl: i32 = self.redis.ttl(format!("session:{}", session_id)).await?;
                if ttl > 0 {
                    self.redis
                        .set_ex::<String, String, String>(
                            format!("session:{}", session_id),
                            updated_json,
                            ttl as u64,
                        )
                        .await?;
                }

                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_session(&mut self, session_id: &str) -> Result<()> {
        // Get session to find user_id
        if let Some(session) = self.get_session(session_id).await? {
            self.redis
                .del::<String, i32>(format!("session:{}", session_id))
                .await?;

            self.redis
                .srem::<String, String, i32>(
                    format!("user_sessions:{}", session.user_id),
                    session_id.to_string(),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn delete_user_sessions(&mut self, user_id: Uuid) -> Result<()> {
        let session_ids: Vec<String> = self
            .redis
            .smembers(format!("user_sessions:{}", user_id))
            .await?;

        for session_id in session_ids {
            self.redis
                .del::<String, i32>(format!("session:{}", session_id))
                .await?;
        }

        self.redis
            .del::<String, i32>(format!("user_sessions:{}", user_id))
            .await?;

        Ok(())
    }

    pub async fn extend_session(&mut self, session_id: &str, duration: Duration) -> Result<bool> {
        let exists: bool = self.redis.exists(format!("session:{}", session_id)).await?;
        
        if exists {
            let additional_seconds = duration.num_seconds() as u64;
            self.redis
                .expire::<String, i64>(format!("session:{}", session_id), additional_seconds as i64)
                .await?;
            
            return Ok(true);
        }
        
        Ok(false)
    }

    pub async fn cleanup_expired_sessions(&mut self) -> Result<()> {
        // Redis automatically handles TTL expiration, but we can implement
        // additional cleanup logic here if needed
        Ok(())
    }
}