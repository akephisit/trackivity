pub mod activity;
pub mod admin_role;
pub mod analytics;
pub mod department;
pub mod faculty;
pub mod notifications;
pub mod participation;
pub mod session;
pub mod subscription;
pub mod user;

pub use admin_role::*;
// Selective imports to avoid naming conflicts
pub use analytics::{
    SystemAnalytics, SystemOverviewResponse, SubscriptionExpiryStats, SystemHealthStats, RecentActivityItem,
    FacultyAnalytics, DepartmentAnalytics, FacultyStatsResponse, DepartmentStatsResponse
};
pub use notifications::{
    NotificationSummary, SubscriptionTrackingSummary, NotificationType, NotificationStatus,
    SubscriptionNotification, EmailQueue, SseSubscriptionNotification, NotificationAction,
    EmailStats, BackgroundTaskStatus, SubscriptionExpiryLog
};
pub use session::*;
pub use user::*;
