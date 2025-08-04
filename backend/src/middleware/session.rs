use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::models::session::{SessionUser, SessionValidation, Permission};
use crate::models::admin_role::{AdminRole, AdminLevel};
use crate::models::user::User;
use crate::services::{RedisSessionStore, SessionConfig};

// Application state for session management
#[derive(Clone)]
pub struct SessionState {
    pub redis_store: Arc<RedisSessionStore>,
    pub db_pool: PgPool,
    pub config: SessionConfig,
}

// Session middleware for validating and extracting session info
pub async fn session_middleware(
    State(session_state): State<SessionState>,
    cookies: Cookies,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract session ID from cookie or header
    let session_id = extract_session_id(&cookies, &headers);
    
    if let Some(session_id) = session_id {
        // Validate session and get user data
        match validate_and_get_session_user(&session_state, &session_id).await {
            Ok(SessionValidation::Valid(session_user)) => {
                // Add session user to request extensions
                request.extensions_mut().insert(session_user);
                request.extensions_mut().insert(session_id);
            }
            Ok(SessionValidation::Expired) => {
                // Clear expired session cookie
                clear_session_cookie(&cookies);
            }
            Ok(SessionValidation::Revoked) => {
                // Session was revoked by admin
                clear_session_cookie(&cookies);
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(SessionValidation::Invalid) | Err(_) => {
                // Invalid session or database error
                clear_session_cookie(&cookies);
            }
        }
    }
    
    Ok(next.run(request).await)
}

// Permission-based route guard middleware
pub fn require_permission(
    permission: Permission,
) -> impl Fn(SessionUser, Request, Next) -> Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send> + Clone {
    move |session_user: SessionUser, request: Request, next: Next| {
        let permission = permission.clone();
        Box::new(async move {
            if has_permission(&session_user, &permission) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
}

// Admin level requirement middleware  
pub fn require_admin_level(
    required_level: AdminLevel,
) -> impl Fn(SessionUser, Request, Next) -> Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send> + Clone {
    move |session_user: SessionUser, request: Request, next: Next| {
        let required_level = required_level.clone();
        Box::new(async move {
            match &session_user.admin_role {
                Some(admin_role) => {
                    if is_admin_level_sufficient(&admin_role.admin_level, &required_level) {
                        Ok(next.run(request).await)
                    } else {
                        Err(StatusCode::FORBIDDEN)
                    }
                }
                None => Err(StatusCode::FORBIDDEN),
            }
        })
    }
}

// Faculty scope requirement middleware
pub fn require_faculty_scope(
    faculty_id: Option<Uuid>,
) -> impl Fn(SessionUser, Request, Next) -> Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send> + Clone {
    move |session_user: SessionUser, request: Request, next: Next| {
        let faculty_id = faculty_id;
        Box::new(async move {
            match (&session_user.admin_role, faculty_id) {
                (Some(admin_role), Some(required_faculty_id)) => {
                    match admin_role.admin_level {
                        AdminLevel::SuperAdmin => Ok(next.run(request).await),
                        AdminLevel::FacultyAdmin | AdminLevel::RegularAdmin => {
                            if session_user.faculty_id == Some(required_faculty_id) {
                                Ok(next.run(request).await)
                            } else {
                                Err(StatusCode::FORBIDDEN)
                            }
                        }
                    }
                }
                (Some(_admin_role), None) => {
                    // No specific faculty required, any admin can access
                    Ok(next.run(request).await)
                }
                _ => Err(StatusCode::FORBIDDEN),
            }
        })
    }
}

// Helper functions
fn extract_session_id(cookies: &Cookies, headers: &HeaderMap) -> Option<String> {
    // Try cookie first (web clients)
    if let Some(cookie) = cookies.get("session_id") {
        return Some(cookie.value().to_string());
    }
    
    // Try custom header (mobile apps)
    if let Some(header_value) = headers.get("X-Session-ID") {
        if let Ok(session_id) = header_value.to_str() {
            return Some(session_id.to_string());
        }
    }
    
    // Try Authorization header (Bearer token format)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    None
}

async fn validate_and_get_session_user(
    session_state: &SessionState,
    session_id: &str,
) -> Result<SessionValidation, anyhow::Error> {
    // Get session from Redis
    let session = match session_state.redis_store.get_session(session_id).await? {
        Some(session) => session,
        None => return Ok(SessionValidation::Invalid),
    };
    
    // Check if session is expired
    if session.expires_at <= chrono::Utc::now() {
        session_state.redis_store.delete_session(session_id).await?;
        return Ok(SessionValidation::Expired);
    }
    
    // Check if session is active
    if !session.is_active {
        return Ok(SessionValidation::Revoked);
    }
    
    // Get user data from database
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(session.user_id)
    .fetch_optional(&session_state.db_pool)
    .await?;
    
    let user = match user {
        Some(user) => user,
        None => return Ok(SessionValidation::Invalid),
    };
    
    // Get admin role if exists
    let admin_role = sqlx::query_as::<_, AdminRole>(
        "SELECT * FROM admin_roles WHERE user_id = $1"
    )
    .bind(user.id)
    .fetch_optional(&session_state.db_pool)
    .await?;
    
    // Build permissions list
    let permissions = match &admin_role {
        Some(role) => {
            let perms = Permission::from_admin_level(&role.admin_level, role.faculty_id);
            // Convert role permissions from strings to Permission enums if needed
            let mut perm_strings: Vec<String> = perms.into_iter().map(|p| format!("{:?}", p)).collect();
            perm_strings.extend(role.permissions.iter().cloned());
            perm_strings
        }
        None => vec!["ViewProfile".to_string(), "UpdateProfile".to_string()],
    };
    
    // Create session user
    let session_user = SessionUser {
        user_id: user.id,
        student_id: user.student_id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        department_id: user.department_id,
        admin_role: admin_role.clone(),
        session_id: session_id.to_string(),
        permissions,
        faculty_id: admin_role.as_ref().and_then(|r| r.faculty_id),
    };
    
    // Update session activity
    session_state.redis_store.update_session_activity(session_id).await?;
    
    Ok(SessionValidation::Valid(session_user))
}

fn clear_session_cookie(cookies: &Cookies) {
    let mut cookie = Cookie::new("session_id", "");
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookies.add(cookie);
}

fn has_permission(session_user: &SessionUser, permission: &Permission) -> bool {
    let permission_str = format!("{:?}", permission);
    session_user.permissions.contains(&permission_str)
}

fn is_admin_level_sufficient(user_level: &AdminLevel, required_level: &AdminLevel) -> bool {
    match (user_level, required_level) {
        (AdminLevel::SuperAdmin, _) => true,
        (AdminLevel::FacultyAdmin, AdminLevel::FacultyAdmin) => true,
        (AdminLevel::FacultyAdmin, AdminLevel::RegularAdmin) => true,
        (AdminLevel::RegularAdmin, AdminLevel::RegularAdmin) => true,
        _ => false,
    }
}

// Extractor for SessionUser from request
#[axum::async_trait]
impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SessionUser>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

// Optional SessionUser extractor
pub struct OptionalSessionUser(pub Option<SessionUser>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalSessionUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session_user = parts.extensions.get::<SessionUser>().cloned();
        Ok(OptionalSessionUser(session_user))
    }
}

// Admin-only extractor
pub struct AdminUser {
    pub session_user: SessionUser,
    pub admin_role: AdminRole,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session_user = parts
            .extensions
            .get::<SessionUser>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        match session_user.admin_role.clone() {
            Some(admin_role) => Ok(AdminUser {
                session_user,
                admin_role,
            }),
            None => Err(StatusCode::FORBIDDEN),
        }
    }
}

// Super admin only extractor
pub struct SuperAdminUser {
    pub session_user: SessionUser,
    pub admin_role: AdminRole,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for SuperAdminUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let admin_user = AdminUser::from_request_parts(parts, _state).await?;
        
        match admin_user.admin_role.admin_level {
            AdminLevel::SuperAdmin => Ok(SuperAdminUser {
                session_user: admin_user.session_user,
                admin_role: admin_user.admin_role,
            }),
            _ => Err(StatusCode::FORBIDDEN),
        }
    }
}

// Faculty admin extractor (Faculty Admin or Super Admin)
pub struct FacultyAdminUser {
    pub session_user: SessionUser,
    pub admin_role: AdminRole,
    pub faculty_id: Option<Uuid>,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for FacultyAdminUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let admin_user = AdminUser::from_request_parts(parts, _state).await?;
        
        match admin_user.admin_role.admin_level {
            AdminLevel::SuperAdmin | AdminLevel::FacultyAdmin => Ok(FacultyAdminUser {
                faculty_id: admin_user.admin_role.faculty_id,
                session_user: admin_user.session_user,
                admin_role: admin_user.admin_role,
            }),
            _ => Err(StatusCode::FORBIDDEN),
        }
    }
}

// Cookie helper functions (legacy support)
pub fn create_session_cookie(session_id: &str, max_age_seconds: i64) -> Cookie<'static> {
    Cookie::build(("session_id", session_id.to_owned()))
        .path("/")
        .max_age(tower_cookies::cookie::time::Duration::seconds(max_age_seconds))
        .http_only(true)
        .secure(true) // Set to false for development if not using HTTPS
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .build()
}

pub fn delete_session_cookie() -> Cookie<'static> {
    Cookie::build(("session_id", ""))
        .path("/")
        .max_age(tower_cookies::cookie::time::Duration::seconds(0))
        .http_only(true)
        .build()
}