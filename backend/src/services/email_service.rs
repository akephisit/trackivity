use anyhow::Result;
use serde_json::json;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, debug, instrument};

use crate::middleware::session::SessionState;
use crate::models::notifications::{EmailQueue, NotificationStatus};

/// SMTP Email Service
pub struct EmailService {
    session_state: SessionState,
    smtp_config: SmtpConfig,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
    pub timeout_seconds: u64,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_address: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_else(|_| "noreply@trackivity.com".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Trackivity System".to_string()),
            use_tls: std::env::var("SMTP_USE_TLS").unwrap_or_else(|_| "true".to_string()) == "true",
            timeout_seconds: std::env::var("SMTP_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}

impl EmailService {
    pub fn new(session_state: SessionState) -> Self {
        Self {
            session_state,
            smtp_config: SmtpConfig::default(),
        }
    }

    pub fn with_config(session_state: SessionState, smtp_config: SmtpConfig) -> Self {
        Self {
            session_state,
            smtp_config,
        }
    }

    /// Start email processing background task
    pub async fn start_email_processor(&self) {
        let session_state = self.session_state.clone();
        let smtp_config = self.smtp_config.clone();

        tokio::spawn(async move {
            email_processor_task(session_state, smtp_config).await;
        });

        info!("Email processor task started");
    }

    /// Send a single email immediately
    #[instrument(skip(self, email))]
    pub async fn send_email(&self, email: &EmailQueue) -> Result<()> {
        self.send_email_impl(email).await
    }

    /// Queue an email for later sending
    #[instrument(skip(self))]
    pub async fn queue_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        body_text: &str,
        body_html: Option<&str>,
        priority: i32,
        metadata: Option<serde_json::Value>,
    ) -> Result<uuid::Uuid> {
        let email_id = uuid::Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO email_queue 
            (id, to_email, to_name, subject, body_text, body_html, priority, status, scheduled_for, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
            "#
        )
        .bind(email_id)
        .bind(to_email)
        .bind(to_name)
        .bind(subject)
        .bind(body_text)
        .bind(body_html)
        .bind(priority)
        .bind(NotificationStatus::Pending)
        .bind(metadata.unwrap_or(json!({})))
        .execute(&self.session_state.db_pool)
        .await?;

        debug!("Queued email {} for {}", email_id, to_email);
        Ok(email_id)
    }

    /// Get email sending statistics
    pub async fn get_email_stats(&self) -> Result<EmailStats> {
        let stats_query = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE DATE(created_at) = CURRENT_DATE) as emails_sent_today,
                COUNT(*) FILTER (WHERE status = 'pending') as emails_pending,
                COUNT(*) FILTER (WHERE status = 'failed') as emails_failed,
                AVG(EXTRACT(EPOCH FROM (sent_at - created_at)) * 1000) FILTER (WHERE sent_at IS NOT NULL) as avg_send_time_ms,
                MAX(sent_at) as last_successful_send
            FROM email_queue
            WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
            "#
        )
        .fetch_one(&self.session_state.db_pool)
        .await?;

        let total_sent = stats_query.emails_sent_today.unwrap_or(0) as i32;
        let failed = stats_query.emails_failed.unwrap_or(0) as i32;
        let delivery_rate = if total_sent + failed > 0 {
            (total_sent as f64 / (total_sent + failed) as f64) * 100.0
        } else {
            0.0
        };

        Ok(EmailStats {
            emails_sent_today: total_sent,
            emails_pending: stats_query.emails_pending.unwrap_or(0) as i32,
            emails_failed: failed,
            bounce_rate: 100.0 - delivery_rate, // Simple bounce rate calculation
            delivery_rate,
            avg_send_time_ms: stats_query.avg_send_time_ms
                .map(|decimal| decimal.to_string().parse().unwrap_or(0.0))
                .unwrap_or(0.0),
            last_successful_send: stats_query.last_successful_send,
            smtp_status: "healthy".to_string(), // TODO: Implement SMTP health check
        })
    }

