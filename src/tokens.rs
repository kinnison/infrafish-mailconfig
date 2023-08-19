//! Token management stuff

use axum::{
    http::{header, Request},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use mailconfig::{
    models::{Authorisation, MailAuthToken, MailUser},
    Connection,
};

use crate::{
    api::{APIError, APIResult},
    state::AppState,
};

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

    let user = MailUser::by_id(&mut db, db_token.mailuser).await?;

    let auth = Authorisation::new(token, &user);

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
