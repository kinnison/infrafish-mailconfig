//! Frontend configuration

use std::collections::HashMap;

use axum::{extract::State, routing::get, Json, Router};
use mailconfig::{models, Connection};
use serde::Serialize;

use crate::{configuration::Configuration, state::AppState};

use super::APIResult;

#[derive(Serialize)]
struct FrontendJson {
    version: String,
    all_domains: Vec<String>,
    per_domain: HashMap<String, FrontendJsonDomain>,
}

#[derive(Serialize)]
struct FrontendJsonDomain {
    sender_allow_list: Vec<String>,
    sender_deny_list: Vec<String>,
    sender_verify_enable: bool,
    greylisting_enable: bool,
    viruscheck_enable: bool,
    spamcheck_threshold: i32,
}

async fn get_json(
    State(config): State<Configuration>,
    mut db: Connection,
) -> APIResult<Json<FrontendJson>> {
    let all_mail_domains = models::MailDomain::get_all(&mut db).await?;
    let all_domains = all_mail_domains
        .iter()
        .map(|d| d.domainname.clone())
        .collect();
    let mut per_domain = HashMap::new();

    for domain in all_mail_domains {
        let fedom = FrontendJsonDomain {
            sender_allow_list: models::AllowDenyList::all_allows(&mut db, domain.id).await?,
            sender_deny_list: models::AllowDenyList::all_denys(&mut db, domain.id).await?,
            sender_verify_enable: domain.sender_verify,
            greylisting_enable: domain.grey_listing,
            viruscheck_enable: domain.virus_check,
            spamcheck_threshold: domain.spamcheck_threshold,
        };
        per_domain.insert(domain.domainname, fedom);
    }

    Ok((FrontendJson {
        version: config.version().to_string(),
        all_domains,
        per_domain,
    })
    .into())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/json", get(get_json))
}
