use std::net::SocketAddr;

use axum::{http::Method, Router};
use configuration::Configuration;
use mailconfig::{apply_migrations, create_pool};
use state::AppState;
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

mod api;
mod configuration;
pub mod state;
pub mod tokens;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("MAILCONFIG_LOG")
                .from_env_lossy(),
        )
        .init();

    if dotenv::dotenv().is_ok() {
        info!("Loaded configuration from .env file");
    } else {
        warn!("No .env file detected, configuration only from process environment");
    }

    let config = Configuration::load().expect("Unable to load config from environment:");

    info!("Applying any pending migrations...");

    apply_migrations(config.database_url().as_str()).expect("Unable to apply migrations:");

    info!("Preparing database connection pool...");

    let pool = create_pool(config.database_url().as_str())
        .await
        .expect("Unable to estable database pool");

    let port = config.port();
    let state = AppState::new(config, pool);
    let app = Router::new()
        .nest("/api", api::router(&state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(AllowOrigin::mirror_request())
                .allow_headers(AllowHeaders::mirror_request()),
        );
    let app = app.with_state(state);

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    info!("Starting server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failure when running axum");
}
