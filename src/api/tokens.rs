use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use mailconfig::{models::MailAuthToken, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    state::AppState,
    tokens::{Authorisation, Authorised},
};

use super::{APIError, APIResult};

#[derive(Serialize)]
struct TokenListResponseEntry {
    token: String,
    label: String,
}

#[derive(Serialize)]
struct TokenListResponse {
    username: String,
    used_token: String,
    tokens: Vec<TokenListResponseEntry>,
}

async fn list_tokens(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<TokenListResponse>> {
    let all_tokens = MailAuthToken::by_owner(&mut db, auth.user()).await?;
    Ok(TokenListResponse {
        username: auth.username().to_string(),
        used_token: auth.token().to_string(),
        tokens: all_tokens
            .into_iter()
            .map(|v| TokenListResponseEntry {
                token: v.token,
                label: v.label,
            })
            .collect(),
    }
    .into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTokenRequest {
    label: String,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    token: String,
}

async fn create_token(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<CreateTokenRequest>,
) -> APIResult<Json<CreateTokenResponse>> {
    let newtok = MailAuthToken::create(&mut db, auth.user(), &body.label).await?;
    Ok(CreateTokenResponse {
        token: newtok.token,
    }
    .into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevokeTokenRequest {
    token: String,
}

#[derive(Serialize)]
struct RevokeTokenResponse {
    label: String,
}

async fn revoke_token(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<RevokeTokenRequest>,
) -> APIResult<Json<RevokeTokenResponse>> {
    if auth.token() == body.token {
        return Err(APIError::AuthErrorTokenInUse(body.token));
    }

    let db_token = MailAuthToken::by_token(&mut db, &body.token)
        .await?
        .ok_or_else(|| APIError::BadToken(body.token.clone()))?;

    if db_token.mailuser != auth.user() {
        return Err(APIError::BadToken(body.token));
    }

    // The token exists and it's ours, remove it

    let ret = RevokeTokenResponse {
        label: db_token.label.clone(),
    };

    db_token.delete_self(&mut db).await?;

    Ok(ret.into())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/list", get(list_tokens))
        .route("/create", post(create_token))
        .route("/revoke", post(revoke_token))
        .authorise(state.clone())
}
