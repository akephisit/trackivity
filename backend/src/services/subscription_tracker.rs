use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::time::interval;
use tracing::{info, warn, error, debug, instrument};

use crate::handlers::sse::{SseConnectionManager, SseMessage};
use crate::middleware::session::SessionState;
use crate::models::{
    notifications::{
        SubscriptionNotification, NotificationType, NotificationStatus,
        EmailQueue, SseSubscriptionNotification, NotificationAction
    },
    subscription::Subscription,
    user::User,
};

/// Subscription Expiry Tracking Service
pub struct SubscriptionTracker {
    session_state: SessionState,
    sse_manager: Arc<SseConnectionManager>,
}

impl SubscriptionTracker {
    pub fn new(session_state: SessionState, sse_manager: Arc<SseConnectionManager>) -> Self {
        Self {
            session_state,
            sse_manager,
        }
    }

    /// Start all subscription tracking background tasks
    pub async fn start_tracking_tasks(&self) {
        let session_state = self.session_state.clone();
        let sse_manager = self.sse_manager.clone();

        // Task 1: Check subscription expiry (runs every 6 hours)
        let expiry_session_state = session_state.clone();
        let expiry_sse_manager = sse_manager.clone();
        tokio::spawn(async move {
            subscription_expiry_check_task(expiry_session_state, expiry_sse_manager).await;
        });

        // Task 2: Process notification queue (runs every 5 minutes)
        let notification_session_state = session_state.clone();
        let notification_sse_manager = sse_manager.clone();
        tokio::spawn(async move {
            notification_processing_task(notification_session_state, notification_sse_manager).await;
        });

        // Task 3: Send email notifications (runs every 10 minutes)
        let email_session_state = session_state.clone();
        tokio::spawn(async move {
            email_notification_task(email_session_state).await;
        });

        // Task 4: Admin alert generation (runs every hour)
        let admin_session_state = session_state.clone();
        let admin_sse_manager = sse_manager.clone();
        tokio::spawn(async move {
            admin_alert_task(admin_session_state, admin_sse_manager).await;
        });

        // Task 5: Clean up old logs (runs daily)
        let cleanup_session_state = session_state.clone();
        tokio::spawn(async move {
            cleanup_task(cleanup_session_state).await;
        });

        info!("Subscription tracking tasks started successfully");
    }
}

/// Task 1: Check for expiring subscriptions and create notifications
#[instrument(skip(session_state, sse_manager))]
async fn subscription_expiry_check_task(
    session_state: SessionState,
    sse_manager: Arc<SseConnectionManager>,
) {
    let mut interval = interval(StdDuration::from_secs(6 * 3600)); // Every 6 hours
    info!("Started subscription expiry check task");

    loop {
        interval.tick().await;
        
        match check_expiring_subscriptions(&session_state, &sse_manager).await {
            Ok(checked_count) => {
                if checked_count > 0 {
                    info!("Checked {} subscriptions for expiry", checked_count);
                }
            }
            Err(e) => {
                error!("Failed to check expiring subscriptions: {}", e);
            }
        }
    }
}

/// Task 2: Process pending notifications
#[instrument(skip(session_state, sse_manager))]
async fn notification_processing_task(
    session_state: SessionState,
    sse_manager: Arc<SseConnectionManager>,
) {
    let mut interval = interval(StdDuration::from_secs(5 * 60)); // Every 5 minutes
    info!("Started notification processing task");

    loop {
        interval.tick().await;
        
        match process_pending_notifications(&session_state, &sse_manager).await {
            Ok(processed_count) => {
                if processed_count > 0 {
                    debug!("Processed {} pending notifications", processed_count);
                }
            }
            Err(e) => {
                error!("Failed to process notifications: {}", e);
            }
        }
    }
}

/// Task 3: Send email notifications
#[instrument(skip(session_state))]
async fn email_notification_task(session_state: SessionState) {
    let mut interval = interval(StdDuration::from_secs(10 * 60)); // Every 10 minutes
    info!("Started email notification task");

    loop {
        interval.tick().await;
        
        match send_pending_emails(&session_state).await {
            Ok(sent_count) => {
                if sent_count > 0 {
                    debug!("Sent {} email notifications", sent_count);
                }
            }
            Err(e) => {
                error!("Failed to send email notifications: {}", e);
            }
        }
    }
}

