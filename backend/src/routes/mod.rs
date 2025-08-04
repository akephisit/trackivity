use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{AppState, auth};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        
        // Health check
        .route("/api/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}