use anyhow::Result;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Configure connection pool with optimized settings for SQLx 0.8.6
        let pool = PgPoolOptions::new()
            .max_connections(20) // Maximum connections in the pool
            .min_connections(5)  // Minimum connections to maintain
            .acquire_timeout(Duration::from_secs(30)) // Connection acquire timeout
            .idle_timeout(Duration::from_secs(300))   // Connection idle timeout (5 minutes)
            .max_lifetime(Duration::from_secs(3600))  // Connection maximum lifetime (1 hour)
            .test_before_acquire(true) // Test connections before using them
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn new_with_custom_config(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(3600))
            .test_before_acquire(true)
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    /// Check if database has been migrated (by checking if users table exists)
    pub async fn is_migrated(&self) -> Result<bool> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'users'
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result > 0)
    }

    /// Run migrations only if not already migrated
    pub async fn migrate_if_needed(&self) -> Result<()> {
        if !self.is_migrated().await? {
            tracing::info!("Database not initialized. Running migrations...");
            sqlx::migrate!("./migrations").run(&self.pool).await?;
            tracing::info!("Database migrations completed successfully");
        } else {
            tracing::info!("Database already initialized. Running any pending migrations...");
            sqlx::migrate!("./migrations").run(&self.pool).await?;
            tracing::info!("Migration check completed successfully");
        }
        Ok(())
    }


    /// Force run migrations (for manual migration)
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get pool statistics for monitoring
    pub fn pool_stats(&self) -> (usize, u32) {
        (self.pool.num_idle(), self.pool.size())
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<bool> {
        let result = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(result == 1)
    }
}
