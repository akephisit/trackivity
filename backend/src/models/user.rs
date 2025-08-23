use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_prefix", rename_all = "snake_case")]
pub enum UserPrefix {
    #[sqlx(rename = "นาย")]
    Mr,
    #[sqlx(rename = "นาง")]
    Mrs,
    #[sqlx(rename = "นางสาว")]
    Miss,
    #[sqlx(rename = "ดร.")]
    Dr,
    #[sqlx(rename = "ศาสตราจารย์")]
    Professor,
    #[sqlx(rename = "รองศาสตราจารย์")]
    AssociateProfessor,
    #[sqlx(rename = "ผู้ช่วยศาสตราจารย์")]
    AssistantProfessor,
    #[sqlx(rename = "อาจารย์")]
    Lecturer,
    #[sqlx(rename = "คุณ")]
    Generic,
}

impl UserPrefix {
    pub fn to_thai_string(&self) -> &'static str {
        match self {
            UserPrefix::Mr => "นาย",
            UserPrefix::Mrs => "นาง",
            UserPrefix::Miss => "นางสาว",
            UserPrefix::Dr => "ดร.",
            UserPrefix::Professor => "ศาสตราจารย์",
            UserPrefix::AssociateProfessor => "รองศาสตราจารย์",
            UserPrefix::AssistantProfessor => "ผู้ช่วยศาสตราจารย์",
            UserPrefix::Lecturer => "อาจารย์",
            UserPrefix::Generic => "คุณ",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub student_id: String,
    pub email: String,
    pub password_hash: String,
    pub prefix: UserPrefix,
    pub first_name: String,
    pub last_name: String,
    pub qr_secret: String,
    pub department_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub student_id: String,
    pub email: String,
    pub password: String,
    pub prefix: UserPrefix,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub prefix: Option<UserPrefix>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub student_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub student_id: String,
    pub email: String,
    pub prefix: UserPrefix,
    pub first_name: String,
    pub last_name: String,
    pub full_name_with_prefix: String,
    pub department_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let full_name_with_prefix = format!("{}{} {}", 
            user.prefix.to_thai_string(), 
            user.first_name, 
            user.last_name
        );
        
        UserResponse {
            id: user.id,
            student_id: user.student_id,
            email: user.email,
            prefix: user.prefix,
            first_name: user.first_name,
            last_name: user.last_name,
            full_name_with_prefix,
            department_id: user.department_id,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
