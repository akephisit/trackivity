pub mod activity;
pub mod admin;
pub mod admin_session;
pub mod admin_session_mgmt;
pub mod auth;
pub mod faculty;
pub mod sse;
pub mod user;

// pub use user::*;
// pub use activity::*;
// pub use admin::*;

use crate::config::Config;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub redis: redis::aio::ConnectionManager,
    pub config: Config,
}
