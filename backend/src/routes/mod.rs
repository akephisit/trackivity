use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::{
    activity, admin, admin_session, admin_session_mgmt, auth, faculty, sse, user,
};
use crate::middleware::session::SessionState;

pub fn create_routes() -> Router<SessionState> {
    Router::new()
        // Student Authentication routes
        .route("/api/auth/login", post(auth::student_login))
        .route("/api/auth/register", post(auth::student_register))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/sessions", get(auth::get_my_sessions))
        .route(
            "/api/auth/sessions/{session_id}",
            delete(auth::revoke_my_session),
        )
        .route("/api/auth/extend", post(auth::extend_session))
        // Admin Authentication routes
        .route("/api/admin/auth/login", post(auth::admin_login))
        .route("/api/admin/auth/logout", post(auth::admin_logout))
        .route("/api/admin/auth/me", get(auth::admin_me))
        // Faculty routes
        .route("/api/faculties", get(faculty::get_faculties))
        .route("/api/faculties/{id}", get(faculty::get_faculty))
        .route("/api/faculties", post(faculty::create_faculty))
        .route("/api/faculties/{id}", put(faculty::update_faculty))
        .route("/api/faculties/{id}", delete(faculty::delete_faculty))
        // User routes
        .route("/api/users", get(user::get_users))
        .route("/api/users/{id}", get(user::get_user))
        .route("/api/users", post(user::create_user))
        .route("/api/users/{id}", put(user::update_user))
        .route("/api/users/{id}", delete(user::delete_user))
        .route("/api/users/{id}/qr", get(user::get_user_qr))
        // Activity routes
        .route("/api/activities", get(activity::get_activities))
        .route("/api/activities/{id}", get(activity::get_activity))
        .route("/api/activities", post(activity::create_activity))
        .route("/api/activities/{id}", put(activity::update_activity))
        .route("/api/activities/{id}", delete(activity::delete_activity))
        .route(
            "/api/activities/{id}/participations",
            get(activity::get_activity_participations),
        )
        .route(
            "/api/activities/{id}/participate",
            post(activity::participate),
        )
        .route("/api/activities/{id}/scan", post(activity::scan_qr))
        // Admin routes
        .route("/api/admin/dashboard", get(admin::get_dashboard))
        .route("/api/admin/users", get(admin::get_admin_users))
        .route("/api/admin/activities", get(admin::get_admin_activities))
        .route("/api/admin/create", post(admin::create_admin))
        .route("/api/admin/roles/{id}/toggle-status", put(admin::toggle_admin_status))
        // Admin session management routes (Super Admin only)
        .route("/api/admin/sessions", get(auth::get_all_sessions))
        .route(
            "/api/admin/sessions/{session_id}",
            delete(auth::admin_revoke_session),
        )
        .route(
            "/api/admin/users/{user_id}/sessions",
            delete(auth::admin_revoke_user_sessions),
        )
        .route(
            "/api/admin/active-sessions",
            get(admin_session::get_active_admin_sessions),
        )
        // Enhanced admin session management routes
        .route(
            "/api/admin/session-management/active",
            get(admin_session_mgmt::get_active_admin_sessions),
        )
        .route(
            "/api/admin/session-management/monitor",
            get(admin_session_mgmt::get_session_monitor),
        )
        .route(
            "/api/admin/session-management/analytics",
            get(admin_session_mgmt::get_session_analytics),
        )
        .route(
            "/api/admin/session-management/force-logout/{session_id}",
            delete(admin_session_mgmt::force_logout_session),
        )
        .route(
            "/api/admin/session-management/batch-logout",
            post(admin_session_mgmt::batch_force_logout_sessions),
        )
        .route(
            "/api/admin/session-management/user/{user_id}/logout",
            post(admin_session_mgmt::force_logout_user_sessions),
        )
        .route(
            "/api/admin/session-management/faculty/{faculty_id}/logout",
            post(admin_session_mgmt::force_logout_faculty_sessions),
        )
        // SSE routes - session-based connections
        .route("/api/sse/{session_id}", get(sse::sse_handler))
        .route("/api/sse/notification", post(sse::send_notification))
        .route(
            "/api/admin/sse/notify/{permission}",
            post(sse::admin_send_notification_by_permission),
        )
        .route(
            "/api/admin/sse/force-logout/{session_id}",
            post(sse::admin_send_force_logout),
        )
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}
