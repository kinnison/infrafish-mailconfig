//! Token management stuff

use axum::{
    http::{header, Request},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use mailconfig::{
    models::{MailAuthToken, MailUser},
    Connection,
};

use crate::{
    api::{APIError, APIResult},
    state::AppState,
};

/// An extension to be used by routes to determine access control
/// If this extension isn't present that means that the user didn't
/// supply a token.  if they supplied a token and it was bad then
/// we return an error instead.
#[derive(Debug, Clone)]
pub struct Authorisation {
    token: String,
    user: i32,
    username: String,
}

impl Authorisation {
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn user(&self) -> i32 {
        self.user
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

async fn auth<B>(
    mut db: Connection,
    mut req: Request<B>,
    next: Next<B>,
) -> APIResult<impl IntoResponse> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer ").map(String::from));

    let token = auth_header.ok_or(APIError::AuthErrorNoToken)?;

    let db_token = MailAuthToken::by_token(&mut db, &token)
        .await?
        .ok_or_else(|| APIError::AuthErrorBadToken(token.clone()))?;

    let user = db_token.mailuser;

    let username = MailUser::by_id(&mut db, user).await?.username;

    let auth = Authorisation {
        token,
        user,
        username,
    };

    req.extensions_mut().insert(auth);

    Ok(next.run(req).await)
}

pub trait Authorised {
    fn authorise(self, state: AppState) -> Self;
}

impl Authorised for Router<AppState> {
    fn authorise(self, state: AppState) -> Self {
        self.route_layer(middleware::from_fn_with_state(state, auth))
    }
}
