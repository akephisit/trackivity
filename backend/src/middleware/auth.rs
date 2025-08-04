use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;

use crate::handlers::AppState;
use crate::models::{User, AdminRole};
use crate::services::{SessionService, AuthService};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: User,
    pub admin_role: Option<AdminRole>,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get session ID from cookie or header
    let session_id = get_session_id(&cookies, &request)?;
    
    // Get session from Redis
    let mut session_service = SessionService::new(state.redis.clone());
    let session = session_service
        .get_session(&session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match session {
        Some(session) => {
            // Get user from database
            let auth_service = AuthService::new(state.database.clone(), state.config.bcrypt_cost);
            let user = auth_service
                .get_user_by_id(session.user_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            match user {
                Some(user) => {
                    // Get admin role if exists
                    let admin_role = auth_service
                        .get_user_admin_role(user.id)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                    let auth_context = AuthContext { user, admin_role };
                    request.extensions_mut().insert(auth_context);
                    
                    Ok(next.run(request).await)
                }
                None => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match &auth_context.admin_role {
        Some(_) => Ok(next.run(request).await),
        None => Err(StatusCode::FORBIDDEN),
    }
}

pub async fn super_admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match &auth_context.admin_role {
        Some(admin_role) => {
            match admin_role.admin_level {
                crate::models::AdminLevel::SuperAdmin => Ok(next.run(request).await),
                _ => Err(StatusCode::FORBIDDEN),
            }
        }
        None => Err(StatusCode::FORBIDDEN),
    }
}

fn get_session_id(cookies: &Cookies, request: &Request) -> Result<String, StatusCode> {
    // Try to get session ID from cookie first
    if let Some(cookie) = cookies.get("session_id") {
        return Ok(cookie.value().to_string());
    }

    // Try to get from Authorization header
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Ok(auth_str[7..].to_string());
            }
        }
    }

    // Try to get from custom X-Session-ID header
    if let Some(session_header) = request.headers().get("X-Session-ID") {
        if let Ok(session_str) = session_header.to_str() {
            return Ok(session_str.to_string());
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}