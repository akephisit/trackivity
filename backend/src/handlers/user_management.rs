use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::middleware::session::{SessionState, SuperAdminUser, FacultyAdminUser};
use crate::models::{
    admin_role::{AdminRole, AdminLevel},
    user::{User, UserResponse},
};

/// Get system-wide users (SuperAdmin only)
pub async fn get_system_users(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50);

    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0);

    let search = params.get("search").cloned();
    let faculty_id = params
        .get("faculty_id")
        .and_then(|f| Uuid::parse_str(f).ok());
    let department_id = params
        .get("department_id")
        .and_then(|d| Uuid::parse_str(d).ok());
    let include_admins = params
        .get("include_admins")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);

    let mut query = r#"
        SELECT 
            u.id,
            u.student_id,
            u.email,
            u.first_name,
            u.last_name,
            u.department_id,
            u.created_at,
            u.updated_at,
            d.name as department_name,
            d.code as department_code,
            f.id as faculty_id,
            f.name as faculty_name,
            f.code as faculty_code,
            ar.id as admin_role_id,
            ar.admin_level,
            ar.faculty_id as admin_faculty_id,
            ar.permissions,
            ar.created_at as role_created_at,
            ar.updated_at as role_updated_at,
            COALESCE(COUNT(p.id), 0) as activity_count,
            MAX(p.registered_at) as last_activity
        FROM users u
        LEFT JOIN departments d ON u.department_id = d.id
        LEFT JOIN faculties f ON d.faculty_id = f.id
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        LEFT JOIN participations p ON u.id = p.user_id
    "#.to_string();

    let mut count_query = r#"
        SELECT COUNT(DISTINCT u.id) 
        FROM users u
        LEFT JOIN departments d ON u.department_id = d.id
        LEFT JOIN faculties f ON d.faculty_id = f.id
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
    "#.to_string();

    let mut conditions = Vec::new();
    let mut param_count = 3;

    if let Some(_search_term) = &search {
        conditions.push(format!("(u.first_name ILIKE ${} OR u.last_name ILIKE ${} OR u.email ILIKE ${} OR u.student_id ILIKE ${} OR f.name ILIKE ${} OR d.name ILIKE ${})", 
            param_count, param_count, param_count, param_count, param_count, param_count));
        param_count += 1;
    }

    if faculty_id.is_some() {
        conditions.push(format!("f.id = ${}", param_count));
        param_count += 1;
    }

    if department_id.is_some() {
        conditions.push(format!("u.department_id = ${}", param_count));
        param_count += 1;
    }

    if !include_admins {
        conditions.push("ar.id IS NULL".to_string());
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        query.push_str(&where_clause);
        count_query.push_str(&where_clause);
    }

    query.push_str(" GROUP BY u.id, u.student_id, u.email, u.first_name, u.last_name, u.department_id, u.created_at, u.updated_at, d.name, d.code, f.id, f.name, f.code, ar.id, ar.admin_level, ar.faculty_id, ar.permissions, ar.created_at, ar.updated_at");
    query.push_str(" ORDER BY f.name NULLS LAST, d.name NULLS LAST, u.last_name, u.first_name LIMIT $1 OFFSET $2");

    let mut query_builder = sqlx::query(&query).bind(limit).bind(offset);
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        query_builder = query_builder.bind(search_pattern.clone());
        count_query_builder = count_query_builder.bind(search_pattern);
    }

    if let Some(fac_id) = faculty_id {
        query_builder = query_builder.bind(fac_id);
        count_query_builder = count_query_builder.bind(fac_id);
    }

    if let Some(dept_id) = department_id {
        query_builder = query_builder.bind(dept_id);
        count_query_builder = count_query_builder.bind(dept_id);
    }

    let users_result = query_builder.fetch_all(&session_state.db_pool).await;
    let total_count_result = count_query_builder.fetch_one(&session_state.db_pool).await;

    match (users_result, total_count_result) {
        (Ok(rows), Ok(total_count)) => {
            let mut users_with_details = Vec::new();

            for row in rows {
                let admin_role = match row.get::<Option<Uuid>, _>("admin_role_id") {
                    Some(role_id) => Some(AdminRole {
                        id: role_id,
                        user_id: row.get("id"),
                        admin_level: row
                            .get::<Option<AdminLevel>, _>("admin_level")
                            .unwrap_or(AdminLevel::RegularAdmin),
                        faculty_id: row.get::<Option<Uuid>, _>("admin_faculty_id"),
                        permissions: row
                            .get::<Option<Vec<String>>, _>("permissions")
                            .unwrap_or_else(|| vec![]),
                        created_at: row.get::<Option<DateTime<Utc>>, _>("role_created_at"),
                        updated_at: row.get::<Option<DateTime<Utc>>, _>("role_updated_at"),
                    }),
                    None => None,
                };

                let user_detail = json!({
                    "id": row.get::<Uuid, _>("id"),
                    "student_id": row.get::<String, _>("student_id"),
                    "email": row.get::<String, _>("email"),
                    "first_name": row.get::<String, _>("first_name"),
                    "last_name": row.get::<String, _>("last_name"),
                    "department_id": row.get::<Option<Uuid>, _>("department_id"),
                    "department_name": row.get::<Option<String>, _>("department_name"),
                    "department_code": row.get::<Option<String>, _>("department_code"),
                    "faculty_id": row.get::<Option<Uuid>, _>("faculty_id"),
                    "faculty_name": row.get::<Option<String>, _>("faculty_name"),
                    "faculty_code": row.get::<Option<String>, _>("faculty_code"),
                    "admin_role": admin_role,
                    "created_at": row.get::<Option<DateTime<Utc>>, _>("created_at"),
                    "updated_at": row.get::<Option<DateTime<Utc>>, _>("updated_at"),
                    "activity_count": row.get::<Option<i64>, _>("activity_count").unwrap_or(0),
                    "last_activity": row.get::<Option<DateTime<Utc>>, _>("last_activity"),
                    "is_admin": admin_role.is_some()
                });

                users_with_details.push(user_detail);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "users": users_with_details,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset,
                    "filters": {
                        "faculty_id": faculty_id,
                        "department_id": department_id,
                        "include_admins": include_admins,
                        "search": search
                    }
                },
                "message": "System users retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve system users"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Bulk user operations (SuperAdmin only)
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkUserOperationRequest {
    pub operation: String, // "transfer_department", "transfer_faculty", "activate", "deactivate"
    pub user_ids: Vec<Uuid>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

pub async fn bulk_user_operations(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
    Json(request): Json<BulkUserOperationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if request.user_ids.is_empty() {
        let error_response = json!({
            "status": "error",
            "message": "No user IDs provided"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let mut tx = match session_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to start transaction: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let mut results = Vec::new();

    match request.operation.as_str() {
        "transfer_department" => {
            let new_department_id = request.parameters
                .as_ref()
                .and_then(|p| p.get("department_id"))
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            if new_department_id.is_none() {
                let error_response = json!({
                    "status": "error",
                    "message": "department_id parameter is required for transfer_department operation"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }

            for user_id in &request.user_ids {
                let result = sqlx::query_as::<_, User>(
                    r#"
                    UPDATE users 
                    SET department_id = $2, updated_at = NOW()
                    WHERE id = $1
                    RETURNING id, student_id, email, password_hash, first_name, last_name, qr_secret, department_id, created_at, updated_at
                    "#
                )
                .bind(user_id)
                .bind(new_department_id)
                .fetch_one(&mut *tx)
                .await;

                match result {
                    Ok(user) => results.push(json!({
                        "user_id": user_id,
                        "status": "success",
                        "message": "Department transfer completed successfully",
                        "user": UserResponse::from(user)
                    })),
                    Err(e) => results.push(json!({
                        "user_id": user_id,
                        "status": "error",
                        "message": format!("Failed to transfer department: {}", e)
                    }))
                }
            }
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid bulk operation"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    if let Err(e) = tx.commit().await {
        let error_response = json!({
            "status": "error",
            "message": format!("Failed to commit bulk operation: {}", e)
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let successful_count = results.iter().filter(|r| r["status"] == "success").count();
    let failed_count = results.len() - successful_count;

    let response = json!({
        "status": "success",
        "data": {
            "operation": request.operation,
            "results": results,
            "summary": {
                "total_attempted": request.user_ids.len(),
                "successful": successful_count,
                "failed": failed_count
            }
        },
        "message": format!("Bulk {} operation completed: {} successful, {} failed", 
            request.operation, successful_count, failed_count)
    });

    Ok(Json(response))
}

/// Get faculty-scoped user statistics (FacultyAdmin can view their faculty, SuperAdmin can view any)
pub async fn get_faculty_user_statistics(
    State(session_state): State<SessionState>,
    faculty_admin: FacultyAdminUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get faculty_id from query parameter or use admin's faculty_id
    let faculty_id = match params.get("faculty_id").and_then(|f| Uuid::parse_str(f).ok()) {
        Some(requested_faculty_id) => {
            // Validate access to the requested faculty
            match faculty_admin.admin_role.admin_level {
                AdminLevel::SuperAdmin => requested_faculty_id, // SuperAdmin can access any faculty
                AdminLevel::FacultyAdmin | AdminLevel::RegularAdmin => {
                    // Faculty admins can only access their own faculty
                    match faculty_admin.faculty_id {
                        Some(admin_faculty_id) if admin_faculty_id == requested_faculty_id => {
                            requested_faculty_id
                        }
                        _ => {
                            let error_response = json!({
                                "status": "error",
                                "message": "Access denied: You can only view statistics for your own faculty"
                            });
                            return Err((StatusCode::FORBIDDEN, Json(error_response)));
                        }
                    }
                }
            }
        }
        None => {
            // No faculty_id provided, use admin's faculty_id if they have one
            match faculty_admin.faculty_id {
                Some(admin_faculty_id) => admin_faculty_id,
                None => {
                    let error_response = json!({
                        "status": "error",
                        "message": "No faculty_id provided and admin has no assigned faculty"
                    });
                    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
                }
            }
        }
    };

    // Get faculty statistics for the specific faculty
    let faculty_stats_result = sqlx::query(
        r#"
        SELECT 
            f.id as faculty_id,
            f.name as faculty_name,
            f.code as faculty_code,
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT ar.id) as admin_count,
            COUNT(DISTINCT d.id) as department_count,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '30 days' THEN u.id END) as new_users_30_days,
            COUNT(DISTINCT CASE WHEN p.registered_at >= NOW() - INTERVAL '30 days' THEN u.id END) as active_users_30_days,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '7 days' THEN u.id END) as new_users_7_days
        FROM faculties f
        LEFT JOIN departments d ON f.id = d.faculty_id
        LEFT JOIN users u ON d.id = u.department_id
        LEFT JOIN admin_roles ar ON (u.id = ar.user_id AND ar.faculty_id = f.id)
        LEFT JOIN participations p ON u.id = p.user_id
        WHERE f.id = $1
        GROUP BY f.id, f.name, f.code
        "#
    )
    .bind(faculty_id)
    .fetch_one(&session_state.db_pool)
    .await;

    // Get department statistics for the specific faculty
    let department_stats_result = sqlx::query(
        r#"
        SELECT 
            d.id as department_id,
            d.name as department_name,
            d.code as department_code,
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '30 days' THEN u.id END) as new_users_30_days,
            COUNT(DISTINCT CASE WHEN p.registered_at >= NOW() - INTERVAL '30 days' THEN u.id END) as active_users_30_days
        FROM departments d
        LEFT JOIN users u ON d.id = u.department_id
        LEFT JOIN participations p ON u.id = p.user_id
        WHERE d.faculty_id = $1
        GROUP BY d.id, d.name, d.code
        ORDER BY d.name
        "#
    )
    .bind(faculty_id)
    .fetch_all(&session_state.db_pool)
    .await;

    match (faculty_stats_result, department_stats_result) {
        (Ok(faculty_row), Ok(department_rows)) => {
            tracing::info!("Faculty stats query successful for faculty_id: {}", faculty_id);
            let faculty_stats = json!({
                "faculty_id": faculty_row.get::<Uuid, _>("faculty_id"),
                "faculty_name": faculty_row.get::<String, _>("faculty_name"),
                "faculty_code": faculty_row.get::<String, _>("faculty_code"),
                "total_users": faculty_row.get::<i64, _>("total_users"),
                "admin_count": faculty_row.get::<i64, _>("admin_count"),
                "department_count": faculty_row.get::<i64, _>("department_count"),
                "new_users_30_days": faculty_row.get::<i64, _>("new_users_30_days"),
                "active_users_30_days": faculty_row.get::<i64, _>("active_users_30_days"),
                "new_users_7_days": faculty_row.get::<i64, _>("new_users_7_days")
            });

            let department_stats: Vec<_> = department_rows
                .into_iter()
                .map(|row| json!({
                    "department_id": row.get::<Uuid, _>("department_id"),
                    "department_name": row.get::<String, _>("department_name"),
                    "department_code": row.get::<String, _>("department_code"),
                    "total_users": row.get::<i64, _>("total_users"),
                    "new_users_30_days": row.get::<i64, _>("new_users_30_days"),
                    "active_users_30_days": row.get::<i64, _>("active_users_30_days")
                }))
                .collect();

            // Build response in format expected by frontend (UserStats interface)
            let response = json!({
                "status": "success",
                "data": {
                    "total_users": faculty_stats["total_users"],
                    "active_users": faculty_stats["active_users_30_days"],
                    "inactive_users": faculty_stats["total_users"].as_i64().unwrap_or(0) - faculty_stats["active_users_30_days"].as_i64().unwrap_or(0),
                    "students": faculty_stats["total_users"], // For now, assuming most users are students
                    "faculty": 0, // Would need additional query to distinguish faculty users
                    "staff": 0,   // Would need additional query to distinguish staff users
                    "recent_registrations": faculty_stats["new_users_30_days"],
                    "faculty_breakdown": [{
                        "faculty_id": faculty_stats["faculty_id"],
                        "faculty_name": faculty_stats["faculty_name"],
                        "user_count": faculty_stats["total_users"]
                    }],
                    "department_breakdown": department_stats,
                    "faculty_stats": faculty_stats
                },
                "message": "Faculty user statistics retrieved successfully"
            });

            Ok(Json(response))
        }
        (Err(faculty_error), _) => {
            tracing::error!("Faculty stats query failed: {:?}", faculty_error);
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to retrieve faculty statistics: {}", faculty_error)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
        (_, Err(dept_error)) => {
            tracing::error!("Department stats query failed: {:?}", dept_error);
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to retrieve department statistics: {}", dept_error)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get user statistics by faculty and department (SuperAdmin only)
pub async fn get_user_statistics(
    State(session_state): State<SessionState>,
    _admin: SuperAdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get faculty statistics
    let faculty_stats_result = sqlx::query(
        r#"
        SELECT 
            f.id as faculty_id,
            f.name as faculty_name,
            f.code as faculty_code,
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT ar.id) as admin_count,
            COUNT(DISTINCT d.id) as department_count,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '30 days' THEN u.id END) as new_users_30_days,
            COUNT(DISTINCT CASE WHEN p.registered_at >= NOW() - INTERVAL '30 days' THEN u.id END) as active_users_30_days
        FROM faculties f
        LEFT JOIN departments d ON f.id = d.faculty_id
        LEFT JOIN users u ON d.id = u.department_id
        LEFT JOIN admin_roles ar ON (u.id = ar.user_id AND ar.faculty_id = f.id)
        LEFT JOIN participations p ON u.id = p.user_id
        GROUP BY f.id, f.name, f.code
        ORDER BY f.name
        "#
    )
    .fetch_all(&session_state.db_pool)
    .await;

    // Get department statistics
    let department_stats_result = sqlx::query(
        r#"
        SELECT 
            d.id as department_id,
            d.name as department_name,
            d.code as department_code,
            f.id as faculty_id,
            f.name as faculty_name,
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '30 days' THEN u.id END) as new_users_30_days,
            COUNT(DISTINCT CASE WHEN p.registered_at >= NOW() - INTERVAL '30 days' THEN u.id END) as active_users_30_days
        FROM departments d
        JOIN faculties f ON d.faculty_id = f.id
        LEFT JOIN users u ON d.id = u.department_id
        LEFT JOIN participations p ON u.id = p.user_id
        GROUP BY d.id, d.name, d.code, f.id, f.name
        ORDER BY f.name, d.name
        "#
    )
    .fetch_all(&session_state.db_pool)
    .await;

    // Get system-wide statistics
    let system_stats_result = sqlx::query(
        r#"
        SELECT 
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT ar.id) as total_admins,
            COUNT(DISTINCT f.id) as total_faculties,
            COUNT(DISTINCT d.id) as total_departments,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '30 days' THEN u.id END) as new_users_30_days,
            COUNT(DISTINCT CASE WHEN u.created_at >= NOW() - INTERVAL '7 days' THEN u.id END) as new_users_7_days,
            COUNT(DISTINCT CASE WHEN p.registered_at >= NOW() - INTERVAL '30 days' THEN u.id END) as active_users_30_days
        FROM users u
        LEFT JOIN admin_roles ar ON u.id = ar.user_id
        LEFT JOIN departments d ON u.department_id = d.id
        LEFT JOIN faculties f ON d.faculty_id = f.id
        LEFT JOIN participations p ON u.id = p.user_id
        "#
    )
    .fetch_one(&session_state.db_pool)
    .await;

    match (faculty_stats_result, department_stats_result, system_stats_result) {
        (Ok(faculty_rows), Ok(department_rows), Ok(system_row)) => {
            let faculty_stats: Vec<_> = faculty_rows
                .into_iter()
                .map(|row| json!({
                    "faculty_id": row.get::<Uuid, _>("faculty_id"),
                    "faculty_name": row.get::<String, _>("faculty_name"),
                    "faculty_code": row.get::<String, _>("faculty_code"),
                    "total_users": row.get::<i64, _>("total_users"),
                    "admin_count": row.get::<i64, _>("admin_count"),
                    "department_count": row.get::<i64, _>("department_count"),
                    "new_users_30_days": row.get::<i64, _>("new_users_30_days"),
                    "active_users_30_days": row.get::<i64, _>("active_users_30_days")
                }))
                .collect();

            let department_stats: Vec<_> = department_rows
                .into_iter()
                .map(|row| json!({
                    "department_id": row.get::<Uuid, _>("department_id"),
                    "department_name": row.get::<String, _>("department_name"),
                    "department_code": row.get::<String, _>("department_code"),
                    "faculty_id": row.get::<Uuid, _>("faculty_id"),
                    "faculty_name": row.get::<String, _>("faculty_name"),
                    "total_users": row.get::<i64, _>("total_users"),
                    "new_users_30_days": row.get::<i64, _>("new_users_30_days"),
                    "active_users_30_days": row.get::<i64, _>("active_users_30_days")
                }))
                .collect();

            let system_stats = json!({
                "total_users": system_row.get::<i64, _>("total_users"),
                "total_admins": system_row.get::<i64, _>("total_admins"),
                "total_faculties": system_row.get::<i64, _>("total_faculties"),
                "total_departments": system_row.get::<i64, _>("total_departments"),
                "new_users_30_days": system_row.get::<i64, _>("new_users_30_days"),
                "new_users_7_days": system_row.get::<i64, _>("new_users_7_days"),
                "active_users_30_days": system_row.get::<i64, _>("active_users_30_days")
            });

            let response = json!({
                "status": "success",
                "data": {
                    "system_stats": system_stats,
                    "faculty_stats": faculty_stats,
                    "department_stats": department_stats
                },
                "message": "User statistics retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve user statistics"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}