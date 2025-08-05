use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use bcrypt;
use chrono::{DateTime, Utc};
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;
// use image::Luma; // Removed unused import
use base64::{engine::general_purpose, Engine as _};

use crate::middleware::session::{AdminUser, SessionState};
use crate::models::session::SessionUser;
use crate::models::{
    admin_role::AdminRole,
    user::{User, UserResponse},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub student_id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithDetails {
    pub id: Uuid,
    pub student_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub faculty_name: Option<String>,
    pub admin_role: Option<AdminRole>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub activity_count: i64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrCodeResponse {
    pub qr_data: String,
    pub qr_image_base64: String,
    pub user_id: Uuid,
    pub student_id: String,
}

/// Get all users with pagination and filtering
pub async fn get_users(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
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
    let department_id = params
        .get("department_id")
        .and_then(|d| Uuid::parse_str(d).ok());

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
            f.name as faculty_name,
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
    "#
    .to_string();

    let mut count_query = r#"
        SELECT COUNT(DISTINCT u.id) 
        FROM users u
        LEFT JOIN departments d ON u.department_id = d.id
    "#
    .to_string();

    let mut conditions = Vec::new();
    let mut param_count = 3;

    if let Some(_search_term) = &search {
        conditions.push(format!("(u.first_name ILIKE ${} OR u.last_name ILIKE ${} OR u.email ILIKE ${} OR u.student_id ILIKE ${})", param_count, param_count, param_count, param_count));
        param_count += 1;
    }

    if department_id.is_some() {
        conditions.push(format!("u.department_id = ${}", param_count));
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        query.push_str(&where_clause);
        count_query.push_str(&where_clause);
    }

    query.push_str(" GROUP BY u.id, u.student_id, u.email, u.first_name, u.last_name, u.department_id, u.created_at, u.updated_at, d.name, f.name, ar.id, ar.admin_level, ar.faculty_id, ar.permissions, ar.created_at, ar.updated_at");
    query.push_str(" ORDER BY u.created_at DESC LIMIT $1 OFFSET $2");

    let mut query_builder = sqlx::query(&query).bind(limit).bind(offset);

    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

    if let Some(search_term) = &search {
        let search_pattern = format!("%{}%", search_term);
        query_builder = query_builder.bind(search_pattern.clone());
        count_query_builder = count_query_builder.bind(search_pattern);
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
                            .get::<Option<crate::models::admin_role::AdminLevel>, _>("admin_level")
                            .unwrap_or(crate::models::admin_role::AdminLevel::RegularAdmin),
                        faculty_id: row.get::<Option<Uuid>, _>("admin_faculty_id"),
                        permissions: row
                            .get::<Option<Vec<String>>, _>("permissions")
                            .unwrap_or_else(|| vec![]),
                        created_at: row.get::<Option<DateTime<Utc>>, _>("role_created_at"),
                        updated_at: row.get::<Option<DateTime<Utc>>, _>("role_updated_at"),
                    }),
                    None => None,
                };

                let user_detail = UserWithDetails {
                    id: row.get("id"),
                    student_id: row.get("student_id"),
                    email: row.get("email"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    department_id: row.get::<Option<Uuid>, _>("department_id"),
                    department_name: row.get::<Option<String>, _>("department_name"),
                    faculty_name: row.get::<Option<String>, _>("faculty_name"),
                    admin_role,
                    created_at: row.get::<Option<DateTime<Utc>>, _>("created_at"),
                    updated_at: row.get::<Option<DateTime<Utc>>, _>("updated_at"),
                    activity_count: row.get::<Option<i64>, _>("activity_count").unwrap_or(0),
                    last_activity: row.get::<Option<DateTime<Utc>>, _>("last_activity"),
                };

                users_with_details.push(user_detail);
            }

            let response = json!({
                "status": "success",
                "data": {
                    "users": users_with_details,
                    "total_count": total_count,
                    "limit": limit,
                    "offset": offset
                },
                "message": "Users retrieved successfully"
            });

            Ok(Json(response))
        }
        _ => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve users"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get user by ID with detailed information
