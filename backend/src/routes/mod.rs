use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::{
    activity, admin, admin_session, admin_session_mgmt, auth, department, faculty, qr_activity, sse_enhanced, sse_api, user, user_management,
};
use crate::middleware::session::SessionState;

pub fn create_routes() -> Router<SessionState> {
    Router::new()
        // Bootstrap route (no auth required)
        .route("/api/admin/bootstrap", post(admin::bootstrap_admin))
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
        .route("/api/admin/faculties", get(faculty::get_all_faculties_admin))
        .route("/api/faculties/{id}", get(faculty::get_faculty))
        .route("/api/faculties", post(faculty::create_faculty))
        .route("/api/faculties/{id}", put(faculty::update_faculty))
        .route("/api/faculties/{id}", delete(faculty::delete_faculty))
        .route("/api/faculties/{id}/toggle-status", put(faculty::toggle_faculty_status))
        // Enhanced faculty management (SuperAdmin only)
        .route("/api/admin/faculties/stats", get(faculty::get_faculties_with_stats))
        .route("/api/admin/faculties/overview", get(faculty::get_faculty_overview))
        // Department routes
        .route("/api/faculties/{faculty_id}/departments", get(department::get_faculty_departments))
        .route("/api/faculties/{faculty_id}/departments/public", get(department::get_faculty_departments_public))
        .route("/api/faculties/{faculty_id}/departments", post(department::create_faculty_department))
        .route("/api/departments", get(department::get_all_departments_admin))
        .route("/api/departments/{id}", put(department::update_department))
        .route("/api/departments/{id}", delete(department::delete_department))
        .route("/api/departments/{id}/toggle-status", put(department::toggle_department_status))
        // Faculty-scoped admin operations
        .route("/api/faculties/{faculty_id}/admins", get(admin::get_faculty_admins))
        .route("/api/faculties/{faculty_id}/users", get(admin::get_faculty_users))
        .route("/api/faculties/{faculty_id}/admins", post(admin::create_faculty_admin))
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
        // Enhanced QR Code routes
        .route("/api/qr/generate", get(qr_activity::generate_user_qr))
        .route("/api/qr/refresh", post(qr_activity::refresh_qr_secret))
        .route("/api/activities/{id}/checkin", post(qr_activity::qr_checkin))
        .route("/api/admin/activities/assigned", get(qr_activity::get_assigned_activities))
        .route("/api/activities/{id}/participants", get(activity::get_activity_participations))
        // Admin routes
        .route("/api/admin/dashboard", get(admin::get_dashboard))
        .route("/api/admin/users", get(admin::get_admin_users))
        .route("/api/admin/activities", get(admin::get_admin_activities))
        .route("/api/admin/create", post(admin::create_admin))
        .route("/api/admin/roles/{id}/toggle-status", put(admin::toggle_admin_status))
        .route("/api/admin/roles/{id}", put(admin::update_admin_role))
        // Enhanced admin management routes (SuperAdmin only)
        .route("/api/admin/system-admins", get(admin::get_all_system_admins))
        .route("/api/admin/bulk-operations", post(admin::bulk_admin_operations))
        // Enhanced user management routes (SuperAdmin only)
        .route("/api/admin/system-users", get(user_management::get_system_users))
        .route("/api/admin/user-statistics", get(user_management::get_user_statistics))
        // Faculty-scoped user statistics (FacultyAdmin and SuperAdmin)
        .route("/api/admin/faculty-user-statistics", get(user_management::get_faculty_user_statistics))
        .route("/api/admin/user-bulk-operations", post(user_management::bulk_user_operations))
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
        // Enhanced SSE routes - session-based connections
        .route("/api/sse/{session_id}", get(sse_enhanced::sse_handler))
        .route("/api/sse/student/{session_id}", get(sse_enhanced::sse_student_handler))
        .route("/api/sse/admin/{session_id}", get(sse_enhanced::sse_admin_handler))
        
        // SSE API endpoints
        .route("/api/sse/notification", post(sse_api::send_notification))
        .route("/api/sse/activity/checked-in", post(sse_api::send_activity_checked_in))
        .route("/api/sse/activity/new", post(sse_api::send_new_activity_created))
        .route("/api/sse/subscription/expiry", post(sse_api::send_subscription_expiry_warning))
        .route("/api/sse/announcement", post(sse_api::send_system_announcement))
        
        // Enhanced admin SSE endpoints
        .route("/api/sse/admin/notify/{permission}", post(sse_api::admin_send_notification_by_permission))
        .route("/api/sse/admin/force-logout/{session_id}", post(sse_api::admin_send_force_logout))
        .route("/api/sse/admin/permission-update", post(sse_api::admin_send_permission_updated))
        .route("/api/sse/admin/promotion", post(sse_api::admin_send_promotion_notification))
        .route("/api/sse/admin/stats", get(sse_api::admin_get_sse_stats))
        .route("/api/sse/admin/cleanup", post(sse_api::admin_cleanup_connections))
        .route("/api/sse/admin/heartbeat", post(sse_api::admin_send_heartbeat))
        .route("/api/sse/admin/test-notification", post(sse_api::admin_send_test_notification))
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}
