use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::middleware::session::{AdminUser, SessionState};
use crate::models::session::SessionUser;
use crate::models::{
    activity::Activity,
    participation::{ParticipationStatus},
    user::User,
};
use crate::utils::qr::{generate_client_qr_data, validate_client_qr_data};

/// Request สำหรับ QR check-in
#[derive(Debug, Serialize, Deserialize)]
pub struct QrCheckInRequest {
    pub qr_data: String,
}

/// Response สำหรับ QR check-in
#[derive(Debug, Serialize, Deserialize)]
pub struct QrCheckInResponse {
    pub success: bool,
    pub message: String,
    pub user_name: String,
    pub student_id: String,
    pub participation_status: ParticipationStatus,
    pub checked_in_at: DateTime<Utc>,
}

/// Request สำหรับ activity assignment
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityAssignmentRequest {
    pub admin_id: Uuid,
    pub activity_ids: Vec<Uuid>,
}

/// สร้าง QR code data สำหรับ user
pub async fn generate_user_qr(
    State(session_state): State<SessionState>,
    user: SessionUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ดึงข้อมูล user และ qr_secret
    let user_data = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user.user_id)
    .fetch_one(&session_state.db_pool)
    .await;

    match user_data {
        Ok(user_data) => {
            match generate_client_qr_data(
                &user_data.id,
                &user_data.student_id,
                &user_data.qr_secret
            ) {
                Ok(qr_response) => {
                    // Render SVG for client to display proper QR without client-side lib
                    let qr_svg = match crate::utils::qr::render_qr_svg(&qr_response.qr_data, 256) {
                        Ok(svg) => Some(svg),
                        Err(_) => None,
                    };
                    let response = json!({
                        "status": "success",
                        "data": {
                            "qr_data": qr_response.qr_data,
                            "expires_at": qr_response.expires_at,
                            "qr_svg": qr_svg,
                            "user_info": {
                                "student_id": user_data.student_id,
                                "name": format!("{} {}", user_data.first_name, user_data.last_name)
                            }
                        },
                        "message": "QR code generated successfully"
                    });
                    Ok(Json(response))
                }
                Err(e) => {
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to generate QR code: {}", e)
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
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
                "message": "Failed to retrieve user data"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Refresh user's QR secret key
pub async fn refresh_qr_secret(
    State(session_state): State<SessionState>,
    user: SessionUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let new_secret = crate::utils::qr::generate_secret_key();

    let update_result = sqlx::query(
        "UPDATE users SET qr_secret = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&new_secret)
    .bind(user.user_id)
    .execute(&session_state.db_pool)
    .await;

    match update_result {
        Ok(_) => {
            let response = json!({
                "status": "success",
                "message": "QR secret refreshed successfully. Previous QR codes are now invalid."
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to refresh QR secret: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// QR check-in endpoint สำหรับ admins
pub async fn qr_checkin(
    State(session_state): State<SessionState>,
    admin: AdminUser, // Only admins can scan QR codes
    Path(activity_id): Path<Uuid>,
    Json(request): Json<QrCheckInRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ตรวจสอบว่า activity มีอยู่จริง
    let activity_check = sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities WHERE id = $1"
    )
    .bind(&activity_id)
    .fetch_one(&session_state.db_pool)
    .await;

    let activity = match activity_check {
        Ok(activity) => activity,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "Activity not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check activity"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Parse QR data
    let client_data: crate::utils::qr::ClientQrData = match serde_json::from_str(&request.qr_data) {
        Ok(data) => data,
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Invalid QR code format"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // ดึงข้อมูล user และตรวจสอบ QR secret
    let user_data = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(&client_data.user_id)
    .fetch_one(&session_state.db_pool)
    .await;

    let user_data = match user_data {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = json!({
                "status": "error",
                "message": "User not found in QR code"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve user data"
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // ตรวจสอบ QR code validity
    let validation_result = validate_client_qr_data(&request.qr_data, &user_data.qr_secret, 300); // 5 minutes
    
    if !validation_result.is_valid {
        let error_response = json!({
            "status": "error",
            "message": validation_result.error_message.unwrap_or("Invalid QR code".to_string())
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // ตรวจสอบว่า user ลงทะเบียนกิจกรรมนี้แล้วหรือยัง
    let existing_participation = sqlx::query(
        "SELECT id, status FROM participations WHERE user_id = $1 AND activity_id = $2"
    )
    .bind(&client_data.user_id)
    .bind(&activity_id)
    .fetch_optional(&session_state.db_pool)
    .await;

    match existing_participation {
        Ok(Some(participation)) => {
            // มีการลงทะเบียนแล้ว - อัปเดต status เป็น checked_in
            let status: String = participation.get("status");
            
            if status == "checked_in" {
                let error_response = json!({
                    "status": "error",
                    "message": "User already checked in"
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            let update_result = sqlx::query(
                "UPDATE participations SET status = 'checked_in', checked_in_at = NOW() WHERE id = $1 RETURNING checked_in_at"
            )
            .bind(participation.get::<Uuid, _>("id"))
            .fetch_one(&session_state.db_pool)
            .await;

            match update_result {
                Ok(updated_participation) => {
                    let checked_in_at: DateTime<Utc> = updated_participation.get("checked_in_at");
                    
                    let response_data = QrCheckInResponse {
                        success: true,
                        message: "Successfully checked in".to_string(),
                        user_name: format!("{} {}", user_data.first_name, user_data.last_name),
                        student_id: user_data.student_id.clone(),
                        participation_status: ParticipationStatus::CheckedIn,
                        checked_in_at,
                    };

                    let response = json!({
                        "status": "success",
                        "data": response_data,
                        "message": "User checked in successfully"
                    });

                    // Send SSE notification for activity check-in
                    if let Some(sse_manager) = &session_state.sse_manager {
                        let notification_data = serde_json::json!({
                            "activity_id": activity_id,
                            "activity_title": activity.title,
                            "user_name": format!("{} {}", user_data.first_name, user_data.last_name),
                            "student_id": user_data.student_id,
                            "action": "checked_in",
                            "checked_in_at": checked_in_at
                        });

                        use crate::handlers::sse_enhanced::*;
                        let sse_message = SseMessageBuilder::new(
                            SseEventType::ActivityCheckedIn,
                            notification_data,
                        )
                        .with_permissions(vec!["ManageActivities".to_string(), "ViewParticipations".to_string()])
                        .with_priority(MessagePriority::Normal);

                        let final_message = if let Some(faculty_id) = activity.faculty_id {
                            sse_message.to_faculty(faculty_id).build()
                        } else {
                            sse_message.build()
                        };

                        let _ = sse_manager.send_message(final_message).await;
                    }
                    
                    Ok(Json(response))
                }
                Err(e) => {
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to update participation: {}", e)
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            // ไม่มีการลงทะเบียน - สร้างใหม่และ check-in ทันที
            let create_result = sqlx::query(
                r#"
                INSERT INTO participations (user_id, activity_id, status, checked_in_at)
                VALUES ($1, $2, 'checked_in', NOW())
                RETURNING checked_in_at
                "#
            )
            .bind(&client_data.user_id)
            .bind(&activity_id)
            .fetch_one(&session_state.db_pool)
            .await;

            match create_result {
                Ok(new_participation) => {
                    let checked_in_at: DateTime<Utc> = new_participation.get("checked_in_at");

                    let response_data = QrCheckInResponse {
                        success: true,
                        message: "Successfully registered and checked in".to_string(),
                        user_name: format!("{} {}", user_data.first_name, user_data.last_name),
                        student_id: user_data.student_id.clone(),
                        participation_status: ParticipationStatus::CheckedIn,
                        checked_in_at,
                    };

                    let response = json!({
                        "status": "success",
                        "data": response_data,
                        "message": "User registered and checked in successfully"
                    });

                    // Send SSE notification for new registration and check-in
                    if let Some(sse_manager) = &session_state.sse_manager {
                        let notification_data = serde_json::json!({
                            "activity_id": activity_id,
                            "activity_title": activity.title,
                            "user_name": format!("{} {}", user_data.first_name, user_data.last_name),
                            "student_id": user_data.student_id,
                            "action": "registered_and_checked_in",
                            "checked_in_at": checked_in_at
                        });

                        use crate::handlers::sse_enhanced::*;
                        let sse_message = SseMessageBuilder::new(
                            SseEventType::ActivityCheckedIn,
                            notification_data,
                        )
                        .with_permissions(vec!["ManageActivities".to_string(), "ViewParticipations".to_string()])
                        .with_priority(MessagePriority::Normal);

                        let final_message = if let Some(faculty_id) = activity.faculty_id {
                            sse_message.to_faculty(faculty_id).build()
                        } else {
                            sse_message.build()
                        };

                        let _ = sse_manager.send_message(final_message).await;
                    }

                    Ok(Json(response))
                }
                Err(e) => {
                    let error_response = json!({
                        "status": "error",
                        "message": format!("Failed to create participation: {}", e)
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to check participation status"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// ดูกิจกรรมที่ admin ได้รับมอบหมาย
pub async fn get_assigned_activities(
    State(session_state): State<SessionState>,
    admin: AdminUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ตรวจสอบสิทธิ์ - ถ้าเป็น SuperAdmin ให้ดูได้ทั้งหมด
    let is_super_admin = admin.session_user.permissions.iter()
        .any(|p| p.contains("SuperAdmin") || p.contains("ManageAllActivities"));

    let activities_result = if is_super_admin {
        // SuperAdmin ดูได้ทั้งหมด
        sqlx::query(
            r#"
            SELECT 
                a.id,
                a.title,
                a.description,
                a.location,
                a.start_time,
                a.end_time,
                a.status,
                a.max_participants,
                f.name as faculty_name,
                d.name as department_name,
                COUNT(p.id) as current_participants
            FROM activities a
            LEFT JOIN faculties f ON a.faculty_id = f.id
            LEFT JOIN departments d ON a.department_id = d.id
            LEFT JOIN participations p ON a.id = p.activity_id
            WHERE a.status IN ('published', 'ongoing')
            GROUP BY a.id, a.title, a.description, a.location, a.start_time, a.end_time, a.status, a.max_participants, f.name, d.name
            ORDER BY a.start_time ASC
            "#
        )
        .fetch_all(&session_state.db_pool)
        .await
    } else {
        // Regular admin ดูเฉพาะที่สร้างเอง หรือในขอบเขตที่มีสิทธิ์
        sqlx::query(
            r#"
            SELECT 
                a.id,
                a.title,
                a.description,
                a.location,
                a.start_time,
                a.end_time,
                a.status,
                a.max_participants,
                f.name as faculty_name,
                d.name as department_name,
                COUNT(p.id) as current_participants
            FROM activities a
            LEFT JOIN faculties f ON a.faculty_id = f.id
            LEFT JOIN departments d ON a.department_id = d.id
            LEFT JOIN participations p ON a.id = p.activity_id
            WHERE a.status IN ('published', 'ongoing')
            AND (a.created_by = $1 OR a.faculty_id = $2)
            GROUP BY a.id, a.title, a.description, a.location, a.start_time, a.end_time, a.status, a.max_participants, f.name, d.name
            ORDER BY a.start_time ASC
            "#
        )
        .bind(admin.session_user.user_id)
        .bind(admin.admin_role.faculty_id.unwrap_or(Uuid::nil())) // Use nil UUID if no faculty
        .fetch_all(&session_state.db_pool)
        .await
    };

    match activities_result {
        Ok(rows) => {
            let activities: Vec<Value> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "title": row.get::<String, _>("title"),
                    "description": row.get::<String, _>("description"),
                    "location": row.get::<String, _>("location"),
                    "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                    "end_time": row.get::<DateTime<Utc>, _>("end_time"),
                    "status": row.get::<String, _>("status"),
                    "max_participants": row.get::<Option<i32>, _>("max_participants"),
                    "current_participants": row.get::<i64, _>("current_participants"),
                    "faculty_name": row.get::<Option<String>, _>("faculty_name"),
                    "department_name": row.get::<Option<String>, _>("department_name")
                })
            }).collect();

            let response = json!({
                "status": "success",
                "data": {
                    "activities": activities,
                    "total_count": activities.len(),
                    "admin_type": if is_super_admin { "super_admin" } else { "regular_admin" }
                },
                "message": "Assigned activities retrieved successfully"
            });

            Ok(Json(response))
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": "Failed to retrieve assigned activities"
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
