use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::middleware::session::{
    create_session_cookie, delete_session_cookie, AdminUser, SessionState, SuperAdminUser,
};
use crate::models::{
    admin_role::{AdminLevel, AdminRole},
    session::{
        AdminSessionInfo, CreateSession, DeviceInfo, LoginMethod, Permission, SessionActivityType,
        SessionLoginRequest, SessionResponse, SessionRevocationRequest, SessionType, SessionUser,
        StudentLoginRequest,
    },
    user::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub session: Option<SessionResponse>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub student_id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub user_id: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<AdminSessionInfo>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionsResponse {
    pub sessions: Vec<SessionInfo>,
    pub active_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub device_info: HashMap<String, Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub last_accessed: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}

// Student login - no admin privileges required
#[debug_handler]
pub async fn student_login(
    State(session_state): State<SessionState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(login_req): Json<StudentLoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Get IP address from headers first
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    // Extract device info from headers
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");

    let accept_language = headers.get("accept-language").and_then(|h| h.to_str().ok());

    let timezone = headers.get("x-timezone").and_then(|h| h.to_str().ok());

    let screen_resolution = headers
        .get("x-screen-resolution")
        .and_then(|h| h.to_str().ok());

    let mut device_info = match login_req.device_info {
        Some(info) => info,
        None => DeviceInfo::from_headers_and_request(
            Some(user_agent),
            accept_language,
            timezone,
            screen_resolution,
        )
        .to_json(),
    };

    // Generate device fingerprint for security
    let mut device_info_obj = DeviceInfo::from_headers_and_request(
        Some(user_agent),
        accept_language,
        timezone,
        screen_resolution,
    );
    device_info_obj.generate_fingerprint(ip_address.as_deref());
    device_info.extend(device_info_obj.to_json());

    // Authenticate user by student ID
    let user = match authenticate_user_by_student_id(
        &session_state,
        &login_req.student_id,
        &login_req.password,
    )
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(Json(LoginResponse {
                success: false,
                session: None,
                message: "Invalid student ID or password".to_string(),
            }));
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // For student login, ensure user is NOT an admin
    let admin_role = get_user_admin_role(&session_state, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if admin_role.is_some() {
        return Ok(Json(LoginResponse {
            success: false,
            session: None,
            message: "Admin users must use admin login portal".to_string(),
        }));
    }

    // Check if the user's faculty is active
    let user_faculty_id = get_user_faculty_id(&session_state, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let faculty_is_active = check_faculty_is_active(&session_state, user_faculty_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !faculty_is_active {
        return Ok(Json(LoginResponse {
            success: false,
            session: None,
            message: "Access denied. Your faculty is currently inactive.".to_string(),
        }));
    }

    // Check existing sessions and enforce limits
    let existing_sessions = session_state
        .redis_store
        .get_user_sessions(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_sessions.len() >= session_state.config.max_sessions_per_user {
        // Remove oldest session
        if let Some(oldest_session) = existing_sessions.iter().min_by_key(|s| s.created_at) {
            let _ = session_state
                .redis_store
                .delete_session(&oldest_session.id)
                .await;
        }
    }

    // Create new session
    let remember_me = login_req.remember_me.unwrap_or(false);
    let expires_at = session_state.config.get_session_expiry(remember_me);

    let create_session = CreateSession {
        user_id: user.id,
        expires_at,
        ip_address: ip_address.clone(),
        user_agent: Some(user_agent.to_string()),
        device_info: device_info.clone(),
    };

    // Create regular student session
    let mut session = session_state
        .redis_store
        .create_session(create_session)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update session with student-specific details
    session.session_type = SessionType::Student;
    session.login_method = LoginMethod::StudentId;
    session.permissions = Permission::student_permissions()
        .into_iter()
        .map(|p| format!("{:?}", p))
        .collect();

    // Log the successful login activity
    session_state
        .redis_store
        .add_session_activity(
            &session.id,
            SessionActivityType::Login,
            Some("Student login successful".to_string()),
            ip_address.clone(),
            Some(user_agent.to_string()),
        )
        .await
        .unwrap_or_default();

    // Store session metadata in database
    let _ = store_session_metadata(&session_state, &session).await;

    // For student login, admin_role should be None
    let admin_role = None;

    // Build session user
    let session_user = build_session_user(&user, &admin_role, &session.id);

    // Set session cookie
    let max_age_seconds = (expires_at - Utc::now()).num_seconds();
    let cookie = create_session_cookie(&session.id, max_age_seconds);
    cookies.add(cookie);

    let response = SessionResponse {
        session_id: session.id,
        user: session_user,
        expires_at,
    };

    Ok(Json(LoginResponse {
        success: true,
        session: Some(response),
        message: "Login successful".to_string(),
    }))
}

// Admin login - requires admin privileges
#[debug_handler]
pub async fn admin_login(
    State(session_state): State<SessionState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(login_req): Json<SessionLoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Get IP address from headers first
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    // Extract device info from headers
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");

    let accept_language = headers.get("accept-language").and_then(|h| h.to_str().ok());

    let timezone = headers.get("x-timezone").and_then(|h| h.to_str().ok());

    let screen_resolution = headers
        .get("x-screen-resolution")
        .and_then(|h| h.to_str().ok());

    let mut device_info = match login_req.device_info {
        Some(info) => info,
        None => DeviceInfo::from_headers_and_request(
            Some(user_agent),
            accept_language,
            timezone,
            screen_resolution,
        )
        .to_json(),
    };

    // Generate device fingerprint for security
    let mut device_info_obj = DeviceInfo::from_headers_and_request(
        Some(user_agent),
        accept_language,
        timezone,
        screen_resolution,
    );
    device_info_obj.generate_fingerprint(ip_address.as_deref());
    device_info.extend(device_info_obj.to_json());

    // Authenticate user
    let user = match authenticate_user(&session_state, &login_req.email, &login_req.password).await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(Json(LoginResponse {
                success: false,
                session: None,
                message: "Invalid email or password".to_string(),
            }));
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // For admin login, ensure user IS an admin
    let admin_role = get_user_admin_role(&session_state, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if admin_role.is_none() {
        return Ok(Json(LoginResponse {
            success: false,
            session: None,
            message: "Access denied. Admin privileges required.".to_string(),
        }));
    }

    // For faculty admins, check if their faculty is active
    let admin_role_ref = admin_role.as_ref().unwrap();
    if admin_role_ref.faculty_id.is_some() {
        let faculty_is_active = check_faculty_is_active(&session_state, admin_role_ref.faculty_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !faculty_is_active {
            return Ok(Json(LoginResponse {
                success: false,
                session: None,
                message: "Access denied. Your faculty is currently inactive.".to_string(),
            }));
        }
    }

    // Check existing sessions and enforce limits
    let existing_sessions = session_state
        .redis_store
        .get_user_sessions(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_sessions.len() >= session_state.config.max_sessions_per_user {
        // Remove oldest session
        if let Some(oldest_session) = existing_sessions.iter().min_by_key(|s| s.created_at) {
            let _ = session_state
                .redis_store
                .delete_session(&oldest_session.id)
                .await;
        }
    }

    // Create new admin session
    let remember_me = login_req.remember_me.unwrap_or(false);
    let expires_at = session_state.config.get_session_expiry(remember_me);

    let create_session = CreateSession {
        user_id: user.id,
        expires_at,
        ip_address: ip_address.clone(),
        user_agent: Some(user_agent.to_string()),
        device_info: device_info.clone(),
    };

    // Determine session type based on admin level
    let session_type = match admin_role.as_ref().unwrap().admin_level {
        AdminLevel::SuperAdmin => SessionType::AdminSuper,
        AdminLevel::FacultyAdmin => SessionType::AdminFaculty,
        AdminLevel::RegularAdmin => SessionType::AdminRegular,
    };

    // Get permissions for admin level
    let permissions = Permission::from_admin_level(
        &admin_role.as_ref().unwrap().admin_level,
        admin_role.as_ref().unwrap().faculty_id,
    )
    .into_iter()
    .map(|p| format!("{:?}", p))
    .collect();

    let session = session_state
        .redis_store
        .create_admin_session(
            create_session,
            session_type,
            admin_role.as_ref().unwrap().admin_level.clone(),
            admin_role.as_ref().unwrap().faculty_id,
            permissions,
            LoginMethod::Email,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store session metadata in database
    let _ = store_session_metadata(&session_state, &session).await;

    // Build session user with admin role
    let session_user = build_session_user(&user, &admin_role, &session.id);

    // Set session cookie
    let max_age_seconds = (expires_at - Utc::now()).num_seconds();
    let cookie = create_session_cookie(&session.id, max_age_seconds);
    cookies.add(cookie);

    let response = SessionResponse {
        session_id: session.id,
        user: session_user,
        expires_at,
    };

    Ok(Json(LoginResponse {
        success: true,
        session: Some(response),
        message: "Admin login successful".to_string(),
    }))
}

// Student registration - no admin privileges
pub async fn student_register(
    State(session_state): State<SessionState>,
    Json(register_req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // Check if user already exists
    let existing_user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 OR student_id = $2")
            .bind(&register_req.email)
            .bind(&register_req.student_id)
            .fetch_optional(&session_state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Ok(Json(RegisterResponse {
            success: false,
            user_id: None,
            message: "User with this email or student ID already exists".to_string(),
        }));
    }

    // Hash password
    let password_hash = bcrypt::hash(&register_req.password, bcrypt::DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate QR secret
    let qr_secret = Uuid::new_v4().to_string();

    // Create user
    let user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (student_id, email, password_hash, first_name, last_name, qr_secret, department_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#
    )
    .bind(&register_req.student_id)
    .bind(&register_req.email)
    .bind(&password_hash)
    .bind(&register_req.first_name)
    .bind(&register_req.last_name)
    .bind(&qr_secret)
    .bind(register_req.department_id)
    .fetch_one(&session_state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse {
        success: true,
        user_id: Some(user_id),
        message: "User registered successfully".to_string(),
    }))
}

// Logout - revoke current session
#[debug_handler]
pub async fn logout(
    State(session_state): State<SessionState>,
    session_user: SessionUser,
    cookies: Cookies,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Delete session from Redis
    session_state
        .redis_store
        .delete_session(&session_user.session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clear session cookie
    let cookie = delete_session_cookie();
    cookies.add(cookie);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

// Get current user session info
pub async fn me(session_user: SessionUser) -> Result<Json<SessionUser>, StatusCode> {
    Ok(Json(session_user))
}

// Admin logout
#[debug_handler]
pub async fn admin_logout(
    State(session_state): State<SessionState>,
    admin_user: AdminUser,
    cookies: Cookies,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Delete session from Redis
    session_state
        .redis_store
        .delete_session(&admin_user.session_user.session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clear session cookie
    let cookie = delete_session_cookie();
    cookies.add(cookie);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Admin logged out successfully"
    })))
}

// Get current admin session info
pub async fn admin_me(admin_user: AdminUser) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "user": admin_user.session_user,
        "admin_role": admin_user.admin_role,
        "permissions": admin_user.session_user.permissions
    })))
}

// Get user's active sessions
pub async fn get_my_sessions(
    State(session_state): State<SessionState>,
    session_user: SessionUser,
) -> Result<Json<UserSessionsResponse>, StatusCode> {
    let sessions = session_state
        .redis_store
        .get_user_sessions(session_user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let session_infos: Vec<SessionInfo> = sessions
        .into_iter()
        .map(|s| SessionInfo {
            session_id: s.id,
            device_info: s.device_info,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            created_at: s.created_at,
            last_accessed: s.last_accessed,
            expires_at: s.expires_at,
        })
        .collect();

    let active_count = session_infos.len();

    Ok(Json(UserSessionsResponse {
        sessions: session_infos,
        active_count,
    }))
}

// Revoke a specific session (user can revoke their own sessions)
pub async fn revoke_my_session(
    State(session_state): State<SessionState>,
    session_user: SessionUser,
    Path(target_session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify the session belongs to the user
    let user_sessions = session_state
        .redis_store
        .get_user_sessions(session_user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let session_exists = user_sessions.iter().any(|s| s.id == target_session_id);

    if !session_exists {
        return Err(StatusCode::FORBIDDEN);
    }

    // Revoke the session
    let success = session_state
        .redis_store
        .revoke_session(&target_session_id, Some("User revoked".to_string()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if success {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Session revoked successfully"
        })))
    } else {
        Ok(Json(serde_json::json!({
            "success": false,
            "message": "Session not found"
        })))
    }
}

// Admin: Get all active sessions
pub async fn get_all_sessions(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SessionListResponse>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let session_ids = session_state
        .redis_store
        .get_active_sessions(Some(limit))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sessions = Vec::new();

    for session_id in &session_ids {
        if let Ok(Some(session)) = session_state.redis_store.get_session(session_id).await {
            // Get user info
            if let Ok(Some(user)) = get_user_by_id(&session_state, session.user_id).await {
                // Get admin role
                let admin_role = get_user_admin_role(&session_state, user.id)
                    .await
                    .ok()
                    .flatten();

                let session_info = AdminSessionInfo {
                    session_id: session.id,
                    user_id: user.id,
                    user_name: format!("{} {}", user.first_name, user.last_name),
                    student_id: user.student_id,
                    email: user.email,
                    admin_level: admin_role.as_ref().map(|r| r.admin_level.clone()),
                    faculty_name: None,    // TODO: Join with faculty table
                    department_name: None, // TODO: Join with department table
                    session_type: session.session_type,
                    login_method: session.login_method,
                    device_info: session.device_info,
                    ip_address: session.ip_address,
                    user_agent: session.user_agent,
                    created_at: session.created_at,
                    last_accessed: session.last_accessed,
                    expires_at: session.expires_at,
                    permissions: session.permissions,
                    sse_connections: session.sse_connections,
                    is_active: session.is_active,
                    recent_activities: session.activity_log,
                };

                sessions.push(session_info);
            }
        }
    }

    let total_count = session_state
        .redis_store
        .get_session_count()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionListResponse {
        sessions,
        total_count,
    }))
}

// Admin: Force logout user
pub async fn admin_revoke_session(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Path(session_id): Path<String>,
    Json(req): Json<SessionRevocationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reason = req.reason.unwrap_or_else(|| "Revoked by admin".to_string());

    let success = session_state
        .redis_store
        .revoke_session(&session_id, Some(reason))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if success {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Session revoked successfully"
        })))
    } else {
        Ok(Json(serde_json::json!({
            "success": false,
            "message": "Session not found"
        })))
    }
}

