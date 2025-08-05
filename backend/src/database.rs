use anyhow::Result;
use sqlx::{PgPool, Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = Pool::<Postgres>::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
