use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers::{auth, faculty, user, activity, admin, admin_session, sse};
use crate::middleware::session::SessionState;

pub fn create_routes() -> Router<SessionState> {
    Router::new()
        // Student Authentication routes
        .route("/api/auth/login", post(auth::student_login))
        .route("/api/auth/register", post(auth::student_register))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        
        // Admin Authentication routes
        .route("/api/admin/auth/login", post(auth::admin_login))
        .route("/api/admin/auth/logout", post(auth::admin_logout))
        .route("/api/admin/auth/me", get(auth::admin_me))
        
        // Faculty routes
        .route("/api/faculties", get(faculty::get_faculties))
        .route("/api/faculties/:id", get(faculty::get_faculty))
        .route("/api/faculties", post(faculty::create_faculty))
        .route("/api/faculties/:id", put(faculty::update_faculty))
        .route("/api/faculties/:id", delete(faculty::delete_faculty))
        
        // User routes
        .route("/api/users", get(user::get_users))
        .route("/api/users/:id", get(user::get_user))
        .route("/api/users", post(user::create_user))
        .route("/api/users/:id", put(user::update_user))
        .route("/api/users/:id", delete(user::delete_user))
        .route("/api/users/:id/qr", get(user::get_user_qr))
        
        // Activity routes
        .route("/api/activities", get(activity::get_activities))
        .route("/api/activities/:id", get(activity::get_activity))
        .route("/api/activities", post(activity::create_activity))
        .route("/api/activities/:id", put(activity::update_activity))
        .route("/api/activities/:id", delete(activity::delete_activity))
        .route("/api/activities/:id/participations", get(activity::get_activity_participations))
        .route("/api/activities/:id/participate", post(activity::participate))
        .route("/api/activities/:id/scan", post(activity::scan_qr))
        
        // Admin routes
        .route("/api/admin/dashboard", get(admin::get_dashboard))
        .route("/api/admin/users", get(admin::get_admin_users))
        .route("/api/admin/activities", get(admin::get_admin_activities))
        .route("/api/admin/sessions", get(admin::get_admin_sessions))
        
        // Admin session management
        .route("/api/admin/sessions", get(admin_session::get_sessions))
        .route("/api/admin/sessions/:id", delete(admin_session::revoke_session))
        .route("/api/admin/sessions/cleanup", post(admin_session::cleanup_expired))
        
        // SSE routes
        .route("/api/sse/events", get(sse::handle_sse))
        .route("/api/sse/admin", get(sse::handle_admin_sse))
        
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}