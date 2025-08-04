use anyhow::Result;
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::database::Database;
use crate::models::{User, CreateUser, LoginRequest, CreateSession, AdminRole, AdminLevel};
use crate::services::SessionService;

pub struct AuthService {
    database: Database,
    bcrypt_cost: u32,
}

impl AuthService {
    pub fn new(database: Database, bcrypt_cost: u32) -> Self {
        Self {
            database,
            bcrypt_cost,
        }
    }

    pub async fn register_user(&self, user_data: CreateUser) -> Result<User> {
        let password_hash = hash(&user_data.password, self.bcrypt_cost)?;
        let qr_secret = Uuid::new_v4().to_string();
        
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (student_id, email, password_hash, first_name, last_name, qr_secret, department_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            user_data.student_id,
            user_data.email,
            password_hash,
            user_data.first_name,
            user_data.last_name,
            qr_secret,
            user_data.department_id
        )
        .fetch_one(&self.database.pool)
        .await?;

        Ok(user)
    }

    pub async fn authenticate_user(&self, credentials: LoginRequest) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE student_id = $1",
            credentials.student_id
        )
        .fetch_optional(&self.database.pool)
        .await?;

        match user {
            Some(user) => {
                if verify(&credentials.password, &user.password_hash)? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn create_session(
        &self,
        session_service: &mut SessionService,
        user_id: Uuid,
        session_max_age: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String> {
        let expires_at = Utc::now() + Duration::seconds(session_max_age);
        
        let session_data = CreateSession {
            user_id,
            expires_at,
            ip_address,
            user_agent,
            device_info: std::collections::HashMap::new(),
        };

        let session = session_service.create_session(session_data).await?;
        Ok(session.id)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.database.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_admin_role(&self, user_id: Uuid) -> Result<Option<AdminRole>> {
        let admin_role = sqlx::query_as!(
            AdminRole,
            r#"
            SELECT id, user_id, admin_level as "admin_level: AdminLevel", faculty_id, permissions, created_at, updated_at
            FROM admin_roles 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.database.pool)
        .await?;

        Ok(admin_role)
    }

    pub async fn change_password(&self, user_id: Uuid, new_password: &str) -> Result<()> {
        let password_hash = hash(new_password, self.bcrypt_cost)?;
        
        sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            password_hash,
            user_id
        )
        .execute(&self.database.pool)
        .await?;

        Ok(())
    }

    pub async fn verify_qr_secret(&self, qr_secret: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE qr_secret = $1",
            qr_secret
        )
        .fetch_optional(&self.database.pool)
        .await?;

        Ok(user)
    }

    pub async fn regenerate_qr_secret(&self, user_id: Uuid) -> Result<String> {
        let new_qr_secret = Uuid::new_v4().to_string();
        
        sqlx::query!(
            "UPDATE users SET qr_secret = $1, updated_at = NOW() WHERE id = $2",
            new_qr_secret,
            user_id
        )
        .execute(&self.database.pool)
        .await?;

        Ok(new_qr_secret)
    }
}