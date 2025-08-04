use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::admin_role::{AdminLevel, AdminRole};

// Enhanced Session model for Redis storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: HashMap<String, Value>,
    pub is_active: bool,
}

// Database model for session tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SessionRecord {
    pub id: String,
    pub user_id: Uuid,
    pub device_info: Value,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

// Session user data with permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user_id: Uuid,
    pub student_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
    pub admin_role: Option<AdminRole>,
    pub session_id: String,
    pub permissions: Vec<String>,
    pub faculty_id: Option<Uuid>,
}

// Request models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub device_info: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub user: SessionUser,
    pub expires_at: DateTime<Utc>,
}

// Admin session management models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSessionInfo {
    pub session_id: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub admin_level: Option<AdminLevel>,
    pub faculty_name: Option<String>,
    pub device_info: HashMap<String, Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRevocationRequest {
    pub session_id: String,
    pub reason: Option<String>,
}

// Permission and access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    // Super Admin permissions
    ManageAllFaculties,
    ViewSystemReports,
    ManageAdmins,
    ViewAllSessions,
    
    // Faculty Admin permissions
    ManageFacultyStudents,
    ManageFacultyActivities,
    ManageDepartments,
    ViewFacultyReports,
    ManageRegularAdmins,
    
    // Regular Admin permissions
    ScanQrCodes,
    ViewAssignedActivities,
    ManageActivityParticipation,
    
    // Common permissions
    ViewProfile,
    UpdateProfile,
}

impl Permission {
    pub fn from_admin_level(level: &AdminLevel, _faculty_id: Option<Uuid>) -> Vec<Permission> {
        match level {
            AdminLevel::SuperAdmin => vec![
                Permission::ManageAllFaculties,
                Permission::ViewSystemReports, 
                Permission::ManageAdmins,
                Permission::ViewAllSessions,
                Permission::ViewProfile,
                Permission::UpdateProfile,
            ],
            AdminLevel::FacultyAdmin => vec![
                Permission::ManageFacultyStudents,
                Permission::ManageFacultyActivities,
                Permission::ManageDepartments,
                Permission::ViewFacultyReports,
                Permission::ManageRegularAdmins,
                Permission::ViewProfile,
                Permission::UpdateProfile,
            ],
            AdminLevel::RegularAdmin => vec![
                Permission::ScanQrCodes,
                Permission::ViewAssignedActivities,
                Permission::ManageActivityParticipation,
                Permission::ViewProfile,
                Permission::UpdateProfile,
            ],
        }
    }
    
    pub fn requires_faculty_scope(&self) -> bool {
        matches!(self, 
            Permission::ManageFacultyStudents | 
            Permission::ManageFacultyActivities |
            Permission::ManageDepartments |
            Permission::ViewFacultyReports |
            Permission::ManageRegularAdmins
        )
    }
}

// Session validation result
#[derive(Debug, Clone)]
pub enum SessionValidation {
    Valid(SessionUser),
    Expired,
    Invalid,
    Revoked,
}

// Device information helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: Option<String>, // "web", "mobile", "tablet"
    pub os: Option<String>,
    pub browser: Option<String>,
    pub app_version: Option<String>,
}

impl DeviceInfo {
    pub fn from_user_agent(user_agent: &str) -> Self {
        // Simple user agent parsing - in production, use a proper library
        let device_type = if user_agent.contains("Mobile") {
            Some("mobile".to_string())
        } else if user_agent.contains("Tablet") {
            Some("tablet".to_string())
        } else {
            Some("web".to_string())
        };
        
        let os = if user_agent.contains("Windows") {
            Some("Windows".to_string())
        } else if user_agent.contains("Mac") {
            Some("macOS".to_string())
        } else if user_agent.contains("Linux") {
            Some("Linux".to_string())
        } else if user_agent.contains("Android") {
            Some("Android".to_string())
        } else if user_agent.contains("iOS") {
            Some("iOS".to_string())
        } else {
            None
        };
        
        let browser = if user_agent.contains("Chrome") {
            Some("Chrome".to_string())
        } else if user_agent.contains("Firefox") {
            Some("Firefox".to_string())
        } else if user_agent.contains("Safari") {
            Some("Safari".to_string())
        } else if user_agent.contains("Edge") {
            Some("Edge".to_string())
        } else {
            None
        };
        
        Self {
            device_type,
            os,
            browser,
            app_version: None,
        }
    }
    
    pub fn to_json(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(ref device_type) = self.device_type {
            map.insert("device_type".to_string(), Value::String(device_type.clone()));
        }
        if let Some(ref os) = self.os {
            map.insert("os".to_string(), Value::String(os.clone()));
        }
        if let Some(ref browser) = self.browser {
            map.insert("browser".to_string(), Value::String(browser.clone()));
        }
        if let Some(ref app_version) = self.app_version {
            map.insert("app_version".to_string(), Value::String(app_version.clone()));
        }
        map
    }
}