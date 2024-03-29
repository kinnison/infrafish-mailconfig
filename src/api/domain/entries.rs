//! Domain entries (logins, accounts, aliases, etc)
//!

use std::collections::HashMap;

use axum::{extract::Path, routing::get, Extension, Json, Router};
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
#[serde(tag = "kind", rename_all = "kebab-case")]
enum EntryListResponseItem {
    Login,
    Account,
    Alias { expansion: String },
    Blackhole { reason: String },
    Bouncer { reason: String },
    List { members: String },
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
                MailEntryKind::Bouncer => EntryListResponseItem::Bouncer {
                    reason: expansion.unwrap_or_default(),
                },
                MailEntryKind::Blackhole => EntryListResponseItem::Blackhole {
                    reason: expansion.unwrap_or_default(),
                },
                MailEntryKind::List => EntryListResponseItem::List {
                    members: expansion.unwrap_or_default(),
                },
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
    Bouncer { name: String, reason: String },
    Blackhole { name: String, reason: String },
    List { name: String, members: String },
}

impl CreateEntryRequest {
    fn name(&self) -> &str {
        match self {
            Self::Login { name, .. } => name,
            Self::Account { name, .. } => name,
            Self::Alias { name, .. } => name,
            Self::Bouncer { name, .. } => name,
            Self::Blackhole { name, .. } => name,
            Self::List { name, .. } => name,
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
        CreateEntryRequest::Bouncer { name, reason } => {
            domain.new_bouncer(&mut db, &name, &reason).await?
        }
        CreateEntryRequest::Blackhole { name, reason } => {
            domain.new_blackhole(&mut db, &name, &reason).await?
        }
        CreateEntryRequest::List { name, members } => {
            domain.new_list(&mut db, &name, &members).await?
        }
    }

    Ok(Json::from(CreationResponse { created: full_name }))
}

async fn get_entry(
    mut db: Connection,
    Path((domain_name, entry)): Path<(String, String)>,
    Extension(auth): Extension<Authorisation>,
) -> APIResult<Json<EntryListResponseItem>> {
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

    Ok(Json::from(match db_entry.kind {
        MailEntryKind::Login => EntryListResponseItem::Login,
        MailEntryKind::Account => EntryListResponseItem::Account,
        MailEntryKind::Alias => EntryListResponseItem::Alias {
            expansion: db_entry.expansion.unwrap_or_default(),
        },
        MailEntryKind::Bouncer => EntryListResponseItem::Bouncer {
            reason: db_entry.expansion.unwrap_or_default(),
        },
        MailEntryKind::Blackhole => EntryListResponseItem::Blackhole {
            reason: db_entry.expansion.unwrap_or_default(),
        },
        MailEntryKind::List => EntryListResponseItem::List {
            members: db_entry.expansion.unwrap_or_default(),
        },
    }))
}

#[derive(Deserialize, Debug)]
#[serde(untagged, rename_all = "kebab-case")]
enum EditEntryRequest {
    SetPassword { password: String },
    Expansion { expansion: String },
    AddExpansion { add: String },
    RemoveExpansion { remove: String },
    ChangeReason { reason: String },
}

#[derive(Serialize, Debug)]
struct EditEntryResponse {
    updated: String,
}

async fn update_entry(
    mut db: Connection,
    Path((domain_name, entry)): Path<(String, String)>,
    Extension(auth): Extension<Authorisation>,
    Json(body): Json<EditEntryRequest>,
) -> APIResult<Json<EditEntryResponse>> {
    let domain = MailDomain::by_name(&mut db, &domain_name)
        .await?
        .ok_or_else(|| APIError::NotFound(domain_name.clone()))?;

    if !domain.may_access(&mut db, &auth).await? {
        return Err(APIError::PermissionDenied(domain_name.clone()));
    }

    let full_name = format!("{entry}@{domain_name}");

    let mut db_entry = domain
        .entry_by_name(&mut db, &entry)
        .await?
        .ok_or_else(|| APIError::NotFound(full_name.clone()))?;

    match body {
        EditEntryRequest::SetPassword { password } => {
            if !matches!(db_entry.kind, MailEntryKind::Login | MailEntryKind::Account) {
                return Err(APIError::NotLoginOrAccount(full_name));
            }
            db_entry.set_password(&password);
        }
        EditEntryRequest::Expansion { expansion } => {
            if !matches!(db_entry.kind, MailEntryKind::Alias) {
                return Err(APIError::NotAlias(full_name));
            }
            db_entry.expansion = Some(expansion);
        }
        EditEntryRequest::AddExpansion { add } => {
            if !matches!(db_entry.kind, MailEntryKind::Alias | MailEntryKind::List) {
                return Err(APIError::NotAlias(full_name));
            }
            let mut bits: Vec<&str> = db_entry
                .expansion
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(str::trim)
                .collect();
            if !bits.contains(&add.as_str()) {
                bits.push(&add);
            }
            db_entry.expansion = Some(bits.join(", "));
        }
        EditEntryRequest::RemoveExpansion { remove } => {
            if !matches!(db_entry.kind, MailEntryKind::Alias | MailEntryKind::List) {
                return Err(APIError::NotAlias(full_name));
            }
            let bits: Vec<&str> = db_entry
                .expansion
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(str::trim)
                .filter(|e| e != &remove)
                .collect();
            if bits.is_empty() {
                return Err(APIError::AliasWouldBecomeEmpty(full_name));
            }
            let new_expn = Some(bits.join(", "));
            if new_expn == db_entry.expansion {
                return Err(APIError::AliasComponentNotFound(remove));
            }
            db_entry.expansion = new_expn;
        }
        EditEntryRequest::ChangeReason { reason } => {
            if !matches!(
                db_entry.kind,
                MailEntryKind::Bouncer | MailEntryKind::Blackhole,
            ) {
                return Err(APIError::NotBouncerOrBlackhole(full_name));
            }
            db_entry.expansion = Some(reason);
        }
    }

    db_entry.save(&mut db).await?;

    Ok(Json::from(EditEntryResponse { updated: full_name }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:domain_name", get(list_entries).put(create_entry))
        .route(
            "/:domain_name/:entry",
            get(get_entry).delete(delete_entry).post(update_entry),
        )
}