/// Task 4: Generate admin alerts for critical subscription issues
#[instrument(skip(session_state, sse_manager))]
async fn admin_alert_task(
    session_state: SessionState,
    sse_manager: Arc<SseConnectionManager>,
) {
    let mut interval = interval(StdDuration::from_secs(3600)); // Every hour
    info!("Started admin alert task");

    loop {
        interval.tick().await;
        
        match generate_admin_alerts(&session_state, &sse_manager).await {
            Ok(alert_count) => {
                if alert_count > 0 {
                    info!("Generated {} admin alerts", alert_count);
                }
            }
            Err(e) => {
                error!("Failed to generate admin alerts: {}", e);
            }
        }
    }
}

/// Task 5: Clean up old logs and notifications
#[instrument(skip(session_state))]
async fn cleanup_task(session_state: SessionState) {
    let mut interval = interval(StdDuration::from_secs(24 * 3600)); // Every 24 hours
    info!("Started cleanup task");

    loop {
        interval.tick().await;
        
        match cleanup_old_data(&session_state).await {
            Ok(cleaned_count) => {
                info!("Cleaned up {} old records", cleaned_count);
            }
            Err(e) => {
                error!("Failed to cleanup old data: {}", e);
            }
        }
    }
}

// Core implementation functions

#[instrument(skip(session_state, _sse_manager))]
async fn check_expiring_subscriptions(
    session_state: &SessionState,
    _sse_manager: &SseConnectionManager,
) -> Result<usize, anyhow::Error> {
    // Get subscriptions expiring within 7 days and 1 day
    let expiring_subscriptions = sqlx::query_as::<_, Subscription>(
        r#"
        SELECT s.* FROM subscriptions s
        WHERE s.is_active = true 
        AND s.expires_at > NOW() 
        AND s.expires_at <= NOW() + INTERVAL '7 days'
        ORDER BY s.expires_at ASC
        "#
    )
    .fetch_all(&session_state.db_pool)
    .await?;

    let mut checked_count = 0;

    for subscription in expiring_subscriptions {
        let days_until_expiry = (subscription.expires_at - Utc::now()).num_days();
        
        // Skip if already notified recently for this timeframe
        let recent_notification = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM subscription_expiry_log 
             WHERE subscription_id = $1 AND days_until_expiry = $2 
             AND check_timestamp >= NOW() - INTERVAL '12 hours')"
        )
        .bind(subscription.id)
        .bind(days_until_expiry as i32)
        .fetch_one(&session_state.db_pool)
        .await?;

        if recent_notification {
            continue;
        }

        // Log the check
        sqlx::query(
            "INSERT INTO subscription_expiry_log 
             (subscription_id, user_id, days_until_expiry, notification_sent, check_timestamp) 
             VALUES ($1, $2, $3, false, NOW())"
        )
        .bind(subscription.id)
        .bind(subscription.user_id)
        .bind(days_until_expiry as i32)
        .execute(&session_state.db_pool)
        .await?;

        // Create notifications based on expiry timeframe
        let should_notify = match days_until_expiry {
            0..=1 => true,  // Critical: expiring within 1 day
            2..=7 => true,  // Warning: expiring within 7 days
            _ => false,
        };

        if should_notify {
            create_expiry_notification(session_state, &subscription, days_until_expiry as i32).await?;
            checked_count += 1;
        }
    }

    Ok(checked_count)
}

