use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
    pub session_secret: String,
    pub session_max_age: i64, // in seconds
    pub bcrypt_cost: u32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost:5432/trackivity".to_string()
            }),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            session_secret: std::env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            session_max_age: std::env::var("SESSION_MAX_AGE")
                .unwrap_or_else(|_| "7200".to_string()) // 2 hours (7200 seconds)
                .parse()?,
            bcrypt_cost: std::env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()?,
        })
    }
}
