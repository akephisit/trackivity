use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use crate::handlers::AppState;
use crate::models::{LoginRequest, CreateUser, UserResponse};
use crate::services::{AuthService, SessionService};
use crate::middleware::session::{create_session_cookie, delete_session_cookie};
use crate::utils::get_client_info;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub message: String,
}

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let auth_service = AuthService::new(state.database.clone(), state.config.bcrypt_cost);
    
    match auth_service.authenticate_user(credentials).await {
        Ok(Some(user)) => {
            let mut session_service = SessionService::new(state.redis.clone());
            let (ip_address, user_agent) = get_client_info();
            
            let session_id = auth_service
                .create_session(
                    &mut session_service,
                    user.id,
                    state.config.session_max_age,
                    ip_address,
                    user_agent,
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Set session cookie
            let cookie = create_session_cookie(&session_id, state.config.session_max_age);
            cookies.add(cookie);

            let response = LoginResponse {
                user: user.into(),
                session_id,
            };

            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(user_data): Json<CreateUser>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let auth_service = AuthService::new(state.database.clone(), state.config.bcrypt_cost);
    
    match auth_service.register_user(user_data).await {
        Ok(user) => {
            let response = RegisterResponse {
                user: user.into(),
                message: "User registered successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<StatusCode, StatusCode> {
    if let Some(cookie) = cookies.get("session_id") {
        let mut session_service = SessionService::new(state.redis.clone());
        let _ = session_service.delete_session(cookie.value()).await;
    }

    // Remove session cookie
    let cookie = delete_session_cookie();
    cookies.add(cookie);

    Ok(StatusCode::OK)
}

pub async fn me(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Json<UserResponse>, StatusCode> {
    // This endpoint requires authentication middleware
    // The user info will be available in request extensions
    // For now, we'll return a placeholder
    Err(StatusCode::NOT_IMPLEMENTED)
}