#[instrument(skip(session_state))]
async fn create_expiry_notification(
    session_state: &SessionState,
    subscription: &Subscription,
    days_until_expiry: i32,
) -> Result<(), anyhow::Error> {
    let (title, message) = match days_until_expiry {
        0 => (
            "Subscription Expires Today".to_string(),
            "Your subscription expires today. Please contact your administrator for renewal.".to_string(),
        ),
        1 => (
            "Subscription Expires Tomorrow".to_string(),
            "Your subscription will expire tomorrow. Please contact your administrator for renewal.".to_string(),
        ),
        2..=7 => (
            format!("Subscription Expires in {} Days", days_until_expiry),
            format!("Your subscription will expire in {} days. Please contact your administrator for renewal.", days_until_expiry),
        ),
        _ => return Ok(()),
    };

    // Create notification record
    let _notification = sqlx::query_as::<_, SubscriptionNotification>(
        r#"
        INSERT INTO subscription_notifications 
        (subscription_id, user_id, notification_type, status, title, message, days_until_expiry, metadata) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(subscription.id)
    .bind(subscription.user_id)
    .bind(NotificationType::SubscriptionExpiry)
    .bind(NotificationStatus::Pending)
    .bind(&title)
    .bind(&message)
    .bind(days_until_expiry)
    .bind(json!({
        "subscription_type": subscription.subscription_type,
        "expires_at": subscription.expires_at,
        "severity": if days_until_expiry <= 1 { "critical" } else { "warning" }
    }))
    .fetch_one(&session_state.db_pool)
    .await?;

    debug!("Created expiry notification for user {} (expires in {} days)", 
           subscription.user_id, days_until_expiry);

    Ok(())
}

#[instrument(skip(session_state, sse_manager))]
async fn process_pending_notifications(
    session_state: &SessionState,
    sse_manager: &SseConnectionManager,
) -> Result<usize, anyhow::Error> {
    // Get pending notifications
    let pending_notifications = sqlx::query_as::<_, SubscriptionNotification>(
        "SELECT * FROM subscription_notifications 
         WHERE status = $1 
         ORDER BY created_at ASC 
         LIMIT 50"
    )
    .bind(NotificationStatus::Pending)
    .fetch_all(&session_state.db_pool)
    .await?;

    let mut processed_count = 0;

    for notification in pending_notifications {
        // Get user details
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(notification.user_id)
        .fetch_optional(&session_state.db_pool)
        .await?;

        let Some(user) = user else {
            warn!("User not found for notification {}", notification.id);
            continue;
        };

        // Send SSE notification
        let sse_success = send_sse_notification(sse_manager, &notification, &user).await;

        // Queue email notification
        let email_success = queue_email_notification(session_state, &notification, &user).await;

        // Update notification status
        let mut sse_sent = notification.sse_sent;
        let mut email_sent = notification.email_sent;
        let mut status = notification.status;

        if sse_success.is_ok() {
            sse_sent = true;
        }

        if email_success.is_ok() {
            email_sent = true;
        }

        // Mark as sent if at least one method succeeded
        if sse_sent || email_sent {
            status = NotificationStatus::Sent;
        }

        // Update the notification
        sqlx::query(
            "UPDATE subscription_notifications 
             SET status = $1, sse_sent = $2, email_sent = $3, sent_at = $4, updated_at = NOW()
             WHERE id = $5"
        )
        .bind(status)
        .bind(sse_sent)
        .bind(email_sent)
        .bind(if sse_sent || email_sent { Some(Utc::now()) } else { None })
        .bind(notification.id)
        .execute(&session_state.db_pool)
        .await?;

        processed_count += 1;
    }

    Ok(processed_count)
}

#[instrument(skip(sse_manager, notification, user))]
async fn send_sse_notification(
    sse_manager: &SseConnectionManager,
    notification: &SubscriptionNotification,
    user: &User,
) -> Result<(), anyhow::Error> {
    let severity = if notification.days_until_expiry.unwrap_or(999) <= 1 {
        "critical"
    } else {
        "warning"
    };

    let actions = vec![
        NotificationAction {
            action_type: "dismiss".to_string(),
            label: "Dismiss".to_string(),
            url: None,
            method: None,
            data: None,
        },
        NotificationAction {
            action_type: "contact_admin".to_string(),
            label: "Contact Admin".to_string(),
            url: Some("/contact".to_string()),
            method: Some("GET".to_string()),
            data: None,
        },
    ];

    let sse_notification = SseSubscriptionNotification {
        notification_id: notification.id,
        subscription_id: notification.subscription_id,
        notification_type: "subscription_expiry".to_string(),
        title: notification.title.clone(),
        message: notification.message.clone(),
        days_until_expiry: notification.days_until_expiry,
        severity: severity.to_string(),
        actions,
        expires_at: Utc::now() + Duration::days(notification.days_until_expiry.unwrap_or(0) as i64),
        can_extend: false, // Not implemented
        timestamp: Utc::now(),
    };

    let sse_message = SseMessage {
        event_type: "subscription_notification".to_string(),
        data: serde_json::to_value(sse_notification)?,
        timestamp: Utc::now(),
        target_permissions: None,
        target_user_id: Some(user.id),
        target_faculty_id: None,
    };

    sse_manager.send_to_user(user.id, sse_message).await
        .map_err(|e| anyhow::anyhow!("SSE send error: {}", e))?;

    Ok(())
}

#[instrument(skip(session_state, notification, user))]
async fn queue_email_notification(
    session_state: &SessionState,
    notification: &SubscriptionNotification,
    user: &User,
) -> Result<(), anyhow::Error> {
    let priority = if notification.days_until_expiry.unwrap_or(999) <= 1 {
        4 // Critical
    } else {
        2 // Normal
    };

    let body_text = format!(
        "Dear {} {},\n\n{}\n\nSubscription Details:\n- Type: {:?}\n- Days until expiry: {}\n\nPlease contact your administrator for assistance.\n\nBest regards,\nTrackivity Team",
        user.first_name,
        user.last_name,
        notification.message,
        notification.metadata.get("subscription_type").unwrap_or(&json!("Unknown")),
        notification.days_until_expiry.unwrap_or(0)
    );

    let body_html = format!(
        r#"
        <html>
        <body>
        <h2>Subscription Notification</h2>
        <p>Dear {} {},</p>
        <p>{}</p>
        <div style="background-color: #f8f9fa; padding: 15px; border-left: 4px solid #{}; margin: 20px 0;">
            <h3>Subscription Details</h3>
            <p><strong>Type:</strong> {:?}</p>
            <p><strong>Days until expiry:</strong> {}</p>
        </div>
        <p>Please contact your administrator for assistance.</p>
        <br>
        <p>Best regards,<br>Trackivity Team</p>
        </body>
        </html>
        "#,
        user.first_name,
        user.last_name,
        notification.message,
        if notification.days_until_expiry.unwrap_or(999) <= 1 { "dc3545" } else { "ffc107" },
        notification.metadata.get("subscription_type").unwrap_or(&json!("Unknown")),
        notification.days_until_expiry.unwrap_or(0)
    );

    // Insert into email queue
    sqlx::query(
        r#"
        INSERT INTO email_queue 
        (to_email, to_name, subject, body_text, body_html, priority, status, scheduled_for, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)
        "#
    )
    .bind(&user.email)
    .bind(format!("{} {}", user.first_name, user.last_name))
    .bind(&notification.title)
    .bind(body_text)
    .bind(body_html)
    .bind(priority)
    .bind(NotificationStatus::Pending)
    .bind(json!({
        "notification_id": notification.id,
        "user_id": user.id,
        "notification_type": "subscription_expiry"
    }))
    .execute(&session_state.db_pool)
    .await?;

    Ok(())
}

#[instrument(skip(session_state))]
async fn send_pending_emails(session_state: &SessionState) -> Result<usize, anyhow::Error> {
    // Get pending emails ordered by priority and scheduled time
    let pending_emails = sqlx::query_as::<_, EmailQueue>(
        "SELECT * FROM email_queue 
         WHERE status = $1 AND scheduled_for <= NOW() 
         ORDER BY priority DESC, scheduled_for ASC 
         LIMIT 20"
    )
    .bind(NotificationStatus::Pending)
    .fetch_all(&session_state.db_pool)
    .await?;

    let mut sent_count = 0;

    for email in pending_emails {
        // Simulate email sending (replace with actual SMTP implementation)
        let send_result = simulate_email_send(&email).await;

        match send_result {
            Ok(_) => {
                // Mark as sent
                sqlx::query(
                    "UPDATE email_queue 
                     SET status = $1, sent_at = NOW(), updated_at = NOW()
                     WHERE id = $2"
                )
                .bind(NotificationStatus::Sent)
                .bind(email.id)
                .execute(&session_state.db_pool)
                .await?;

                sent_count += 1;
            }
            Err(e) => {
                // Increment attempts and handle failure
                let new_attempts = email.attempts + 1;
                let new_status = if new_attempts >= email.max_attempts {
                    NotificationStatus::Failed
                } else {
                    NotificationStatus::Pending
                };

                // Reschedule for retry (exponential backoff)
                let next_retry = Utc::now() + Duration::minutes(2_i64.pow(new_attempts as u32));

                sqlx::query(
                    "UPDATE email_queue 
                     SET attempts = $1, status = $2, error_message = $3, 
                         scheduled_for = $4, updated_at = NOW()
                     WHERE id = $5"
                )
                .bind(new_attempts)
                .bind(&new_status)
                .bind(e.to_string())
                .bind(if new_status == NotificationStatus::Pending { Some(next_retry) } else { None })
                .bind(email.id)
                .execute(&session_state.db_pool)
                .await?;
            }
        }
    }

    Ok(sent_count)
}

// Simulate email sending - replace with actual SMTP implementation
async fn simulate_email_send(email: &EmailQueue) -> Result<(), anyhow::Error> {
    // For now, just simulate success/failure
    debug!("Simulating email send to: {}", email.to_email);
    
    // Simulate 95% success rate
    if rand::random::<f32>() < 0.95 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Simulated SMTP failure"))
    }
}

