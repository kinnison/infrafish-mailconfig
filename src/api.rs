use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use thiserror::Error;

use crate::{configuration::Configuration, state::AppState};

mod frontend;
mod tokens;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Database failure: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Authentication failed, no token provided")]
    AuthErrorNoToken,
    #[error("Authentication failed, bad token provided: {0}")]
    AuthErrorBadToken(String),
    #[error("Authentication token is in use: {0}")]
    AuthErrorTokenInUse(String),
    #[error("Bad token: {0}")]
    BadToken(String),
}

pub type APIResult<T> = std::result::Result<T, APIError>;

#[derive(Serialize)]
enum APIResponseError {
    DatabaseError(String),
    AuthenticationFailure(String),
    TokenInUse(String),
    BadToken(String),
}

impl From<APIError> for APIResponseError {
    fn from(value: APIError) -> Self {
        match value {
            APIError::DatabaseError(e) => Self::DatabaseError(e.to_string()),
            e @ APIError::AuthErrorNoToken => Self::AuthenticationFailure(e.to_string()),
            e @ APIError::AuthErrorBadToken(_) => Self::AuthenticationFailure(e.to_string()),
            e @ APIError::AuthErrorTokenInUse(_) => Self::TokenInUse(e.to_string()),
            e @ APIError::BadToken(_) => Self::BadToken(e.to_string()),
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

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_ping))
        .nest("/frontend", frontend::router())
        .nest("/token", tokens::router(state))
}
