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

pub fn router() -> Router<AppState> {
    Router::new().route("/:domain_name", get(list_entries))
}
