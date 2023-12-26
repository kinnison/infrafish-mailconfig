use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use thiserror::Error;

use crate::{configuration::Configuration, state::AppState};

mod domain;
mod frontend;
mod tokens;
mod users;

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
    #[error("Not a login or account: {0}")]
    NotLoginOrAccount(String),
    #[error("Not an alias: {0}")]
    NotAlias(String),
    #[error("Alias component {0} was not found")]
    AliasComponentNotFound(String),
    #[error("Cannot remove last component, alias {0} would become empty")]
    AliasWouldBecomeEmpty(String),
    #[error("User already exists")]
    UserAlreadyExists(String),
}

pub type APIResult<T> = std::result::Result<T, APIError>;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum APIResponseError {
    NotFound { item: String },
    PermissionDenied { why: String },
    DatabaseError { msg: String },
    AuthenticationFailure { reason: String },
    TokenInUse { token: String },
    BadToken { token: String },
    NotLoginOrAccount { item: String },
    NotAlias { item: String },
    AliasComponentNotFound { component: String },
    AliasWouldBecomeEmpty { item: String },
    UserAlreadyExists { item: String },
}

impl From<APIError> for APIResponseError {
    fn from(value: APIError) -> Self {
        match value {
            APIError::DatabaseError(e) => Self::DatabaseError { msg: e.to_string() },
            e @ APIError::AuthErrorNoToken => Self::AuthenticationFailure {
                reason: e.to_string(),
            },
            e @ APIError::AuthErrorBadToken(_) => Self::AuthenticationFailure {
                reason: e.to_string(),
            },
            e @ APIError::AuthErrorTokenInUse(_) => Self::TokenInUse {
                token: e.to_string(),
            },
            e @ APIError::BadToken(_) => Self::BadToken {
                token: e.to_string(),
            },
            APIError::NotFound(e) => Self::NotFound { item: e },
            APIError::PermissionDenied(e) => Self::PermissionDenied { why: e },
            APIError::NotLoginOrAccount(e) => Self::NotLoginOrAccount { item: e },
            APIError::NotAlias(e) => Self::NotAlias { item: e },
            APIError::AliasComponentNotFound(s) => Self::AliasComponentNotFound { component: s },
            APIError::AliasWouldBecomeEmpty(s) => Self::AliasWouldBecomeEmpty { item: s },
            APIError::UserAlreadyExists(s) => Self::UserAlreadyExists { item: s },
        }
    }
}

impl APIResponseError {
    fn code(&self) -> StatusCode {
        match self {
            APIResponseError::NotFound { .. } => StatusCode::NOT_FOUND,
            APIResponseError::BadToken { .. }
            | APIResponseError::AuthenticationFailure { .. }
            | APIResponseError::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            APIResponseError::TokenInUse { .. } => StatusCode::BAD_REQUEST,
            APIResponseError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            APIResponseError::AliasComponentNotFound { .. }
            | APIResponseError::AliasWouldBecomeEmpty { .. }
            | APIResponseError::NotAlias { .. }
            | APIResponseError::UserAlreadyExists { .. }
            | APIResponseError::NotLoginOrAccount { .. } => StatusCode::BAD_REQUEST,
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

async fn autoconfig(Path(domain): Path<String>) -> APIResult<Response> {
    const TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>

<clientConfig version="1.1">
  <emailProvider id="infrafish.uk">
    <domain>$DOMAIN</domain>
    <displayName>Infrafish Email</displayName>
    <displayShortName>Infrafish</displayShortName>
    <incomingServer type="imap">
      <hostname>mail.infrafish.uk</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <authentication>password-cleartext</authentication>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>mail.infrafish.uk</hostname>
      <port>587</port>
      <socketType>STARTTLS</socketType>
      <authentication>password-cleartext</authentication>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>
"#;

    let domain = domain.strip_prefix("autoconfig.").unwrap_or(&domain);

    let body = TEMPLATE.replace("$DOMAIN", domain);

    println!("Created: {body}");

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/xml")], body).into_response())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/autoconfig/:domain", get(autoconfig))
        .route("/ping", get(get_ping))
        .nest("/frontend", frontend::router())
        .nest("/token", tokens::router(state))
        .nest("/domain", domain::router(state))
        .nest("/user", users::router(state))
}
