use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use thiserror::Error;

use crate::{configuration::Configuration, state::AppState};

mod domain;
mod frontend;
mod tokens;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Entry not found: {0}")]
    NotFound(String),
    #[error("Permission denied accessing: {0}")]
    PermissionDenied(String),
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
    NotFound(String),
    PermissionDenied(String),
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
            APIError::NotFound(e) => Self::NotFound(e),
            APIError::PermissionDenied(e) => Self::PermissionDenied(e),
        }
    }
}

impl APIResponseError {
    fn code(&self) -> StatusCode {
        match self {
            APIResponseError::NotFound(_) => StatusCode::NOT_FOUND,
            APIResponseError::BadToken(_)
            | APIResponseError::AuthenticationFailure(_)
            | APIResponseError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            APIResponseError::TokenInUse(_) => StatusCode::BAD_REQUEST,
            APIResponseError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct ErrorOutcome {
    error: APIResponseError,
}

impl IntoResponse for APIError {
    fn into_response(self) -> axum::response::Response {
        let error = APIResponseError::from(self);
        let code = error.code();
        (code, Json::from(ErrorOutcome { error })).into_response()
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
        .nest("/domain", domain::router(state))
}
