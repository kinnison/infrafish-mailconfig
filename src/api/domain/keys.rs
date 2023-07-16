use std::collections::BTreeMap;

use axum::{routing::post, Extension, Json, Router};
use mailconfig::{
    models::{MailDomain, MailDomainKey},
    Connection,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{APIError, APIResult},
    state::AppState,
    tokens::Authorisation,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ListDomainKeyRequest {
    mail_domain: String,
}

#[derive(Serialize)]
struct ListDomainKeyResponse {
    active: BTreeMap<String, String>,
    passive: BTreeMap<String, String>,
}

async fn list_domain_keys(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<ListDomainKeyRequest>,
) -> APIResult<Json<ListDomainKeyResponse>> {
    let domain = MailDomain::by_name(&mut db, &body.mail_domain)
        .await?
        .ok_or_else(|| APIError::NotFound(body.mail_domain.clone()))?;

    if !domain.may_access(&mut db, auth.user()).await? {
        return Err(APIError::PermissionDenied(body.mail_domain));
    }

    let keys = MailDomainKey::by_domain(&mut db, domain.id).await?;

    Ok(ListDomainKeyResponse {
        active: keys
            .iter()
            .filter(|k| k.signing)
            .map(|k| (k.selector.clone(), k.render_pubkey()))
            .collect(),
        passive: keys
            .iter()
            .filter(|k| !k.signing)
            .map(|k| (k.selector.clone(), k.render_pubkey()))
            .collect(),
    }
    .into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SetDomainKeySigningRequest {
    mail_domain: String,
    selector: String,
    signing: bool,
}

#[derive(Serialize)]
struct SetDomainKeySigningResponse {
    signing: bool,
}

async fn set_domainkey_signing(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<SetDomainKeySigningRequest>,
) -> APIResult<Json<SetDomainKeySigningResponse>> {
    let domain = MailDomain::by_name(&mut db, &body.mail_domain)
        .await?
        .ok_or_else(|| APIError::NotFound(body.mail_domain.clone()))?;

    if !domain.may_access(&mut db, auth.user()).await? {
        return Err(APIError::PermissionDenied(body.mail_domain));
    }

    let mut key = MailDomainKey::by_domain_and_selector(&mut db, domain.id, &body.selector)
        .await?
        .ok_or_else(|| APIError::NotFound(body.selector.clone()))?;

    key.signing = body.signing;

    key.save(&mut db).await?;

    Ok(SetDomainKeySigningResponse {
        signing: key.signing,
    }
    .into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct CreateDomainKeyRequest {
    mail_domain: String,
    selector: String,
    #[serde(default)]
    signing: bool,
}

#[derive(Serialize)]
struct CreateDomainKeyResponse {
    signing: bool,
    key: String,
}

async fn create_domain_key(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<CreateDomainKeyRequest>,
) -> APIResult<Json<CreateDomainKeyResponse>> {
    let domain = MailDomain::by_name(&mut db, &body.mail_domain)
        .await?
        .ok_or_else(|| APIError::NotFound(body.mail_domain.clone()))?;

    if !domain.may_access(&mut db, auth.user()).await? {
        return Err(APIError::PermissionDenied(body.mail_domain));
    }

    let key = MailDomainKey::create(&mut db, domain.id, &body.selector, body.signing).await?;

    Ok(CreateDomainKeyResponse {
        signing: key.signing,
        key: key.render_pubkey(),
    }
    .into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct DeleteDomainKeyRequest {
    mail_domain: String,
    selector: String,
}

#[derive(Serialize)]
struct DeleteDomainKeyResponse {
    selector: String,
    signing: bool,
}

async fn delete_domainkey(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<DeleteDomainKeyRequest>,
) -> APIResult<Json<DeleteDomainKeyResponse>> {
    let domain = MailDomain::by_name(&mut db, &body.mail_domain)
        .await?
        .ok_or_else(|| APIError::NotFound(body.mail_domain.clone()))?;

    if !domain.may_access(&mut db, auth.user()).await? {
        return Err(APIError::PermissionDenied(body.mail_domain));
    }

    let key = MailDomainKey::by_domain_and_selector(&mut db, domain.id, &body.selector)
        .await?
        .ok_or_else(|| APIError::NotFound(body.selector.clone()))?;

    let res = DeleteDomainKeyResponse {
        selector: body.selector,
        signing: key.signing,
    };

    key.delete_self(&mut db).await?;

    Ok(res.into())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", post(list_domain_keys))
        .route("/set-signing", post(set_domainkey_signing))
        .route("/create", post(create_domain_key))
        .route("/delete", post(delete_domainkey))
}
