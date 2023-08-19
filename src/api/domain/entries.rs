//! Domain entries (logins, accounts, aliases, etc)
//!

use std::collections::HashMap;

use axum::{
    extract::Path,
    routing::{delete, get},
    Extension, Json, Router,
};
use mailconfig::{models::*, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    api::{APIError, APIResult},
    state::AppState,
};

#[derive(Serialize, Debug, Default)]
struct EntryListResponse {
    entries: HashMap<String, EntryListResponseItem>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "kind")]
enum EntryListResponseItem {
    Login,
    Account,
    Alias { expansion: String },
    Blackhole,
    Bouncer,
}
async fn list_entries(
    mut db: Connection,
    Path(domain_name): Path<String>,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<EntryListResponse>> {
    let domain = MailDomain::by_name(&mut db, &domain_name)
        .await?
        .ok_or_else(|| APIError::NotFound(domain_name.clone()))?;

    if !domain.may_access(&mut db, &auth).await? {
        return Err(APIError::PermissionDenied(domain_name.clone()));
    }

    let mut res = EntryListResponse::default();

    for MailEntry {
        name,
        kind,
        expansion,
        ..
    } in domain.entries(&mut db).await?
    {
        res.entries.insert(
            name,
            match kind {
                MailEntryKind::Login => EntryListResponseItem::Login,
                MailEntryKind::Account => EntryListResponseItem::Account,
                MailEntryKind::Alias => EntryListResponseItem::Alias {
                    expansion: expansion.unwrap_or_default(),
                },
                MailEntryKind::Bouncer => EntryListResponseItem::Bouncer,
                MailEntryKind::Blackhole => EntryListResponseItem::Blackhole,
            },
        );
    }

    Ok(Json::from(res))
}

#[derive(Serialize, Debug)]
struct DeletionResponse {
    deleted: String,
}

async fn delete_entry(
    mut db: Connection,
    Path((domain_name, entry)): Path<(String, String)>,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<DeletionResponse>> {
    let domain = MailDomain::by_name(&mut db, &domain_name)
        .await?
        .ok_or_else(|| APIError::NotFound(domain_name.clone()))?;

    if !domain.may_access(&mut db, &auth).await? {
        return Err(APIError::PermissionDenied(domain_name.clone()));
    }

    let full_name = format!("{entry}@{domain_name}");

    let db_entry = domain
        .entry_by_name(&mut db, &entry)
        .await?
        .ok_or_else(|| APIError::NotFound(full_name.clone()))?;

    db_entry.delete(&mut db).await?;

    Ok(Json::from(DeletionResponse { deleted: full_name }))
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum CreateEntryRequest {
    Login { name: String, password: String },
    Account { name: String, password: String },
    Alias { name: String, expansion: String },
}

impl CreateEntryRequest {
    fn name(&self) -> &str {
        match self {
            Self::Login { name, .. } => name,
            Self::Account { name, .. } => name,
            Self::Alias { name, .. } => name,
        }
    }
}

#[derive(Serialize, Debug)]
struct CreationResponse {
    created: String,
}

async fn create_entry(
    mut db: Connection,
    Path(domain_name): Path<String>,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<CreateEntryRequest>,
) -> APIResult<Json<CreationResponse>> {
    let domain = MailDomain::by_name(&mut db, &domain_name)
        .await?
        .ok_or_else(|| APIError::NotFound(domain_name.clone()))?;

    if !domain.may_access(&mut db, &auth).await? {
        return Err(APIError::PermissionDenied(domain_name.clone()));
    }

    let full_name = format!("{}@{domain_name}", body.name());

    match body {
        CreateEntryRequest::Login { name, password } => {
            domain.new_login(&mut db, &name, &password, false).await?
        }
        CreateEntryRequest::Account { name, password } => {
            domain.new_login(&mut db, &name, &password, true).await?
        }
        CreateEntryRequest::Alias { name, expansion } => {
            domain.new_alias(&mut db, &name, &expansion).await?
        }
    }

    Ok(Json::from(CreationResponse { created: full_name }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:domain_name", get(list_entries).put(create_entry))
        .route("/:domain_name/:entry", delete(delete_entry))
}
