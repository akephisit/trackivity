pub mod auth;
pub mod user;
pub mod activity;
pub mod admin;
pub mod sse;

pub use auth::*;
pub use user::*;
pub use activity::*;
pub use admin::*;
pub use sse::*;

use crate::config::Config;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub redis: redis::aio::ConnectionManager,
    pub config: Config,
}