use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::admin_role::{AdminLevel, AdminRole};

// Session type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionType {
    Student,      // Regular student login
    AdminRegular, // Regular admin login
    AdminFaculty, // Faculty admin login
    AdminSuper,   // Super admin login
}

// Login method tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoginMethod {
    StudentId,    // Login with student ID
    Email,        // Login with email (admin)
    TokenRefresh, // Session refreshed/extended
}

// Session activity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    pub timestamp: DateTime<Utc>,
    pub activity_type: SessionActivityType,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionActivityType {
    Login,
    Logout,
    PermissionChanged,
    ForceLogout,
    SessionExtended,
    SseConnected,
    SseDisconnected,
    SuspiciousActivity,
    AdminAction,
}

impl Default for SessionType {
    fn default() -> Self {
        SessionType::Student
    }
}

impl Default for LoginMethod {
    fn default() -> Self {
        LoginMethod::StudentId
    }
}

// Enhanced Session model for Redis storage with comprehensive admin tracking
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
    // Enhanced admin tracking fields
    pub session_type: SessionType,
    pub admin_level: Option<AdminLevel>,
    pub faculty_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub revoked_by: Option<Uuid>, // Admin who revoked this session
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    pub login_method: LoginMethod,    // Email/StudentID login tracking
    pub sse_connections: Vec<String>, // Track SSE connection IDs
    pub activity_log: Vec<SessionActivity>, // Track important session activities
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
pub struct StudentLoginRequest {
    pub student_id: String,
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
    pub student_id: String,
    pub email: String,
    pub admin_level: Option<AdminLevel>,
    pub faculty_name: Option<String>,
    pub department_name: Option<String>,
    pub session_type: SessionType,
    pub login_method: LoginMethod,
    pub device_info: HashMap<String, Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
    pub sse_connections: Vec<String>,
    pub is_active: bool,
    pub recent_activities: Vec<SessionActivity>,
}

// Enhanced admin session monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSessionMonitor {
    pub total_sessions: usize,
    pub active_admin_sessions: usize,
    pub active_student_sessions: usize,
    pub sessions_by_level: HashMap<String, usize>,
    pub sessions_by_faculty: HashMap<String, usize>,
    pub recent_logins: Vec<AdminSessionInfo>,
    pub suspicious_activities: Vec<SuspiciousActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivity {
    pub session_id: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub activity_type: String,
    pub description: String,
    pub severity: SuspiciousSeverity,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspiciousSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRevocationRequest {
    pub session_id: String,
    pub reason: Option<String>,
}

// Batch session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSessionRevocationRequest {
    pub session_ids: Vec<String>,
    pub reason: Option<String>,
    pub notify_users: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSessionRevocationResponse {
    pub success: bool,
    pub revoked_sessions: Vec<String>,
    pub failed_sessions: Vec<String>,
    pub total_revoked: usize,
    pub errors: Vec<String>,
}

// Force logout all sessions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceLogoutUserRequest {
    pub user_id: Uuid,
    pub reason: Option<String>,
    pub exclude_current_session: bool,
    pub notify_user: bool,
}

// Force logout all sessions for a faculty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceLogoutFacultyRequest {
    pub faculty_id: Uuid,
    pub reason: Option<String>,
    pub admin_level_filter: Option<AdminLevel>,
    pub notify_users: bool,
}

// Session analytics for admin dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub current_active_sessions: usize,
    pub today_logins: usize,
    pub today_admin_logins: usize,
    pub failed_login_attempts: usize,
    pub average_session_duration: f64, // in minutes
    pub most_active_hours: Vec<HourlyActivity>,
    pub device_distribution: HashMap<String, usize>,
    pub browser_distribution: HashMap<String, usize>,
    pub top_active_users: Vec<UserActivity>,
    pub recent_suspicious_activities: Vec<SuspiciousActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyActivity {
    pub hour: u8,
    pub login_count: usize,
    pub active_sessions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: Uuid,
    pub user_name: String,
    pub student_id: String,
    pub admin_level: Option<AdminLevel>,
    pub active_sessions: usize,
    pub total_login_time: f64, // in hours
    pub last_activity: DateTime<Utc>,
}

