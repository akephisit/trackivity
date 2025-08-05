use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_type", rename_all = "snake_case")]
pub enum SubscriptionType {
    Basic,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_type: SubscriptionType,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscription {
    pub user_id: Uuid,
    pub subscription_type: SubscriptionType,
    pub duration_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscription {
    pub subscription_type: Option<SubscriptionType>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}
