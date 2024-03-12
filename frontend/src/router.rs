//! Page router
//!
//!

use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;
use time::Duration;
use uuid::Uuid;

use crate::state::{ApplicationState, LoginState};

#[component]
pub fn PageRouter() -> impl IntoView {
    let state = ApplicationState::acquire();
    let ui_ready = state.is_ready();
    let login_state = state.login_state();
    let logged_in =
        move || ui_ready.get() && matches!(login_state.get(), LoginState::LoggedIn(_, _));
    view! {
        <LoginDialog/>
        <LoggingInDialog/>
        <Show when=logged_in fallback=|| view! {}>
            <TopLevelRouter/>
        </Show>
    }
}

#[component]
fn TopLevelRouter() -> impl IntoView {
    view! {}
}

#[component]
fn LoggingInDialog() -> impl IntoView {
    let state = ApplicationState::acquire();
    let ui_ready = state.is_ready();
    let login_state = state.login_state();
    let logging_in = Signal::derive(move || {
        ui_ready.get() && matches!(login_state.get(), LoginState::TryingLogin(_))
    });

    view! {
        <Modal show_when=logging_in>
            <ModalHeader>
                <ModalTitle>"Please wait…"</ModalTitle>
            </ModalHeader>
            <ModalBody>
                <Skeleton>"Trying to log in"</Skeleton>
            </ModalBody>
        </Modal>
    }
}

#[component]
fn LoginDialog() -> impl IntoView {
    let state = ApplicationState::acquire();
    let login_state = state.login_state();
    let ui_ready = state.is_ready();
    let logged_out = Signal::derive(move || {
        ui_ready.get() && matches!(login_state.get(), LoginState::LoggedOut(_))
    });

    let token = Signal::derive(move || login_state.get().token().unwrap_or_default());

    let edit_token = store_value(String::new());
    let set_edit_token = Callback::from(move |value| edit_token.set_value(value));

    let try_login = move |_| {
        state.run_login(edit_token.get_value());
    };

    let forget = move |_| {
        state.forget_login();
    };

    create_effect(move |_| {
        edit_token.set_value(token.get());
    });

    // Finally, toast if we can.
    let toaster: Toasts = expect_context();
    create_effect(move |prev| {
        let state = login_state.get();
        match (prev, &state) {
            (Some(LoginState::TryingLogin(_)), LoginState::LoggedOut(_)) => {
                // We were trying to log in, but we stopped trying, that means the login failed
                toaster.push(Toast {
                    id: Uuid::new_v4(),
                    created_at: time::OffsetDateTime::now_utc(),
                    variant: ToastVariant::Error,
                    header: "Login failed".into_view(),
                    body: "Bad, or unknown token".into_view(),
                    timeout: ToastTimeout::CustomDelay(Duration::SECOND * 2),
                });
            }
            (Some(LoginState::TryingLogin(_)), LoginState::LoggedIn(_, username)) => {
                toaster.push(Toast {
                    id: Uuid::new_v4(),
                    created_at: time::OffsetDateTime::now_utc(),
                    variant: ToastVariant::Success,
                    header: "Login successful".into_view(),
                    body: format!("Logged in as {username}").into_view(),
                    timeout: ToastTimeout::CustomDelay(Duration::SECOND * 2),
                });
            }

            // Every other state change we don't care about
            _ => (),
        }

        state
    });

    view! {
        <Modal show_when=logged_out>
            <ModalHeader>
                <ModalTitle>"Provide Mail Admin Token…"</ModalTitle>
            </ModalHeader>
            <ModalBody>
                <PasswordInput get=token set=set_edit_token/>
            </ModalBody>
            <ModalFooter>
                <ButtonWrapper>
                    <Button on_click=try_login color=ButtonColor::Primary>
                        "Log in"
                    </Button>
                    <Button on_click=forget color=ButtonColor::Secondary>
                        "Forget token"
                    </Button>
                </ButtonWrapper>
            </ModalFooter>
        </Modal>
    }
}
