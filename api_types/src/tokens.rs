use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenListResponseEntry {
    pub token: String,
    pub label: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenListResponse {
    pub username: String,
    pub used_token: String,
    pub tokens: Vec<TokenListResponseEntry>,
}
