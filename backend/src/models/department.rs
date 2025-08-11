use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub faculty_id: Uuid,
    pub description: Option<String>,
    pub status: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDepartment {
    pub name: String,
    pub code: String,
    pub faculty_id: Uuid,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDepartment {
    pub name: Option<String>,
    pub code: Option<String>,
    pub faculty_id: Option<Uuid>,
    pub description: Option<String>,
}
