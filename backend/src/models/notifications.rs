use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Notification Type Enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    SubscriptionExpiry,
    SystemAlert,
    AdminNotice,
    FacultyUpdate,
}

/// Notification Status Enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "snake_case")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Delivered,
}

/// Subscription Notification Model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionNotification {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub status: NotificationStatus,
    pub title: String,
    pub message: String,
    pub days_until_expiry: Option<i32>,
    pub sent_at: Option<DateTime<Utc>>,
    pub email_sent: bool,
    pub sse_sent: bool,
    pub admin_notified: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Email Queue Model for async email processing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailQueue {
    pub id: Uuid,
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub priority: i32, // 1=low, 2=normal, 3=high, 4=critical
    pub status: NotificationStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_for: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription Expiry Log Model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionExpiryLog {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub user_id: Uuid,
    pub days_until_expiry: i32,
    pub notification_sent: bool,
    pub admin_alerted: bool,
    pub check_timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Create Subscription Notification Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionNotification {
    pub subscription_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub days_until_expiry: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Email Template for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub template_name: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub variables: serde_json::Value,
}

/// Subscription Expiry Notification Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionExpiryNotificationData {
    pub user_name: String,
    pub user_email: String,
    pub faculty_name: Option<String>,
    pub subscription_type: String,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiry: i32,
    pub renewal_url: Option<String>,
    pub support_email: String,
}

/// SSE Notification Message for subscription expiry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseSubscriptionNotification {
    pub notification_id: Uuid,
    pub subscription_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub days_until_expiry: Option<i32>,
    pub severity: String, // info, warning, error, critical
    pub actions: Vec<NotificationAction>,
    pub expires_at: DateTime<Utc>,
    pub can_extend: bool,
    pub timestamp: DateTime<Utc>,
}

/// Notification Action for SSE messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub action_type: String, // dismiss, extend, contact_admin, renew
    pub label: String,
    pub url: Option<String>,
    pub method: Option<String>, // GET, POST
    pub data: Option<serde_json::Value>,
}

/// Admin Alert for subscription management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAlert {
    pub alert_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub faculty_id: Option<Uuid>,
    pub affected_users: Vec<Uuid>,
    pub recommendations: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub requires_action: bool,
    pub auto_resolve: bool,
}

/// Notification Preferences (future enhancement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub sse_enabled: bool,
    pub subscription_alerts: bool,
    pub system_alerts: bool,
    pub admin_notices: bool,
    pub alert_timing: i32, // days before expiry to start alerts
    pub digest_frequency: String, // daily, weekly, never
}

/// Notification Summary for Admin Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSummary {
    pub total_notifications: i32,
    pub pending_notifications: i32,
    pub failed_notifications: i32,
    pub subscription_expiry_alerts: i32,
    pub critical_alerts: i32,
    pub email_queue_size: i32,
    pub avg_delivery_time_seconds: Option<f64>,
    pub last_batch_processed: Option<DateTime<Utc>>,
}

/// Subscription Tracking Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionTrackingSummary {
    pub total_subscriptions: i32,
    pub active_subscriptions: i32,
    pub expiring_within_7_days: i32,
    pub expiring_within_1_day: i32,
    pub expired_subscriptions: i32,
    pub notifications_sent_today: i32,
    pub admin_alerts_pending: i32,
    pub auto_extensions_available: bool, // If system supports it
    pub last_check_timestamp: DateTime<Utc>,
}

/// Email Stats for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailStats {
    pub emails_sent_today: i32,
    pub emails_pending: i32,
    pub emails_failed: i32,
    pub bounce_rate: f64,
    pub delivery_rate: f64,
    pub avg_send_time_ms: f64,
    pub last_successful_send: Option<DateTime<Utc>>,
    pub smtp_status: String,
}

/// Background Task Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskStatus {
    pub task_name: String,
    pub is_running: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: i64,
    pub error_count: i64,
    pub avg_duration_ms: f64,
    pub last_error: Option<String>,
    pub last_error_time: Option<DateTime<Utc>>,
}