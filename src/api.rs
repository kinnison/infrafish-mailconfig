use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::{configuration::Configuration, state::AppState};

#[derive(Serialize)]
struct PingOutput {
    version: String,
}
async fn get_ping(State(config): State<Configuration>) -> Json<PingOutput> {
    (PingOutput {
        version: config.version().to_string(),
    })
    .into()
}

pub fn router() -> Router<AppState> {
    Router::new().route("/ping", get(get_ping))
}