#[instrument(skip(session_state, sse_manager))]
async fn generate_admin_alerts(
    session_state: &SessionState,
    sse_manager: &SseConnectionManager,
) -> Result<usize, anyhow::Error> {
    // Get critical subscription statistics
    let critical_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE expires_at <= NOW() + INTERVAL '1 day' AND expires_at > NOW()) as expiring_1d,
            COUNT(*) FILTER (WHERE expires_at <= NOW() + INTERVAL '3 days' AND expires_at > NOW()) as expiring_3d,
            COUNT(*) FILTER (WHERE expires_at <= NOW()) as expired_today
        FROM subscriptions 
        WHERE is_active = true
        "#
    )
    .fetch_one(&session_state.db_pool)
    .await?;

    let mut alert_count = 0;

    // Alert for critical expiries (1 day)
    if critical_stats.expiring_1d.unwrap_or(0) > 0 {
        let alert_message = SseMessage {
            event_type: "admin_alert".to_string(),
            data: json!({
                "alert_type": "subscription_critical",
                "severity": "critical",
                "title": "Critical Subscription Expiries",
                "message": format!("{} subscriptions expire within 24 hours", critical_stats.expiring_1d.unwrap_or(0)),
                "count": critical_stats.expiring_1d.unwrap_or(0),
                "action_required": true,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
            target_permissions: Some(vec!["super_admin".to_string(), "faculty_admin".to_string()]),
            target_user_id: None,
            target_faculty_id: None,
        };

        sse_manager.broadcast_to_admins(alert_message).await
            .map_err(|e| anyhow::anyhow!("SSE broadcast error: {}", e))?;
        alert_count += 1;
    }

    // Alert for high volume of expiring subscriptions (3 days)
    if critical_stats.expiring_3d.unwrap_or(0) > 10 {
        let alert_message = SseMessage {
            event_type: "admin_alert".to_string(),
            data: json!({
                "alert_type": "subscription_warning",
                "severity": "warning",
                "title": "High Volume of Expiring Subscriptions",
                "message": format!("{} subscriptions expire within 3 days", critical_stats.expiring_3d.unwrap_or(0)),
                "count": critical_stats.expiring_3d.unwrap_or(0),
                "action_required": true,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
            target_permissions: Some(vec!["super_admin".to_string(), "faculty_admin".to_string()]),
            target_user_id: None,
            target_faculty_id: None,
        };

        sse_manager.broadcast_to_admins(alert_message).await
            .map_err(|e| anyhow::anyhow!("SSE broadcast error: {}", e))?;
        alert_count += 1;
    }

    Ok(alert_count)
}

#[instrument(skip(session_state))]
async fn cleanup_old_data(session_state: &SessionState) -> Result<usize, anyhow::Error> {
    let mut cleaned_count = 0;

    // Clean up old subscription expiry logs (older than 30 days)
    let expiry_logs_result = sqlx::query(
        "DELETE FROM subscription_expiry_log WHERE check_timestamp < NOW() - INTERVAL '30 days'"
    )
    .execute(&session_state.db_pool)
    .await?;
    cleaned_count += expiry_logs_result.rows_affected() as usize;

    // Clean up old sent notifications (older than 90 days)
    let old_notifications_result = sqlx::query(
        "DELETE FROM subscription_notifications 
         WHERE status = $1 AND sent_at < NOW() - INTERVAL '90 days'"
    )
    .bind(NotificationStatus::Sent)
    .execute(&session_state.db_pool)
    .await?;
    cleaned_count += old_notifications_result.rows_affected() as usize;

    // Clean up old sent emails (older than 30 days)
    let old_emails_result = sqlx::query(
        "DELETE FROM email_queue 
         WHERE status = $1 AND sent_at < NOW() - INTERVAL '30 days'"
    )
    .bind(NotificationStatus::Sent)
    .execute(&session_state.db_pool)
    .await?;
    cleaned_count += old_emails_result.rows_affected() as usize;

    Ok(cleaned_count)
}