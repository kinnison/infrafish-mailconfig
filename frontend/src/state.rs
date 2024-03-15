//! Application state
//!
//!

use api_types::tokens::TokenListResponse;
use leptos::{logging::log, *};
use leptos_use::{storage::*, utils::JsonCodec};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Copy)]
pub struct ApplicationState {
    mounted: RwSignal<bool>,
    login_state: Signal<LoginState>,
    write_login_state: WriteSignal<LoginState>,
    forget_login_state: Callback<()>,
    try_login_action: Action<String, Result<(String, String), String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoginState {
    LoggedOut(Option<String>),
    TryingLogin(String),
    LoggedIn(String, String),
}

impl Default for LoginState {
    fn default() -> Self {
        LoginState::LoggedOut(None)
    }
}

const API_BASE: &str = "https://mail.infrafish.uk/api";

impl LoginState {
    pub fn token(&self) -> Option<String> {
        match self {
            LoginState::LoggedOut(token) => token.clone(),
            LoginState::TryingLogin(token) => Some(token.clone()),
            LoginState::LoggedIn(token, _username) => Some(token.clone()),
        }
    }

    pub fn api_token(&self) -> Option<String> {
        match self {
            LoginState::LoggedIn(token, _) => Some(token.clone()),
            _ => None,
        }
    }

    pub fn username(&self) -> Option<String> {
        match self {
            LoginState::LoggedIn(_token, username) => Some(username.clone()),
            _ => None,
        }
    }
}

impl ApplicationState {
    pub fn provide() -> Self {
        let (login_state, write_login_state, forget_login_state) =
            use_local_storage::<LoginState, JsonCodec>("login-token");
        // Normalise login state
        if let LoginState::TryingLogin(token) = login_state.get_untracked() {
            write_login_state.set_untracked(LoginState::LoggedOut(Some(token)));
        }
        let try_login_action = create_action(|token: &String| Self::try_login(token.clone()));

        create_effect({
            let value_getter = try_login_action.value();
            move |_prev| {
                let value = value_getter.get();
                if let Some(outcome) = value.clone() {
                    match outcome {
                        Ok((token, username)) => {
                            write_login_state.set(LoginState::LoggedIn(token, username))
                        }
                        Err(token) => write_login_state.set(LoginState::LoggedOut(Some(token))),
                    }
                }

                value
            }
        });

        let state = ApplicationState {
            mounted: create_rw_signal(false),
            login_state,
            write_login_state,
            forget_login_state: Callback::from(move |_| forget_login_state()),
            try_login_action,
        };
        provide_context(state);

        queue_microtask(move || state.mark_ready());

        state
    }

    pub fn is_ready(&self) -> ReadSignal<bool> {
        self.mounted.read_only()
    }

    fn mark_ready(&self) {
        self.mounted.set(true);
    }

    pub fn acquire() -> ApplicationState {
        expect_context()
    }

    pub fn login_state(&self) -> Signal<LoginState> {
        self.login_state
    }

    async fn try_login(token: String) -> Result<(String, String), String> {
        // Try and log in using the token (Basically try and get the username back)
        log!("Trying to log in with {token}");

        let client = reqwest::Client::builder()
            .build()
            .map_err(|_e| token.clone())?;
        let mut request = client.get("https://mail.infrafish.uk/api/token/list");
        request = request.bearer_auth(token.clone());
        let res = request.send().await.map_err(|_e| token.clone())?;

        if res.status().is_success() {
            let body: TokenListResponse = res.json().await.map_err(|_e| token.clone())?;

            Ok((token, body.username))
        } else {
            Err(token)
        }
    }

    pub async fn api_get<T>(&self, token: &str, uri: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned + 'static,
    {
        let client = reqwest::Client::builder().build()?;
        let res = client
            .get(format!("{}{}", API_BASE, uri))
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;
        res.json().await
    }

    pub async fn api_post<T, B>(
        &self,
        token: &str,
        uri: &str,
        body: &B,
    ) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned + 'static,
        B: Serialize,
    {
        let client = reqwest::Client::builder().build()?;
        let res = client
            .post(format!("{}{}", API_BASE, uri))
            .bearer_auth(token)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        res.json().await
    }

    pub fn log_out(&self) {
        if let LoginState::LoggedIn(token, _) = self.login_state.get() {
            self.write_login_state
                .set(LoginState::LoggedOut(Some(token)))
        }
    }

    pub fn run_login(&self, token: String) {
        self.write_login_state
            .set(LoginState::TryingLogin(token.clone()));
        self.try_login_action.dispatch(token);
    }

    pub fn forget_login(&self) {
        self.forget_login_state.call(());
    }
}
