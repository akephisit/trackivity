mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
#[cfg(test)]
mod test_faculty_scope;

use axum::{http::{HeaderName, Method}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::database::Database;
use crate::handlers::sse_enhanced::{SseConnectionManager, SseConfig};
use crate::handlers::sse_tasks;
use crate::routes::create_routes;
use crate::services::background_tasks::BackgroundTaskManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trackivity=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize database
    let database = Database::new(&config.database_url).await?;

    // Run migrations if needed (auto-setup on first run)
    database.migrate_if_needed().await?;

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let _redis_manager = redis::aio::ConnectionManager::new(redis_client).await?;

    // Build Redis session store
    let redis_store = Arc::new(crate::services::RedisSessionStore::new(&config.redis_url)?);

    // Initialize enhanced SSE connection manager
    let redis_client_for_sse = redis::Client::open(config.redis_url.clone())?;
    let sse_config = SseConfig {
        max_connections_per_user: 5,
        heartbeat_interval: std::time::Duration::from_secs(30),
        connection_timeout: std::time::Duration::from_secs(300),
        rate_limit_per_minute: 100,
        channel_buffer_size: 1000,
        redis_pubsub_channel: "trackivity_sse".to_string(),
        enable_compression: false,
    };
    let sse_manager = Arc::new(SseConnectionManager::with_config(redis_client_for_sse, sse_config.clone()));

    // Build session state with SSE manager
    let session_state = crate::middleware::session::SessionState {
        redis_store: redis_store.clone(),
        db_pool: database.pool.clone(),
        config: crate::services::SessionConfig::default(),
        sse_manager: Some(sse_manager.clone()),
    };

    // Start background tasks including SSE tasks
    let background_task_manager =
        BackgroundTaskManager::new(session_state.clone(), sse_manager.clone());
    background_task_manager.start_all_tasks().await;

    // Start SSE-specific background tasks
    let sse_tasks = sse_tasks::spawn_sse_background_tasks(
        session_state.clone(),
        (*sse_manager).clone(),
        sse_config,
        redis::Client::open(config.redis_url.clone())?,
    );
    
    tracing::info!("Started {} SSE background tasks", sse_tasks.len());

    tracing::info!("Background tasks started successfully");

    // Build the application with session middleware
    let app = Router::new()
        .merge(create_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CookieManagerLayer::new())
                .layer(axum::middleware::from_fn_with_state(
                    session_state.clone(),
                    crate::middleware::session::session_middleware,
                ))
                .layer(
                    CorsLayer::new()
                        .allow_origin([
                            "http://localhost:5173".parse().unwrap(),
                            "http://localhost:5174".parse().unwrap(),
                            "http://localhost:3000".parse().unwrap(),
                            "http://127.0.0.1:5173".parse().unwrap(),
                            "http://127.0.0.1:5174".parse().unwrap()
                        ])
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::DELETE,
                            Method::OPTIONS
                        ])
                        .allow_headers([
                            HeaderName::from_static("content-type"),
                            HeaderName::from_static("authorization"), 
                            HeaderName::from_static("x-session-id"),
                            HeaderName::from_static("x-timezone"),
                            HeaderName::from_static("x-screen-resolution"),
                            HeaderName::from_static("accept-language")
                        ])
                        .allow_credentials(true),
                ),
        )
        .with_state(session_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting Trackivity server on {}", addr);
    tracing::info!("Session-based authentication enabled");
    tracing::info!("Redis session store configured");
    tracing::info!("SSE connections enabled");
    tracing::info!("Background cleanup tasks running");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