    // Internal email sending implementation
    #[instrument(skip(self, email))]
    async fn send_email_impl(&self, email: &EmailQueue) -> Result<()> {
        // For now, simulate email sending
        // In production, replace with actual SMTP implementation using lettre or similar crate
        debug!(
            "Sending email to {} with subject: {}",
            email.to_email, email.subject
        );

        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Simulate 95% success rate
        if rand::random::<f32>() < 0.05 {
            return Err(anyhow::anyhow!("Simulated SMTP send failure"));
        }

        // Log successful send
        info!(
            "Email sent successfully to {} (ID: {})",
            email.to_email, email.id
        );

        Ok(())
    }
}

/// Background task for processing email queue
#[instrument(skip(session_state, smtp_config))]
async fn email_processor_task(session_state: SessionState, smtp_config: SmtpConfig) {
    let mut interval = interval(Duration::from_secs(60)); // Process every minute
    info!("Started email processor background task");

    loop {
        interval.tick().await;

        match process_email_queue(&session_state, &smtp_config).await {
            Ok(processed_count) => {
                if processed_count > 0 {
                    debug!("Processed {} emails from queue", processed_count);
                }
            }
            Err(e) => {
                error!("Email processor error: {}", e);
            }
        }
    }
}

#[instrument(skip(session_state, smtp_config))]
async fn process_email_queue(
    session_state: &SessionState,
    smtp_config: &SmtpConfig,
) -> Result<usize> {
    // Get pending emails ordered by priority and scheduled time
    let pending_emails = sqlx::query_as::<_, EmailQueue>(
        "SELECT * FROM email_queue 
         WHERE status = $1 AND scheduled_for <= NOW() 
         ORDER BY priority DESC, scheduled_for ASC 
         LIMIT 50"
    )
    .bind(NotificationStatus::Pending)
    .fetch_all(&session_state.db_pool)
    .await?;

    let mut processed_count = 0;

    for email in pending_emails {
        // Mark as processing
        sqlx::query(
            "UPDATE email_queue SET updated_at = NOW() WHERE id = $1"
        )
        .bind(email.id)
        .execute(&session_state.db_pool)
        .await?;

        // Send email
        let send_result = send_email_smtp(&email, smtp_config).await;

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

                processed_count += 1;
            }
            Err(e) => {
                // Increment attempts and handle failure
                let new_attempts = email.attempts + 1;
                let new_status = if new_attempts >= email.max_attempts {
                    NotificationStatus::Failed
                } else {
                    NotificationStatus::Pending
                };

                // Reschedule for retry with exponential backoff
                let retry_delay = Duration::from_secs(60 * 2_u64.pow(new_attempts as u32).min(1440)); // Max 24 hours
                let next_retry = chrono::Utc::now() + chrono::Duration::from_std(retry_delay)?;

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

                warn!("Email send failed (attempt {}): {}", new_attempts, e);
            }
        }
    }

    Ok(processed_count)
}

// Placeholder for actual SMTP implementation
#[instrument(skip(email, _smtp_config))]
async fn send_email_smtp(email: &EmailQueue, _smtp_config: &SmtpConfig) -> Result<()> {
    // TODO: Implement actual SMTP sending using lettre crate
    // This is a placeholder implementation

    debug!(
        "SMTP: Sending email to {} with subject: {}",
        email.to_email, email.subject
    );

    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Simulate 90% success rate for SMTP
    if rand::random::<f32>() < 0.1 {
        return Err(anyhow::anyhow!("SMTP server connection failed"));
    }

    Ok(())
}

// Email statistics struct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailStats {
    pub emails_sent_today: i32,
    pub emails_pending: i32,
    pub emails_failed: i32,
    pub bounce_rate: f64,
    pub delivery_rate: f64,
    pub avg_send_time_ms: f64,
    pub last_successful_send: Option<chrono::DateTime<chrono::Utc>>,
    pub smtp_status: String,
}

// Email template builder utility
pub struct EmailTemplate {
    subject: String,
    body_text: String,
    body_html: Option<String>,
}

impl EmailTemplate {
    pub fn new() -> Self {
        Self {
            subject: String::new(),
            body_text: String::new(),
            body_html: None,
        }
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }

    pub fn body_text(mut self, body: &str) -> Self {
        self.body_text = body.to_string();
        self
    }

