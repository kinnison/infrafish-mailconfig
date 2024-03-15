use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;

use crate::{router::PageRouter, state::ApplicationState};

mod router;
mod state;

#[component]
pub fn App() -> impl IntoView {
    let state = ApplicationState::provide();

    let do_logout = move |_| {
        state.log_out();
    };

    let username = Signal::derive(move || state.login_state().get().username());

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <Box style="display: flex; flex-direction: column; align-items: center; min-height: var(--leptonic-vh); min-width: 100%">
                <Router>
                    <AppBar height=Height::Em(3.0)>

                        <H3 style="margin-left: 1em; color: white;">
                            {move || username.get().map(|n| format!("Infrafish Mail Admin ({n})"))}
                        </H3>
                        <Stack
                            orientation=StackOrientation::Horizontal
                            spacing=Size::Em(1.0)
                            style="margin-right: 1em"
                        >
                            <Button on_click=do_logout variant=ButtonVariant::Flat>
                                <Icon icon=icondata::TbLogout/>
                            </Button>
                            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
                        </Stack>
                    </AppBar>
                    <Box style="display: flex; flex-direction: column; align-items: center; padding: 1em; min-width: 100%; min-height: calc(var(--leptonic-vh) - 3em);">
                        <PageRouter/>
                    </Box>
                </Router>
            </Box>
        </Root>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
