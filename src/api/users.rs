use std::collections::HashMap;

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use mailconfig::{
    models::{Authorisation, MailUser},
    Connection,
};
use serde::{Deserialize, Serialize};

use crate::{state::AppState, tokens::Authorised};

use super::{APIError, APIResult};

#[derive(Serialize, Default)]
struct ListUsersResponse {
    users: HashMap<String, ListUsersResponseEntry>,
}

#[derive(Serialize)]
struct ListUsersResponseEntry {
    superuser: bool,
    tokens: HashMap<String, String>,
}

async fn list_users(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<ListUsersResponse>> {
    if !auth.superuser() {
        return Err(APIError::PermissionDenied("You may not list users".into()));
    }
    let mut res = ListUsersResponse::default();

    for user in MailUser::all(&mut db).await? {
        let tokens = user.tokens(&mut db).await?;
        res.users.insert(
            user.username,
            ListUsersResponseEntry {
                superuser: user.superuser,
                tokens: tokens
                    .into_iter()
                    .map(|tok| (tok.label, tok.token))
                    .collect(),
            },
        );
    }

    Ok(Json::from(res))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    #[serde(default)]
    superuser: bool,
}

async fn create_user(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<CreateUserRequest>,
) -> APIResult<Json<ListUsersResponseEntry>> {
    if !auth.superuser() {
        return Err(APIError::PermissionDenied(
            "You may not create users".into(),
        ));
    }

    if MailUser::by_name(&mut db, &body.username).await?.is_some() {
        return Err(APIError::UserAlreadyExists(body.username));
    }

    let user = MailUser::create(&mut db, &body.username, body.superuser).await?;

    let tokens = user.tokens(&mut db).await?;

    Ok(Json::from(ListUsersResponseEntry {
        superuser: user.superuser,
        tokens: tokens
            .into_iter()
            .map(|tok| (tok.label, tok.token))
            .collect(),
    }))
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/list", get(list_users))
        .route("/new", post(create_user))
        .authorise(state.clone())
}
