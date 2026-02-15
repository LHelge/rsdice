use std::net::Ipv4Addr;

use axum::Router;
use backend::prelude::*;
use backend::routes;
use sqlx::PgPool;
use thiserror::Error;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Error)]
enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    info!("Starting up...");

    if let Err(e) = app().await {
        error!("Application error: {e}");
    }
}

async fn app() -> std::result::Result<(), AppError> {
    let config = Config::from_env()?;
    debug!("Configuration loaded: {:?}", config);

    let db = PgPool::connect(&config.database_url).await?;
    info!("Connected to database");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&db).await?;
    info!("Migrations complete");

    let state = AppState::new(config.clone(), db);

    let app = Router::new()
        .nest("/api", routes::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)).await?;

    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server error");

    Ok(())
}
