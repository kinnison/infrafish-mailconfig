use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use thiserror::Error;

use crate::{configuration::Configuration, state::AppState};

mod frontend;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Database failure: {0}")]
    DatabaseError(#[from] diesel::result::Error),
}

pub type APIResult<T> = std::result::Result<T, APIError>;

#[derive(Serialize)]
enum APIResponseError {
    DatabaseError(String),
}

impl From<APIError> for APIResponseError {
    fn from(value: APIError) -> Self {
        match value {
            APIError::DatabaseError(e) => Self::DatabaseError(e.to_string()),
        }
    }
}

#[derive(Serialize)]
struct ErrorOutcome {
    error: APIResponseError,
}

impl IntoResponse for APIError {
    fn into_response(self) -> axum::response::Response {
        Json::from(ErrorOutcome { error: self.into() }).into_response()
    }
}

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
    Router::new()
        .route("/ping", get(get_ping))
        .nest("/frontend", frontend::router())
}
