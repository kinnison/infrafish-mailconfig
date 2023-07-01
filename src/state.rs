use axum::extract::FromRef;

use crate::configuration::Configuration;

#[derive(Clone, FromRef)]
pub struct AppState {
    config: Configuration,
    pool: mailconfig::Pool,
}

impl AppState {
    pub fn new(config: Configuration, pool: mailconfig::Pool) -> Self {
        Self { config, pool }
    }
}
