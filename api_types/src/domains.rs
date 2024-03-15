//! Domains
//!

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListDomainResponse {
    pub domains: BTreeMap<String, ListDomainResponseEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ListDomainResponseEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_mx: Option<String>,
    pub sender_verify: bool,
    pub grey_listing: bool,
    pub virus_check: bool,
    pub spamcheck_threshold: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct SetDomainFlagsRequest {
    pub domain_name: String,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub remote_mx: Option<String>,
    #[serde(default)]
    pub sender_verify: Option<bool>,
    #[serde(default)]
    pub grey_listing: Option<bool>,
    #[serde(default)]
    pub virus_check: Option<bool>,
    #[serde(default)]
    pub spamcheck_threshold: Option<i32>,
}