// Permission and access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    // Super Admin permissions
    ManageAllFaculties,
    ViewSystemReports,
    ManageAdmins,
    ViewAllSessions,
    ForceLogoutAnyUser,
    ViewSuspiciousActivities,
    ManageSystemSettings,
    ViewSystemAnalytics,
    ManageSubscriptions,

    // Faculty Admin permissions
    ManageFacultyStudents,
    ManageFacultyActivities,
    ManageDepartments,
    ViewFacultyReports,
    ManageRegularAdmins,
    ForceLogoutFacultyUsers,
    ViewFacultyAnalytics,
    ManageFacultySessions,

    // Regular Admin permissions
    ScanQrCodes,
    ViewAssignedActivities,
    ManageActivityParticipation,
    ViewActivityReports,

    // Session management permissions
    ViewOwnSessions,
    ManageOwnSessions,
    ExtendOwnSession,

    // Common permissions
    ViewProfile,
    UpdateProfile,

    // SSE permissions
    ReceiveNotifications,
    ReceiveSystemAlerts,
    ReceiveFacultyAlerts,
}

impl Permission {
    pub fn from_admin_level(level: &AdminLevel, _faculty_id: Option<Uuid>) -> Vec<Permission> {
        match level {
            AdminLevel::SuperAdmin => vec![
                // All admin permissions
                Permission::ManageAllFaculties,
                Permission::ViewSystemReports,
                Permission::ManageAdmins,
                Permission::ViewAllSessions,
                Permission::ForceLogoutAnyUser,
                Permission::ViewSuspiciousActivities,
                Permission::ManageSystemSettings,
                Permission::ViewSystemAnalytics,
                Permission::ManageSubscriptions,
                // Session permissions
                Permission::ViewOwnSessions,
                Permission::ManageOwnSessions,
                Permission::ExtendOwnSession,
                // Common permissions
                Permission::ViewProfile,
                Permission::UpdateProfile,
                // SSE permissions
                Permission::ReceiveNotifications,
                Permission::ReceiveSystemAlerts,
            ],
            AdminLevel::FacultyAdmin => vec![
                // Faculty-level permissions
                Permission::ManageFacultyStudents,
                Permission::ManageFacultyActivities,
                Permission::ManageDepartments,
                Permission::ViewFacultyReports,
                Permission::ManageRegularAdmins,
                Permission::ForceLogoutFacultyUsers,
                Permission::ViewFacultyAnalytics,
                Permission::ManageFacultySessions,
                // Session permissions
                Permission::ViewOwnSessions,
                Permission::ManageOwnSessions,
                Permission::ExtendOwnSession,
                // Common permissions
                Permission::ViewProfile,
                Permission::UpdateProfile,
                // SSE permissions
                Permission::ReceiveNotifications,
                Permission::ReceiveFacultyAlerts,
            ],
            AdminLevel::RegularAdmin => vec![
                // Basic admin permissions
                Permission::ScanQrCodes,
                Permission::ViewAssignedActivities,
                Permission::ManageActivityParticipation,
                Permission::ViewActivityReports,
                // Session permissions
                Permission::ViewOwnSessions,
                Permission::ManageOwnSessions,
                Permission::ExtendOwnSession,
                // Common permissions
                Permission::ViewProfile,
                Permission::UpdateProfile,
                // SSE permissions
                Permission::ReceiveNotifications,
            ],
        }
    }

    // Get student permissions (non-admin users)
    pub fn student_permissions() -> Vec<Permission> {
        vec![
            Permission::ViewOwnSessions,
            Permission::ManageOwnSessions,
            Permission::ExtendOwnSession,
            Permission::ViewProfile,
            Permission::UpdateProfile,
            Permission::ReceiveNotifications,
        ]
    }

