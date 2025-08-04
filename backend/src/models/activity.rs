use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "activity_status", rename_all = "snake_case")]
pub enum ActivityStatus {
    Draft,
    Published,
    Ongoing,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub status: ActivityStatus,
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivity {
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub max_participants: Option<i32>,
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActivity {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub max_participants: Option<i32>,
    pub status: Option<ActivityStatus>,
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}