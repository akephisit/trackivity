mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::database::Database;
use crate::routes::create_routes;

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
    
    // Run migrations
    database.migrate().await?;
    
    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let redis_manager = redis::aio::ConnectionManager::new(redis_client).await?;

    // Build application state
    let app_state = crate::handlers::AppState {
        database,
        redis: redis_manager,
        config: config.clone(),
    };

    // Build the application
    let app = Router::new()
        .merge(create_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .into_inner(),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}