    pub fn requires_faculty_scope(&self) -> bool {
        matches!(
            self,
            Permission::ManageFacultyStudents
                | Permission::ManageFacultyActivities
                | Permission::ManageDepartments
                | Permission::ViewFacultyReports
                | Permission::ManageRegularAdmins
                | Permission::ForceLogoutFacultyUsers
                | Permission::ViewFacultyAnalytics
                | Permission::ManageFacultySessions
                | Permission::ReceiveFacultyAlerts
        )
    }

    // Check if permission requires super admin level
    pub fn requires_super_admin(&self) -> bool {
        matches!(
            self,
            Permission::ManageAllFaculties
                | Permission::ViewSystemReports
                | Permission::ManageAdmins
                | Permission::ViewAllSessions
                | Permission::ForceLogoutAnyUser
                | Permission::ViewSuspiciousActivities
                | Permission::ManageSystemSettings
                | Permission::ViewSystemAnalytics
                | Permission::ManageSubscriptions
                | Permission::ReceiveSystemAlerts
        )
    }

    // Check if permission is session-related
    pub fn is_session_permission(&self) -> bool {
        matches!(
            self,
            Permission::ViewOwnSessions
                | Permission::ManageOwnSessions
                | Permission::ExtendOwnSession
                | Permission::ViewAllSessions
                | Permission::ForceLogoutAnyUser
                | Permission::ForceLogoutFacultyUsers
                | Permission::ManageFacultySessions
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
    pub os_version: Option<String>,
    pub browser: Option<String>,
    pub browser_version: Option<String>,
    pub app_version: Option<String>,
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub is_trusted_device: bool,
    pub device_fingerprint: Option<String>, // For security tracking
}

impl DeviceInfo {
    pub fn from_user_agent(user_agent: &str) -> Self {
        // Enhanced user agent parsing - in production, use a proper library like woothee
        let device_type = if user_agent.contains("Mobile") {
            Some("mobile".to_string())
        } else if user_agent.contains("Tablet") {
            Some("tablet".to_string())
        } else {
            Some("web".to_string())
        };

        let (os, os_version) = if user_agent.contains("Windows NT 10.0") {
            (Some("Windows".to_string()), Some("10".to_string()))
        } else if user_agent.contains("Windows NT 6.3") {
            (Some("Windows".to_string()), Some("8.1".to_string()))
        } else if user_agent.contains("Windows") {
            (Some("Windows".to_string()), None)
        } else if user_agent.contains("Mac OS X") {
            (Some("macOS".to_string()), None)
        } else if user_agent.contains("Linux") {
            (Some("Linux".to_string()), None)
        } else if user_agent.contains("Android") {
            (Some("Android".to_string()), None)
        } else if user_agent.contains("iOS") {
            (Some("iOS".to_string()), None)
        } else {
            (None, None)
        };

        let (browser, browser_version) = if user_agent.contains("Chrome/") {
            let version = Self::extract_version(user_agent, "Chrome/");
            (Some("Chrome".to_string()), version)
        } else if user_agent.contains("Firefox/") {
            let version = Self::extract_version(user_agent, "Firefox/");
            (Some("Firefox".to_string()), version)
        } else if user_agent.contains("Safari/") && !user_agent.contains("Chrome") {
            let version = Self::extract_version(user_agent, "Version/");
            (Some("Safari".to_string()), version)
        } else if user_agent.contains("Edge/") {
            let version = Self::extract_version(user_agent, "Edge/");
            (Some("Edge".to_string()), version)
        } else {
            (None, None)
        };

        Self {
            device_type,
            os,
            os_version,
            browser,
            browser_version,
            app_version: None,
            screen_resolution: None,
            timezone: None,
            language: None,
            is_trusted_device: false,
            device_fingerprint: None,
        }
    }

    fn extract_version(user_agent: &str, prefix: &str) -> Option<String> {
        if let Some(start) = user_agent.find(prefix) {
            let version_start = start + prefix.len();
            let version_end = user_agent[version_start..]
                .find(' ')
                .map(|i| version_start + i)
                .unwrap_or(user_agent.len());
            Some(user_agent[version_start..version_end].to_string())
        } else {
            None
        }
    }

    // Create comprehensive device info from headers and request data
    pub fn from_headers_and_request(
        user_agent: Option<&str>,
        accept_language: Option<&str>,
        timezone: Option<&str>,
        screen_resolution: Option<&str>,
    ) -> Self {
        let mut device_info = match user_agent {
            Some(ua) => Self::from_user_agent(ua),
            None => Self::default(),
        };

        if let Some(lang) = accept_language {
            device_info.language = Some(lang.split(',').next().unwrap_or(lang).to_string());
        }

        device_info.timezone = timezone.map(|tz| tz.to_string());
        device_info.screen_resolution = screen_resolution.map(|sr| sr.to_string());

        device_info
    }

    // Generate device fingerprint for security tracking
    pub fn generate_fingerprint(&mut self, ip_address: Option<&str>) {
        let mut fingerprint_data = String::new();

        if let Some(device_type) = &self.device_type {
            fingerprint_data.push_str(device_type);
        }
        if let Some(os) = &self.os {
            fingerprint_data.push_str(os);
        }
        if let Some(browser) = &self.browser {
            fingerprint_data.push_str(browser);
        }
        if let Some(screen) = &self.screen_resolution {
            fingerprint_data.push_str(screen);
        }
        if let Some(tz) = &self.timezone {
            fingerprint_data.push_str(tz);
        }
        if let Some(lang) = &self.language {
            fingerprint_data.push_str(lang);
        }
        if let Some(ip) = ip_address {
            fingerprint_data.push_str(ip);
        }

        // Simple hash for fingerprint (in production, use proper hashing)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        fingerprint_data.hash(&mut hasher);
        self.device_fingerprint = Some(format!("{:x}", hasher.finish()));
    }

    pub fn to_json(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        if let Some(ref device_type) = self.device_type {
            map.insert(
                "device_type".to_string(),
                Value::String(device_type.clone()),
            );
        }
        if let Some(ref os) = self.os {
            map.insert("os".to_string(), Value::String(os.clone()));
        }
        if let Some(ref os_version) = self.os_version {
            map.insert("os_version".to_string(), Value::String(os_version.clone()));
        }
        if let Some(ref browser) = self.browser {
            map.insert("browser".to_string(), Value::String(browser.clone()));
        }
        if let Some(ref browser_version) = self.browser_version {
            map.insert(
                "browser_version".to_string(),
                Value::String(browser_version.clone()),
            );
        }
        if let Some(ref app_version) = self.app_version {
            map.insert(
                "app_version".to_string(),
                Value::String(app_version.clone()),
            );
        }
        if let Some(ref screen_resolution) = self.screen_resolution {
            map.insert(
                "screen_resolution".to_string(),
                Value::String(screen_resolution.clone()),
            );
        }
        if let Some(ref timezone) = self.timezone {
            map.insert("timezone".to_string(), Value::String(timezone.clone()));
        }
        if let Some(ref language) = self.language {
            map.insert("language".to_string(), Value::String(language.clone()));
        }
        map.insert(
            "is_trusted_device".to_string(),
            Value::Bool(self.is_trusted_device),
        );
        if let Some(ref fingerprint) = self.device_fingerprint {
            map.insert(
                "device_fingerprint".to_string(),
                Value::String(fingerprint.clone()),
            );
        }
        map
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            device_type: None,
            os: None,
            os_version: None,
            browser: None,
            browser_version: None,
            app_version: None,
            screen_resolution: None,
            timezone: None,
            language: None,
            is_trusted_device: false,
            device_fingerprint: None,
        }
    }
}