    pub fn body_html(mut self, body: &str) -> Self {
        self.body_html = Some(body.to_string());
        self
    }

    pub fn subscription_expiry_template(
        user_name: &str,
        days_until_expiry: i32,
        subscription_type: &str,
    ) -> Self {
        let subject = if days_until_expiry <= 1 {
            "URGENT: Subscription Expires Soon - Action Required"
        } else {
            "Subscription Expiry Notice"
        };

        let body_text = format!(
            r#"Dear {},

This is an automated notification regarding your Trackivity subscription.

Subscription Details:
- Type: {}
- Days until expiry: {}
- Status: {}

Action Required:
Please contact your system administrator to renew your subscription before it expires.

If you believe this is an error, please contact support immediately.

Best regards,
Trackivity System
"#,
            user_name,
            subscription_type,
            days_until_expiry,
            if days_until_expiry <= 1 { "CRITICAL" } else { "Warning" }
        );

        let body_html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Subscription Expiry Notice</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: {}; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background-color: #f9f9f9; }}
        .details {{ background-color: white; padding: 15px; border-left: 4px solid {}; }}
        .footer {{ text-align: center; padding: 20px; font-size: 12px; color: #666; }}
        .urgent {{ color: #dc3545; font-weight: bold; }}
        .warning {{ color: #ffc107; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Subscription Expiry Notice</h1>
        </div>
        <div class="content">
            <p>Dear {},</p>
            <p>This is an automated notification regarding your Trackivity subscription.</p>
            
            <div class="details">
                <h3>Subscription Details</h3>
                <p><strong>Type:</strong> {}</p>
                <p><strong>Days until expiry:</strong> <span class="{}">{}</span></p>
                <p><strong>Status:</strong> <span class="{}">{}</span></p>
            </div>
            
            <h3>Action Required</h3>
            <p>Please contact your system administrator to renew your subscription before it expires.</p>
            <p>If you believe this is an error, please contact support immediately.</p>
        </div>
        <div class="footer">
            <p>Best regards,<br>Trackivity System</p>
            <p>This is an automated message. Please do not reply to this email.</p>
        </div>
    </div>
</body>
</html>
            "#,
            if days_until_expiry <= 1 { "#dc3545" } else { "#ffc107" }, // Header color
            if days_until_expiry <= 1 { "#dc3545" } else { "#ffc107" }, // Border color
            user_name,
            subscription_type,
            if days_until_expiry <= 1 { "urgent" } else { "warning" },
            days_until_expiry,
            if days_until_expiry <= 1 { "urgent" } else { "warning" },
            if days_until_expiry <= 1 { "CRITICAL" } else { "Warning" }
        );

        Self {
            subject: subject.to_string(),
            body_text,
            body_html: Some(body_html),
        }
    }

    pub fn admin_alert_template(
        alert_type: &str,
        count: i32,
        details: &str,
    ) -> Self {
        let subject = format!("Admin Alert: {} ({} items)", alert_type, count);
        
        let body_text = format!(
            r#"Admin Alert Notification

Alert Type: {}
Count: {}
Details: {}

Please log into the Trackivity admin dashboard to review and take action.

Time: {}
System: Trackivity

This is an automated alert from your Trackivity system.
"#,
            alert_type,
            count,
            details,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let body_html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Admin Alert</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: #dc3545; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background-color: #f9f9f9; }}
        .alert-box {{ background-color: #fff3cd; border: 1px solid #ffeaa7; padding: 15px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚨 Admin Alert</h1>
        </div>
        <div class="content">
            <div class="alert-box">
                <h3>{}</h3>
                <p><strong>Count:</strong> {}</p>
                <p><strong>Details:</strong> {}</p>
                <p><strong>Time:</strong> {}</p>
            </div>
            <p>Please log into the Trackivity admin dashboard to review and take action.</p>
        </div>
    </div>
</body>
</html>
            "#,
            alert_type,
            count,
            details,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        Self {
            subject,
            body_text,
            body_html: Some(body_html),
        }
    }

    pub fn build(self) -> (String, String, Option<String>) {
        (self.subject, self.body_text, self.body_html)
    }
}