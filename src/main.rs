use std::net::SocketAddr;

use axum::Router;
use configuration::Configuration;
use mailconfig::{apply_migrations, create_pool};
use state::AppState;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

mod api;
mod configuration;
mod state;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("MAILCONFIG_LOG")
                .from_env_lossy(),
        )
        .init();

    let config = Configuration::load().expect("Unable to load config from environment:");

    info!("Applying any pending migrations...");

    apply_migrations(config.database_url().as_str()).expect("Unable to apply migrations:");

    info!("Preparing database connection pool...");

    let pool = create_pool(config.database_url().as_str())
        .await
        .expect("Unable to estable database pool");

    let app = Router::new().nest("/api", api::router());
    let port = config.port();
    let app = app.with_state(AppState::new(config, pool));

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    info!("Starting server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failure when running axum");
}
