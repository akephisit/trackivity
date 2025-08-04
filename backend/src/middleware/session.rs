use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookie;
use tower_cookies::cookie::time::Duration;

use crate::handlers::AppState;

pub async fn session_cleanup_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // This middleware can be used for periodic session cleanup
    // For now, we'll just pass through
    Ok(next.run(request).await)
}

pub fn create_session_cookie(session_id: &str, max_age_seconds: i64) -> Cookie<'static> {
    Cookie::build(("session_id", session_id.to_owned()))
        .path("/")
        .max_age(Duration::seconds(max_age_seconds))
        .http_only(true)
        .secure(true) // Set to false for development if not using HTTPS
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .build()
}

pub fn delete_session_cookie() -> Cookie<'static> {
    Cookie::build(("session_id", ""))
        .path("/")
        .max_age(Duration::seconds(0))
        .http_only(true)
        .build()
}