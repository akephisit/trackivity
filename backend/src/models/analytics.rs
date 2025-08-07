use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Use f64 for decimal arithmetic (simpler than BigDecimal for this use case)

/// Faculty Analytics model for tracking faculty-level statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FacultyAnalytics {
    pub id: Uuid,
    pub faculty_id: Uuid,
    pub total_students: i32,
    pub active_students: i32,
    pub total_activities: i32,
    pub completed_activities: i32,
    pub average_participation_rate: f64,
    pub monthly_activity_count: i32,
    pub department_count: i32,
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Department Analytics model for tracking department-level statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentAnalytics {
    pub id: Uuid,
    pub department_id: Uuid,
    pub faculty_id: Uuid,
    pub total_students: i32,
    pub active_students: i32,
    pub total_activities: i32,
    pub participation_rate: f64,
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// System Analytics model for Super Admin Dashboard
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemAnalytics {
    pub id: Uuid,
    pub total_faculties: i32,
    pub total_departments: i32,
    pub total_users: i32,
    pub total_activities: i32,
    pub active_subscriptions: i32,
    pub expiring_subscriptions_7d: i32,
    pub expiring_subscriptions_1d: i32,
    pub system_uptime_hours: Option<f64>,
    pub avg_response_time_ms: Option<f64>,
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Faculty Statistics Response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacultyStatsResponse {
    pub faculty_id: Uuid,
    pub faculty_name: String,
    pub faculty_code: String,
    pub total_students: i32,
    pub active_students: i32,
    pub total_activities: i32,
    pub completed_activities: i32,
    pub participation_rate: f64,
    pub monthly_activity_count: i32,
    pub department_count: i32,
    pub departments: Vec<DepartmentStatsResponse>,
    pub last_calculated: DateTime<Utc>,
}

/// Department Statistics Response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentStatsResponse {
    pub department_id: Uuid,
    pub department_name: String,
    pub department_code: String,
    pub total_students: i32,
    pub active_students: i32,
    pub total_activities: i32,
    pub participation_rate: f64,
    pub last_calculated: DateTime<Utc>,
}

/// System Overview Response for Super Admin Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverviewResponse {
    pub total_faculties: i32,
    pub total_departments: i32,
    pub total_users: i32,
    pub total_activities: i32,
    pub active_subscriptions: i32,
    pub expiring_subscriptions: SubscriptionExpiryStats,
    pub system_health: SystemHealthStats,
    pub recent_activity: Vec<RecentActivityItem>,
    pub last_updated: DateTime<Utc>,
}

/// Subscription Expiry Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionExpiryStats {
    pub expiring_in_7_days: i32,
    pub expiring_in_1_day: i32,
    pub expired_today: i32,
    pub total_active: i32,
    pub critical_alerts: i32,
}

/// System Health Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStats {
    pub uptime_hours: Option<f64>,
    pub avg_response_time_ms: Option<f64>,
    pub active_sessions: i32,
    pub sse_connections: i32,
    pub background_tasks_running: bool,
    pub database_status: String,
    pub redis_status: String,
}

/// Recent Activity Item for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivityItem {
    pub activity_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub faculty_id: Option<Uuid>,
    pub metadata: serde_json::Value,
}

/// Analytics Query Filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQueryFilters {
    pub faculty_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub include_inactive: bool,
}

/// Participation Trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationTrends {
    pub daily_participation: Vec<DailyParticipation>,
    pub weekly_participation: Vec<WeeklyParticipation>,
    pub monthly_participation: Vec<MonthlyParticipation>,
    pub top_performing_departments: Vec<DepartmentPerformance>,
    pub activity_completion_rates: Vec<ActivityCompletionRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyParticipation {
    pub date: chrono::NaiveDate,
    pub total_activities: i32,
    pub total_participants: i32,
    pub completion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyParticipation {
    pub week_start: chrono::NaiveDate,
    pub total_activities: i32,
    pub total_participants: i32,
    pub average_participation_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyParticipation {
    pub year: i32,
    pub month: u32,
    pub total_activities: i32,
    pub unique_participants: i32,
    pub participation_growth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentPerformance {
    pub department_id: Uuid,
    pub department_name: String,
    pub faculty_name: String,
    pub participation_rate: f64,
    pub activity_completion_rate: f64,
    pub total_students: i32,
    pub active_students: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCompletionRate {
    pub activity_id: Uuid,
    pub activity_title: String,
    pub total_registered: i32,
    pub total_completed: i32,
    pub completion_rate: f64,
    pub start_time: DateTime<Utc>,
}