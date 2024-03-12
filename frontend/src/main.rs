use leptonic::prelude::*;
use leptos::*;

use crate::{router::PageRouter, state::ApplicationState};

mod router;
mod state;

#[component]
pub fn App() -> impl IntoView {
    let state = ApplicationState::provide();

    let do_logout = move |_| {
        state.log_out();
    };

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <Box style="display: flex; flex-direction: column; align-items: center; min-height: var(--leptonic-vh); min-width: 100%">

                <AppBar height=Height::Em(3.0)>

                    <H3 style="margin-left: 1em; color: white;">"Infrafish Mailconfig"</H3>
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

                <PageRouter/>
            </Box>
        </Root>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
