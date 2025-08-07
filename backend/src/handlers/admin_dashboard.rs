use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::middleware::session::SessionState;
use crate::models::{
    analytics::{SystemAnalytics, SystemOverviewResponse, SubscriptionExpiryStats, SystemHealthStats, RecentActivityItem},
    notifications::{NotificationSummary, SubscriptionTrackingSummary},
    subscription::SubscriptionType,
};

#[derive(Debug, Deserialize)]
pub struct DashboardQueryParams {
    pub include_detailed_stats: Option<bool>,
    pub timeframe_days: Option<i32>,
}

/// Get Super Admin Dashboard Overview
pub async fn get_dashboard_overview(
    State(session_state): State<SessionState>,
    Query(params): Query<DashboardQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Calculate system analytics
    let _ = sqlx::query("SELECT calculate_system_analytics()")
        .execute(&session_state.db_pool)
        .await;

    // Get latest system analytics
    let system_analytics_query = sqlx::query_as::<_, SystemAnalytics>(
        "SELECT * FROM system_analytics ORDER BY calculated_at DESC LIMIT 1"
    )
    .fetch_optional(&session_state.db_pool)
    .await;

    let system_analytics = match system_analytics_query {
        Ok(Some(analytics)) => analytics,
        Ok(None) => {
            // Create default analytics if none exist
            SystemAnalytics {
                id: Uuid::new_v4(),
                total_faculties: 0,
                total_departments: 0,
                total_users: 0,
                total_activities: 0,
                active_subscriptions: 0,
                expiring_subscriptions_7d: 0,
                expiring_subscriptions_1d: 0,
                system_uptime_hours: None,
                avg_response_time_ms: None,
                calculated_at: Utc::now(),
                created_at: Utc::now(),
            }
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch system analytics: {}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Get subscription expiry stats
    let subscription_expiry_stats = get_subscription_expiry_stats(&session_state).await?;

    // Get system health stats
    let system_health_stats = get_system_health_stats(&session_state).await?;

    // Get recent activity
    let recent_activity = if params.include_detailed_stats.unwrap_or(false) {
        get_recent_activity(&session_state, params.timeframe_days.unwrap_or(7)).await?
    } else {
        vec![]
    };

    let overview = SystemOverviewResponse {
        total_faculties: system_analytics.total_faculties,
        total_departments: system_analytics.total_departments,
        total_users: system_analytics.total_users,
        total_activities: system_analytics.total_activities,
        active_subscriptions: system_analytics.active_subscriptions,
        expiring_subscriptions: subscription_expiry_stats,
        system_health: system_health_stats,
        recent_activity,
        last_updated: system_analytics.calculated_at,
    };

    let response = json!({
        "status": "success",
        "data": overview
    });
    Ok(Json(response))
}

/// Get detailed system analytics
pub async fn get_system_analytics(
    State(session_state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get historical analytics (last 30 records)
    let analytics_query = sqlx::query_as::<_, SystemAnalytics>(
        "SELECT * FROM system_analytics ORDER BY calculated_at DESC LIMIT 30"
    )
    .fetch_all(&session_state.db_pool)
    .await;

    match analytics_query {
        Ok(analytics) => {
            let response = json!({
                "status": "success",
                "data": {
                    "analytics": analytics,
                    "trends": calculate_system_trends(&analytics)
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch analytics: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get all faculties with statistics
pub async fn get_faculties_stats(
    State(session_state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get all faculties with their analytics
    let faculties_query = sqlx::query!(
        r#"
        SELECT 
            f.id, f.name, f.code, f.description, f.created_at, f.updated_at,
            fa.total_students, fa.active_students, fa.total_activities,
            fa.completed_activities, fa.average_participation_rate,
            fa.monthly_activity_count, fa.department_count, fa.calculated_at
        FROM faculties f
        LEFT JOIN faculty_analytics fa ON f.id = fa.faculty_id
        ORDER BY f.name
        "#
    )
    .fetch_all(&session_state.db_pool)
    .await;

    match faculties_query {
        Ok(results) => {
            let faculties_stats: Vec<serde_json::Value> = results
                .into_iter()
                .map(|row| {
                    json!({
                        "faculty": {
                            "id": row.id,
                            "name": row.name,
                            "code": row.code,
                            "description": row.description,
                            "created_at": row.created_at,
                            "updated_at": row.updated_at
                        },
                        "stats": {
                            "total_students": row.total_students.unwrap_or(0),
                            "active_students": row.active_students.unwrap_or(0),
                            "total_activities": row.total_activities.unwrap_or(0),
                            "completed_activities": row.completed_activities.unwrap_or(0),
                            "participation_rate": row.average_participation_rate
                                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                                .unwrap_or(0.0),
                            "monthly_activity_count": row.monthly_activity_count.unwrap_or(0),
                            "department_count": row.department_count.unwrap_or(0),
                            "last_calculated": row.calculated_at
                        }
                    })
                })
                .collect();

            let response = json!({
                "status": "success",
                "data": {
                    "faculties": faculties_stats,
                    "total_count": faculties_stats.len()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch faculty statistics: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get subscription status overview
pub async fn get_subscriptions_overview(
    State(session_state): State<SessionState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Get subscription summary
    let subscription_summary = get_subscription_tracking_summary(&session_state).await?;
    
    // Get expiring subscriptions details
    let expiring_subscriptions_query = sqlx::query!(
        r#"
        SELECT 
            s.id, s.user_id, s.subscription_type as "subscription_type: SubscriptionType", s.expires_at, s.is_active,
            u.first_name, u.last_name, u.email, u.student_id,
            d.name as department_name, f.name as faculty_name
        FROM subscriptions s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN departments d ON u.department_id = d.id
        LEFT JOIN faculties f ON d.faculty_id = f.id
        WHERE s.is_active = true 
        AND s.expires_at > NOW() 
        AND s.expires_at <= NOW() + INTERVAL '7 days'
        ORDER BY s.expires_at ASC
        "#
    )
    .fetch_all(&session_state.db_pool)
    .await;

    let expiring_subscriptions = match expiring_subscriptions_query {
        Ok(results) => results
            .into_iter()
            .map(|row| {
                json!({
                    "subscription_id": row.id,
                    "user_id": row.user_id,
                    "subscription_type": row.subscription_type,
                    "expires_at": row.expires_at,
                    "days_until_expiry": (row.expires_at - Utc::now()).num_days(),
                    "user": {
                        "name": format!("{} {}", row.first_name, row.last_name),
                        "email": row.email,
                        "student_id": row.student_id
                    },
                    "department": row.department_name,
                    "faculty": row.faculty_name
                })
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            tracing::error!("Failed to fetch expiring subscriptions: {}", e);
            vec![]
        }
    };

    let response = json!({
        "status": "success",
        "data": {
            "summary": subscription_summary,
            "expiring_subscriptions": expiring_subscriptions,
            "notifications": get_notification_summary(&session_state).await?
        }
    });
    Ok(Json(response))
}

// Helper functions

async fn get_subscription_expiry_stats(
    session_state: &SessionState,
) -> Result<SubscriptionExpiryStats, (StatusCode, Json<serde_json::Value>)> {
    let expiry_stats_query = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE expires_at <= NOW() + INTERVAL '7 days' AND expires_at > NOW()) as expiring_7d,
            COUNT(*) FILTER (WHERE expires_at <= NOW() + INTERVAL '1 day' AND expires_at > NOW()) as expiring_1d,
            COUNT(*) FILTER (WHERE expires_at <= NOW()) as expired_today,
            COUNT(*) FILTER (WHERE is_active = true AND expires_at > NOW()) as total_active,
            COUNT(*) FILTER (WHERE expires_at <= NOW() + INTERVAL '1 day' AND expires_at > NOW()) as critical_alerts
        FROM subscriptions
        WHERE is_active = true
        "#
    )
    .fetch_one(&session_state.db_pool)
    .await;

    match expiry_stats_query {
        Ok(row) => Ok(SubscriptionExpiryStats {
            expiring_in_7_days: row.expiring_7d.unwrap_or(0) as i32,
            expiring_in_1_day: row.expiring_1d.unwrap_or(0) as i32,
            expired_today: row.expired_today.unwrap_or(0) as i32,
            total_active: row.total_active.unwrap_or(0) as i32,
            critical_alerts: row.critical_alerts.unwrap_or(0) as i32,
        }),
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch subscription expiry stats: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn get_system_health_stats(
    session_state: &SessionState,
) -> Result<SystemHealthStats, (StatusCode, Json<serde_json::Value>)> {
    // Get active sessions count
    let active_sessions = session_state
        .redis_store
        .get_active_sessions(None)
        .await
        .map(|sessions| sessions.len())
        .unwrap_or(0) as i32;

    // Check database status
    let db_status = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&session_state.db_pool)
        .await
    {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check Redis status - simple ping check
    let redis_status = match session_state.redis_store.get_active_sessions(Some(1)).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    Ok(SystemHealthStats {
        uptime_hours: None, // TODO: Implement system uptime tracking
        avg_response_time_ms: None, // TODO: Implement response time tracking
        active_sessions,
        sse_connections: 0, // TODO: Get from SSE manager
        background_tasks_running: true, // TODO: Check actual task status
        database_status: db_status.to_string(),
        redis_status: redis_status.to_string(),
    })
}

async fn get_recent_activity(
    session_state: &SessionState,
    days: i32,
) -> Result<Vec<RecentActivityItem>, (StatusCode, Json<serde_json::Value>)> {
    let activity_query = sqlx::query!(
        r#"
        SELECT 
            'activity_created' as activity_type,
            CONCAT('Activity "', title, '" was created') as description,
            created_at as timestamp,
            created_by as user_id,
            faculty_id,
            json_build_object('activity_id', id, 'status', status) as metadata
        FROM activities
        WHERE created_at >= NOW() - make_interval(days => $1)
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        days
    )
    .fetch_all(&session_state.db_pool)
    .await;

    match activity_query {
        Ok(results) => {
            let activities = results
                .into_iter()
                .map(|row| RecentActivityItem {
                    activity_type: row.activity_type.unwrap_or_default(),
                    description: row.description.unwrap_or_default(),
                    timestamp: row.timestamp.unwrap(),
                    user_id: Some(row.user_id),
                    faculty_id: row.faculty_id,
                    metadata: row.metadata.unwrap_or(json!({})),
                })
                .collect();
            Ok(activities)
        }
        Err(e) => {
            tracing::error!("Failed to fetch recent activity: {}", e);
            Ok(vec![])
        }
    }
}

async fn get_subscription_tracking_summary(
    session_state: &SessionState,
) -> Result<SubscriptionTrackingSummary, (StatusCode, Json<serde_json::Value>)> {
    let summary_query = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_subscriptions,
            COUNT(*) FILTER (WHERE is_active = true) as active_subscriptions,
            COUNT(*) FILTER (WHERE is_active = true AND expires_at <= NOW() + INTERVAL '7 days' AND expires_at > NOW()) as expiring_7d,
            COUNT(*) FILTER (WHERE is_active = true AND expires_at <= NOW() + INTERVAL '1 day' AND expires_at > NOW()) as expiring_1d,
            COUNT(*) FILTER (WHERE expires_at <= NOW()) as expired_subscriptions
        FROM subscriptions
        "#
    )
    .fetch_one(&session_state.db_pool)
    .await;

    // Get notifications sent today
    let notifications_today_query = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM subscription_notifications WHERE DATE(created_at) = CURRENT_DATE"
    )
    .fetch_one(&session_state.db_pool)
    .await;

    match (summary_query, notifications_today_query) {
        (Ok(row), Ok(notifications_count)) => Ok(SubscriptionTrackingSummary {
            total_subscriptions: row.total_subscriptions.unwrap_or(0) as i32,
            active_subscriptions: row.active_subscriptions.unwrap_or(0) as i32,
            expiring_within_7_days: row.expiring_7d.unwrap_or(0) as i32,
            expiring_within_1_day: row.expiring_1d.unwrap_or(0) as i32,
            expired_subscriptions: row.expired_subscriptions.unwrap_or(0) as i32,
            notifications_sent_today: notifications_count as i32,
            admin_alerts_pending: 0, // TODO: Implement admin alerts tracking
            auto_extensions_available: false, // Feature not implemented
            last_check_timestamp: Utc::now(),
        }),
        (Err(e), _) | (_, Err(e)) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch subscription summary: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

async fn get_notification_summary(
    session_state: &SessionState,
) -> Result<NotificationSummary, (StatusCode, Json<serde_json::Value>)> {
    let notification_stats_query = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_notifications,
            COUNT(*) FILTER (WHERE status = 'pending') as pending_notifications,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_notifications,
            COUNT(*) FILTER (WHERE notification_type = 'subscription_expiry') as subscription_alerts,
            COUNT(*) FILTER (WHERE days_until_expiry <= 1 AND status = 'pending') as critical_alerts
        FROM subscription_notifications
        WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
        "#
    )
    .fetch_one(&session_state.db_pool)
    .await;

    let email_queue_size_query = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM email_queue WHERE status IN ('pending', 'failed')"
    )
    .fetch_one(&session_state.db_pool)
    .await;

    match (notification_stats_query, email_queue_size_query) {
        (Ok(row), Ok(email_queue_size)) => Ok(NotificationSummary {
            total_notifications: row.total_notifications.unwrap_or(0) as i32,
            pending_notifications: row.pending_notifications.unwrap_or(0) as i32,
            failed_notifications: row.failed_notifications.unwrap_or(0) as i32,
            subscription_expiry_alerts: row.subscription_alerts.unwrap_or(0) as i32,
            critical_alerts: row.critical_alerts.unwrap_or(0) as i32,
            email_queue_size: email_queue_size as i32,
            avg_delivery_time_seconds: None, // TODO: Calculate from email_queue
            last_batch_processed: None, // TODO: Track from background tasks
        }),
        (Err(e), _) | (_, Err(e)) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to fetch notification summary: {}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

fn calculate_system_trends(analytics: &[SystemAnalytics]) -> serde_json::Value {
    if analytics.len() < 2 {
        return json!({
            "users_growth": 0.0,
            "activities_growth": 0.0,
            "subscriptions_growth": 0.0,
            "faculties_growth": 0.0
        });
    }

    let latest = &analytics[0];
    let previous = &analytics[1];

    let users_growth = calculate_growth_rate(previous.total_users, latest.total_users);
    let activities_growth = calculate_growth_rate(previous.total_activities, latest.total_activities);
    let subscriptions_growth = calculate_growth_rate(previous.active_subscriptions, latest.active_subscriptions);
    let faculties_growth = calculate_growth_rate(previous.total_faculties, latest.total_faculties);

    json!({
        "users_growth": users_growth,
        "activities_growth": activities_growth,
        "subscriptions_growth": subscriptions_growth,
        "faculties_growth": faculties_growth,
        "period_start": previous.calculated_at,
        "period_end": latest.calculated_at
    })
}

fn calculate_growth_rate(old_value: i32, new_value: i32) -> f64 {
    if old_value == 0 {
        return if new_value > 0 { 100.0 } else { 0.0 };
    }
    ((new_value - old_value) as f64 / old_value as f64) * 100.0
}