// Admin: Force logout all user sessions
pub async fn admin_revoke_user_sessions(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SessionRevocationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reason = req
        .reason
        .unwrap_or_else(|| "All sessions revoked by admin".to_string());

    let user_sessions = session_state
        .redis_store
        .get_user_sessions(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut revoked_count = 0;
    for session in user_sessions {
        if session_state
            .redis_store
            .revoke_session(&session.id, Some(reason.clone()))
            .await
            .unwrap_or(false)
        {
            revoked_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Revoked {} sessions", revoked_count),
        "revoked_count": revoked_count
    })))
}

// Extend current session
pub async fn extend_session(
    State(session_state): State<SessionState>,
    session_user: SessionUser,
    Json(params): Json<HashMap<String, Value>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hours = params.get("hours").and_then(|h| h.as_i64()).unwrap_or(24);

    let new_expiry = Utc::now() + chrono::Duration::hours(hours);

    let success = session_state
        .redis_store
        .extend_session(&session_user.session_id, new_expiry)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if success {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Session extended successfully",
            "expires_at": new_expiry
        })))
    } else {
        Ok(Json(serde_json::json!({
            "success": false,
            "message": "Failed to extend session"
        })))
    }
}

// Helper functions
async fn authenticate_user(
    session_state: &SessionState,
    email: &str,
    password: &str,
) -> Result<Option<User>, anyhow::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&session_state.db_pool)
        .await?;

    match user {
        Some(user) => {
            if bcrypt::verify(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

async fn authenticate_user_by_student_id(
    session_state: &SessionState,
    student_id: &str,
    password: &str,
) -> Result<Option<User>, anyhow::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE student_id = $1")
        .bind(student_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    match user {
        Some(user) => {
            if bcrypt::verify(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

async fn get_user_by_id(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<User>, anyhow::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    Ok(user)
}

async fn get_user_admin_role(
    session_state: &SessionState,
    user_id: Uuid,
) -> Result<Option<AdminRole>, anyhow::Error> {
    // Check if admin exists and is active (has non-empty permissions array)
    let admin_role = sqlx::query_as::<_, AdminRole>("SELECT * FROM admin_roles WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

    // If admin role exists, check if it's active (has permissions)
    match admin_role {
        Some(role) => {
            if role.permissions.is_empty() {
                // Admin is disabled (no permissions)
                Ok(None)
            } else {
                Ok(Some(role))
            }
        }
        None => Ok(None)
    }
}

async fn check_faculty_is_active(
    session_state: &SessionState,
    faculty_id: Option<Uuid>,
) -> Result<bool, anyhow::Error> {
    // If no faculty_id provided, consider it as inactive
    let faculty_id = match faculty_id {
        Some(id) => id,
        None => return Ok(false),
    };

    // Query faculty status from database
    let is_active = sqlx::query_scalar::<_, bool>(
        "SELECT status FROM faculties WHERE id = $1"
    )
    .bind(faculty_id)
    .fetch_optional(&session_state.db_pool)
    .await?;

    // Return false if faculty doesn't exist or is inactive
    Ok(is_active.unwrap_or(false))
}

async fn get_user_faculty_id(
    session_state: &SessionState, 
    user_id: Uuid,
) -> Result<Option<Uuid>, anyhow::Error> {
    // Get faculty_id through user's department
    let faculty_id = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT d.faculty_id 
        FROM users u
        INNER JOIN departments d ON u.department_id = d.id
        WHERE u.id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(&session_state.db_pool)
    .await?;

    Ok(faculty_id.flatten())
}

async fn store_session_metadata(
    session_state: &SessionState,
    session: &crate::models::session::Session,
) -> Result<(), anyhow::Error> {
    let device_info_json = serde_json::to_value(&session.device_info)?;
    let ip_addr_str: Option<String> = session.ip_address.clone();

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, device_info, ip_address, user_agent, created_at, last_accessed, expires_at, is_active)
        VALUES ($1, $2, $3, $4::inet, $5, $6, $7, $8, $9)
        ON CONFLICT (id) DO UPDATE SET
            last_accessed = $7,
            expires_at = $8,
            is_active = $9
        "#
    )
    .bind(&session.id)
    .bind(session.user_id)
    .bind(device_info_json)
    .bind(ip_addr_str)
    .bind(&session.user_agent)
    .bind(session.created_at)
    .bind(session.last_accessed)
    .bind(session.expires_at)
    .bind(session.is_active)
    .execute(&session_state.db_pool)
    .await?;

    Ok(())
}

fn build_session_user(
    user: &User,
    admin_role: &Option<AdminRole>,
    session_id: &str,
) -> SessionUser {
    let permissions = match admin_role {
        Some(role) => {
            let perms = crate::models::session::Permission::from_admin_level(
                &role.admin_level,
                role.faculty_id,
            );
            let mut perm_strings: Vec<String> =
                perms.into_iter().map(|p| format!("{:?}", p)).collect();
            perm_strings.extend(role.permissions.iter().cloned());
            perm_strings
        }
        None => vec!["ViewProfile".to_string(), "UpdateProfile".to_string()],
    };

    SessionUser {
        user_id: user.id,
        student_id: user.student_id.clone(),
        email: user.email.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        department_id: user.department_id,
        admin_role: admin_role.clone(),
        session_id: session_id.to_string(),
        permissions,
        faculty_id: admin_role.as_ref().and_then(|r| r.faculty_id),
    }
}
