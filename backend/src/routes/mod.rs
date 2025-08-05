use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{auth, faculty};
use crate::middleware::session::SessionState;

pub fn create_routes() -> Router<SessionState> {
    Router::new()
        // Authentication routes
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        
        // Faculty routes
        .route("/api/faculties", get(faculty::get_faculties))
        
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}