pub async fn get_user(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query_result =
        sqlx::query_as::<_, crate::models::user::User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&session_state.db_pool)
            .await;

    match query_result {
        Ok(Some(row)) => {
            // Simple response for now, will add detailed info later
            let response = json!({
                "status": "success",
                "data": {
                    "id": row.id,
                    "student_id": row.student_id,
                    "email": row.email,
                    "first_name": row.first_name,
                    "last_name": row.last_name,
                    "department_id": row.department_id,
                    "created_at": row.created_at,
                    "updated_at": row.updated_at
                },
                "message": "User retrieved successfully"
            });

            Ok(Json(response))
        }
        Ok(None) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve user"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create new user
pub async fn create_user(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user already exists
    let existing_user = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1 OR student_id = $2",
    )
    .bind(&request.email)
    .bind(&request.student_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match existing_user {
        Ok(count) if count > 0 => {
            let error_response = json!({
                "status": "error",
                "message": "User with this email or student ID already exists"
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check existing user"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match bcrypt::hash(&request.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to hash password"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Generate QR secret
    let qr_secret = Uuid::new_v4().to_string();

    // Create user
    let create_result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (student_id, email, password_hash, first_name, last_name, qr_secret, department_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, student_id, email, password_hash, first_name, last_name, qr_secret, department_id, created_at, updated_at
        "#
    )
    .bind(&request.student_id)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(&qr_secret)
    .bind(request.department_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match create_result {
        Ok(user) => {
            let user_response = UserResponse::from(user);
            let response = json!({
                "status": "success",
                "data": user_response,
                "message": "User created successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to create user: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Update user
pub async fn update_user(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user exists
    let existing_user = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&session_state.db_pool)
        .await;

    match existing_user {
        Ok(0) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check user existence"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        _ => {}
    }

    // Build dynamic update query
    let mut query = "UPDATE users SET updated_at = NOW()".to_string();
    let mut params: Vec<Box<dyn std::fmt::Display + Send + Sync>> = vec![];
    let mut param_count = 1;

    if let Some(email) = &request.email {
        query.push_str(&format!(", email = ${}", param_count));
        params.push(Box::new(email.clone()));
        param_count += 1;
    }

    if let Some(first_name) = &request.first_name {
        query.push_str(&format!(", first_name = ${}", param_count));
        params.push(Box::new(first_name.clone()));
        param_count += 1;
    }

    if let Some(last_name) = &request.last_name {
        query.push_str(&format!(", last_name = ${}", param_count));
        params.push(Box::new(last_name.clone()));
        param_count += 1;
    }

    if let Some(department_id) = request.department_id {
        query.push_str(&format!(", department_id = ${}", param_count));
        params.push(Box::new(department_id));
        param_count += 1;
    }

    if let Some(password) = &request.password {
        let password_hash = match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
            Ok(hash) => hash,
            Err(_) => {
                let error_response = json!({
                    "status": "error",
                    "message": "Failed to hash password"
                });
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }
        };
        query.push_str(&format!(", password_hash = ${}", param_count));
        params.push(Box::new(password_hash));
        param_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING id, student_id, email, password_hash, first_name, last_name, qr_secret, department_id, created_at, updated_at", param_count));

    // Execute query with proper parameter binding
    let mut query_builder = sqlx::query_as::<_, User>(&query);

    if let Some(email) = &request.email {
        query_builder = query_builder.bind(email);
    }
    if let Some(first_name) = &request.first_name {
        query_builder = query_builder.bind(first_name);
    }
    if let Some(last_name) = &request.last_name {
        query_builder = query_builder.bind(last_name);
    }
    if let Some(department_id) = request.department_id {
        query_builder = query_builder.bind(department_id);
    }
    if let Some(password) = &request.password {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        query_builder = query_builder.bind(password_hash);
    }
    query_builder = query_builder.bind(user_id);

    match query_builder.fetch_one(&session_state.db_pool).await {
        Ok(user) => {
            let user_response = UserResponse::from(user);
            let response = json!({
                "status": "success",
                "data": user_response,
                "message": "User updated successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to update user: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Delete user
pub async fn delete_user(
    State(session_state): State<SessionState>,
    _admin: AdminUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let delete_result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&session_state.db_pool)
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response = json!({
                    "status": "error",
                    "message": "User not found"
                });
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            } else {
                let response = json!({
                    "status": "success",
                    "message": "User deleted successfully"
                });
                Ok(Json(response))
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to delete user: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get user QR code
pub async fn get_user_qr(
    State(session_state): State<SessionState>,
    user: SessionUser, // Allow both admin and regular users to access their own QR
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check if user is requesting their own QR or is an admin
    if user.user_id != user_id && !user.permissions.iter().any(|p| p.contains("ManageUsers")) {
        let error_response = json!({
            "status": "error",
            "message": "Access denied: You can only access your own QR code"
        });
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    let user_result = sqlx::query("SELECT id, student_id, qr_secret FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_one(&session_state.db_pool)
        .await;

    match user_result {
        Ok(user_data) => {
            // Create QR data (JSON with user info and secret)
            let qr_data = json!({
                "user_id": user_data.get::<Uuid, _>("id"),
                "student_id": user_data.get::<String, _>("student_id"),
                "secret": user_data.get::<String, _>("qr_secret"),
                "timestamp": chrono::Utc::now().timestamp()
            });

            let qr_data_string = qr_data.to_string();

            // Generate QR code
            let qr_code = match QrCode::with_error_correction_level(&qr_data_string, EcLevel::M) {
                Ok(code) => code,
                Err(_) => {
                    let error_response = json!({
                        "status": "error",
                        "message": "Failed to generate QR code"
                    });
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
                }
            };

            // Create image from QR code
            let image = qr_code
                .render::<char>()
                .quiet_zone(false)
                .dark_color(' ')
                .light_color('█')
                .build();

            // Convert to base64 (simplified version using string)
            let qr_image_base64 = general_purpose::STANDARD.encode(image.as_bytes());

            let response_data = QrCodeResponse {
                qr_data: qr_data_string,
                qr_image_base64: format!("data:image/png;base64,{}", qr_image_base64),
                user_id: user_data.get::<Uuid, _>("id"),
                student_id: user_data.get::<String, _>("student_id"),
            };

            let response = json!({
                "status": "success",
                "data": response_data,
                "message": "QR code generated successfully"
            });

            Ok(Json(response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve user"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
