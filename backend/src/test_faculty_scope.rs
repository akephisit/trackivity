use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::middleware::session::{
    FacultyScopedAdminUser, AdminUser,
    has_faculty_access, validate_faculty_access, get_accessible_faculty_ids,
};
use crate::models::session::SessionUser;

/// Example handler using the new FacultyScopedAdminUser extractor
/// This automatically validates that the admin has access to the faculty
/// specified in the path parameter
pub async fn get_faculty_stats(
    faculty_scoped_admin: FacultyScopedAdminUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // The faculty_id is guaranteed to be accessible by this admin
    let faculty_id = faculty_scoped_admin.faculty_id;
    let admin_level = &faculty_scoped_admin.admin_role.admin_level;
    
    Ok(Json(json!({
        "status": "success",
        "data": {
            "faculty_id": faculty_id,
            "admin_level": format!("{:?}", admin_level),
            "message": "Faculty statistics retrieved successfully"
        }
    })))
}

/// Example handler using manual validation with helper functions
/// This provides more flexibility for complex validation logic
pub async fn update_faculty_settings(
    Path(faculty_id): Path<Uuid>,
    admin_user: AdminUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Manual validation using helper function
    validate_faculty_access(&admin_user.session_user, faculty_id)?;
    
    // Proceed with the operation
    Ok(Json(json!({
        "status": "success",
        "data": {
            "faculty_id": faculty_id,
            "message": "Faculty settings updated successfully"
        }
    })))
}

/// Example handler that lists accessible faculties for the current admin
pub async fn list_accessible_faculties(
    admin_user: AdminUser,
) -> Json<serde_json::Value> {
    let accessible_faculties = get_accessible_faculty_ids(&admin_user.session_user);
    
    match accessible_faculties {
        None => {
            // SuperAdmin - can access all faculties
            Json(json!({
                "status": "success",
                "data": {
                    "access_level": "all_faculties",
                    "message": "SuperAdmin can access all faculties"
                }
            }))
        }
        Some(faculty_ids) => {
            Json(json!({
                "status": "success",
                "data": {
                    "access_level": "specific_faculties",
                    "accessible_faculty_ids": faculty_ids,
                    "count": faculty_ids.len()
                }
            }))
        }
    }
}

/// Example handler demonstrating conditional logic based on faculty access
pub async fn handle_faculty_request(
    Path(faculty_id): Path<Uuid>,
    session_user: SessionUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if has_faculty_access(&session_user, faculty_id) {
        Ok(Json(json!({
            "status": "success",
            "data": {
                "faculty_id": faculty_id,
                "message": "Access granted to faculty"
            }
        })))
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Example router setup demonstrating the new faculty scope functionality
pub fn create_test_router() -> Router {
    Router::new()
        // Route using FacultyScopedAdminUser extractor
        // Automatically validates faculty access
        .route(
            "/api/admin/faculties/:faculty_id/stats",
            get(get_faculty_stats)
        )
        
        // Route using manual validation
        // Provides more flexibility for complex logic
        .route(
            "/api/admin/faculties/:faculty_id/settings",
            get(update_faculty_settings)
        )
        
        // Route for listing accessible faculties
        .route(
            "/api/admin/accessible-faculties",
            get(list_accessible_faculties)
        )
        
        // Route with conditional access checking
        .route(
            "/api/faculties/:faculty_id/info",
            get(handle_faculty_request)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_role::{AdminLevel, AdminRole};
    use crate::models::session::SessionUser;
    
    /// Helper function to create a test SessionUser
    fn create_test_session_user(
        admin_level: AdminLevel,
        faculty_id: Option<Uuid>,
    ) -> SessionUser {
        let admin_role = AdminRole {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            admin_level,
            faculty_id,
            permissions: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        SessionUser {
            user_id: Uuid::new_v4(),
            student_id: Some("TEST123".to_string()),
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            department_id: None,
            admin_role: Some(admin_role),
            session_id: "test_session".to_string(),
            permissions: vec![],
            faculty_id,
        }
    }
    
    #[test]
    fn test_super_admin_faculty_access() {
        let faculty_id = Uuid::new_v4();
        let session_user = create_test_session_user(AdminLevel::SuperAdmin, None);
        
        // SuperAdmin should have access to any faculty
        assert!(has_faculty_access(&session_user, faculty_id));
        assert!(validate_faculty_access(&session_user, faculty_id).is_ok());
    }
    
    #[test]
    fn test_faculty_admin_same_faculty_access() {
        let faculty_id = Uuid::new_v4();
        let session_user = create_test_session_user(AdminLevel::FacultyAdmin, Some(faculty_id));
        
        // FacultyAdmin should have access to their own faculty
        assert!(has_faculty_access(&session_user, faculty_id));
        assert!(validate_faculty_access(&session_user, faculty_id).is_ok());
    }
    
    #[test]
    fn test_faculty_admin_different_faculty_access() {
        let admin_faculty_id = Uuid::new_v4();
        let other_faculty_id = Uuid::new_v4();
        let session_user = create_test_session_user(AdminLevel::FacultyAdmin, Some(admin_faculty_id));
        
        // FacultyAdmin should NOT have access to different faculty
        assert!(!has_faculty_access(&session_user, other_faculty_id));
        assert!(validate_faculty_access(&session_user, other_faculty_id).is_err());
    }
    
    #[test]
    fn test_regular_user_faculty_access() {
        let faculty_id = Uuid::new_v4();
        let mut session_user = create_test_session_user(AdminLevel::RegularAdmin, Some(faculty_id));
        session_user.admin_role = None; // Regular user has no admin role
        
        // Regular user should NOT have faculty access
        assert!(!has_faculty_access(&session_user, faculty_id));
        assert!(validate_faculty_access(&session_user, faculty_id).is_err());
    }
    
    #[test]
    fn test_get_accessible_faculty_ids() {
        let faculty_id = Uuid::new_v4();
        
        // SuperAdmin
        let super_admin = create_test_session_user(AdminLevel::SuperAdmin, None);
        assert_eq!(get_accessible_faculty_ids(&super_admin), None);
        
        // FacultyAdmin
        let faculty_admin = create_test_session_user(AdminLevel::FacultyAdmin, Some(faculty_id));
        assert_eq!(get_accessible_faculty_ids(&faculty_admin), Some(vec![faculty_id]));
        
        // Regular user
        let mut regular_user = create_test_session_user(AdminLevel::RegularAdmin, None);
        regular_user.admin_role = None;
        assert_eq!(get_accessible_faculty_ids(&regular_user), Some(vec![]));
    }
}