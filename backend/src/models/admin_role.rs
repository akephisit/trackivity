use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "admin_level", rename_all = "snake_case")]
pub enum AdminLevel {
    SuperAdmin,
    FacultyAdmin,
    RegularAdmin,
}

impl Default for AdminLevel {
    fn default() -> Self {
        AdminLevel::RegularAdmin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub admin_level: AdminLevel,
    pub faculty_id: Option<Uuid>, // null for super admin, required for faculty admin
    pub permissions: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdminRole {
    pub user_id: Uuid,
    pub admin_level: AdminLevel,
    pub faculty_id: Option<Uuid>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdminRole {
    pub admin_level: Option<AdminLevel>,
    pub faculty_id: Option<Uuid>,
    pub permissions: Option<Vec<String>>,
}
