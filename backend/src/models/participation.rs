use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::user::UserPrefix;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "participation_status", rename_all = "snake_case")]
pub enum ParticipationStatus {
    Registered,
    CheckedIn,
    CheckedOut,
    Completed,
    NoShow,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Participation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub status: ParticipationStatus,
    pub registered_at: DateTime<Utc>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub checked_out_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateParticipation {
    pub user_id: Uuid,
    pub activity_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParticipation {
    pub status: Option<ParticipationStatus>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInRequest {
    pub qr_code: String,
    pub activity_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationWithUserDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub status: ParticipationStatus,
    pub registered_at: DateTime<Utc>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub checked_out_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub user_prefix: UserPrefix,
    pub user_first_name: String,
    pub user_last_name: String,
    pub user_full_name: String,
    pub user_full_name_with_prefix: String,
    pub user_student_id: String,
    pub user_email: String,
    pub user_department_name: Option<String>,
}
