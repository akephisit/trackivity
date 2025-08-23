mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use axum::{
    http::{HeaderName, HeaderValue, Method, Request, Uri},
    middleware::Next,
    response::Response,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::database::Database;
use crate::routes::create_routes;
use crate::services::background_tasks::BackgroundTaskManager;

// Middleware to normalize URIs and fix double slash issues
async fn normalize_uri_middleware(
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let original_uri = request.uri().clone();
    let path = original_uri.path();
    
    // Normalize double slashes in path
    if path.contains("//") {
        let normalized_path = if path.starts_with("//") {
            // For paths like //api/admin/auth/me -> /api/admin/auth/me
            path.chars().skip(1).collect::<String>()
        } else {
            // For paths like /api//admin/auth/me -> /api/admin/auth/me
            path.replace("//", "/")
        };
        
        if let Ok(new_uri) = normalized_path.parse::<Uri>() {
            *request.uri_mut() = new_uri;
        }
    }
    
    next.run(request).await
}

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

    // Create database and run migrations (auto-setup on first run)
    database.create_and_migrate(&config.database_url).await?;

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let _redis_manager = redis::aio::ConnectionManager::new(redis_client).await?;

    // Build Redis session store
    let redis_store = Arc::new(crate::services::RedisSessionStore::new(&config.redis_url)?);

    // Build session state
    let session_state = crate::middleware::session::SessionState {
        redis_store: redis_store.clone(),
        db_pool: database.pool.clone(),
        config: crate::services::SessionConfig::default(),
    };
    // Start background tasks
    let background_task_manager = BackgroundTaskManager::new(session_state.clone());
    background_task_manager.start_all_tasks().await;

    // Build CORS allowed origins from env or defaults
    let allowed_origins: Vec<HeaderValue> = {
        let env_val = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        let mut list: Vec<HeaderValue> = Vec::new();
        if !env_val.trim().is_empty() {
            for o in env_val.split(',') {
                let s = o.trim().trim_end_matches('/');
                if s.is_empty() { continue; }
                if let Ok(v) = s.parse() { list.push(v); }
            }
        }
        if list.is_empty() {
            // defaults for local dev
            list = vec![
                "http://localhost:5173".parse().unwrap(),
                "http://localhost:5174".parse().unwrap(),
                "http://localhost:3000".parse().unwrap(),
                "http://127.0.0.1:5173".parse().unwrap(),
                "http://127.0.0.1:5174".parse().unwrap(),
            ];
        }
        list
    };

    // Build the application with session middleware
    let app = Router::new()
        .merge(create_routes())
        .layer(axum::middleware::from_fn(normalize_uri_middleware))
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
                        .allow_origin(allowed_origins)
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
                            HeaderName::from_static("accept-language"),
                            HeaderName::from_static("x-device-type"),
                            HeaderName::from_static("x-device-info")
                        ])
                        .allow_credentials(true),
                ),
        )
        .with_state(session_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting Trackivity server on {}", addr);
    tracing::info!("Session-based authentication enabled");
    tracing::info!("Redis session store configured");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
