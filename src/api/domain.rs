use std::collections::BTreeMap;

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use mailconfig::{
    models::{Authorisation, MailDomain, MailUser},
    Connection,
};
use serde::{Deserialize, Serialize};

use crate::{api::APIError, state::AppState, tokens::Authorised};

use super::APIResult;

mod entries;
mod keys;

#[derive(Serialize)]
struct ListDomainResponse {
    domains: BTreeMap<String, ListDomainResponseEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ListDomainResponseEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_mx: Option<String>,
    sender_verify: bool,
    grey_listing: bool,
    virus_check: bool,
    spamcheck_threshold: i32,
}

async fn list_domains(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<ListDomainResponse>> {
    let all_domains = MailDomain::by_owner(&mut db, auth.user()).await?;
    let ret = ListDomainResponse {
        domains: all_domains
            .into_iter()
            .map(|dom| {
                (
                    dom.domainname,
                    ListDomainResponseEntry {
                        remote_mx: dom.remotemx,
                        sender_verify: dom.sender_verify,
                        grey_listing: dom.grey_listing,
                        virus_check: dom.virus_check,
                        spamcheck_threshold: dom.spamcheck_threshold,
                    },
                )
            })
            .collect(),
    };
    Ok(ret.into())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SetDomainFlagsRequest {
    domain_name: String,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    remote_mx: Option<String>,
    #[serde(default)]
    sender_verify: Option<bool>,
    #[serde(default)]
    grey_listing: Option<bool>,
    #[serde(default)]
    virus_check: Option<bool>,
    #[serde(default)]
    spamcheck_threshold: Option<i32>,
}

async fn set_domain_flags(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<SetDomainFlagsRequest>,
) -> APIResult<Json<ListDomainResponseEntry>> {
    let mut domain = MailDomain::by_name(&mut db, &body.domain_name)
        .await?
        .ok_or_else(|| APIError::NotFound(body.domain_name.clone()))?;

    if !domain.may_access(&mut db, &auth).await? {
        return Err(APIError::PermissionDenied(body.domain_name.clone()));
    }

    // Permission granted, so let's see what we can do...
    domain.owner = if let Some(owner) = body.owner.as_deref() {
        if !auth.superuser() {
            return Err(APIError::PermissionDenied(body.domain_name.clone()));
        }
        if let Some(user) = MailUser::by_name(&mut db, owner).await? {
            user.id
        } else {
            return Err(APIError::NotFound(owner.to_string()));
        }
    } else {
        domain.owner
    };

    // The rest does not need superuser
    if let Some(remote_mx) = body.remote_mx.as_deref() {
        if remote_mx.is_empty() {
            domain.remotemx = None;
        } else {
            domain.remotemx = Some(remote_mx.to_string());
        }
    }
    if let Some(sender_verify) = body.sender_verify {
        domain.sender_verify = sender_verify;
    }
    if let Some(grey_listing) = body.grey_listing {
        domain.grey_listing = grey_listing;
    }
    if let Some(virus_check) = body.virus_check {
        domain.virus_check = virus_check;
    }
    if let Some(spamcheck_threshold) = body.spamcheck_threshold {
        domain.spamcheck_threshold = spamcheck_threshold;
    }

    domain.save(&mut db).await?;

    Ok(ListDomainResponseEntry {
        remote_mx: domain.remotemx,
        sender_verify: domain.sender_verify,
        grey_listing: domain.grey_listing,
        virus_check: domain.virus_check,
        spamcheck_threshold: domain.spamcheck_threshold,
    }
    .into())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct CreateDomainRequest {
    domain_name: String,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    remote_mx: Option<String>,
    #[serde(default)]
    sender_verify: Option<bool>,
    #[serde(default)]
    grey_listing: Option<bool>,
    #[serde(default)]
    virus_check: Option<bool>,
    #[serde(default)]
    spamcheck_threshold: Option<i32>,
}

async fn create_domain(
    mut db: Connection,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<CreateDomainRequest>,
) -> APIResult<Json<ListDomainResponseEntry>> {
    if !auth.superuser() {
        return Err(APIError::PermissionDenied(
            "You are not permitted to create domains".into(),
        ));
    }

    let owner = if let Some(owner) = &body.owner {
        MailUser::by_name(&mut db, owner)
            .await?
            .ok_or_else(|| APIError::NotFound(format!("Unknown user {owner}")))?
            .id
    } else {
        auth.user()
    };

    let domain = MailDomain::create(
        &mut db,
        &body.domain_name,
        owner,
        body.remote_mx.as_deref(),
        body.sender_verify.unwrap_or(true),
        body.grey_listing.unwrap_or(false),
        body.virus_check.unwrap_or(true),
        body.spamcheck_threshold.unwrap_or(100),
    )
    .await?;

    Ok(Json::from(ListDomainResponseEntry {
        remote_mx: domain.remotemx,
        sender_verify: domain.sender_verify,
        grey_listing: domain.grey_listing,
        virus_check: domain.virus_check,
        spamcheck_threshold: domain.spamcheck_threshold,
    }))
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/new", post(create_domain))
        .route("/list", get(list_domains))
        .route("/set-flags", post(set_domain_flags))
        .nest("/key", keys::router())
        .nest("/entry", entries::router())
        .authorise(state.clone())
}
