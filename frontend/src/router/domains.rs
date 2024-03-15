use std::convert::identity;

use api_types::domains::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;

use crate::state::ApplicationState;

#[component]
pub fn DomainList() -> impl IntoView {
    let state = ApplicationState::acquire();

    let domain_list = create_resource(
        move || state.login_state().get().api_token(),
        move |token| async move {
            match token {
                None => None,
                Some(token) => state
                    .api_get::<ListDomainResponse>(&token, "/domain/list")
                    .await
                    .ok(),
            }
            .unwrap_or_else(|| ListDomainResponse {
                domains: Default::default(),
            })
        },
    );

    view! {
        <Box style="display: flex; flex-direction: row; justify-content: flex-start; align-items: flex-start; width: 100%">
            <Drawer
                side=DrawerSide::Left
                shown=true
                style="overflow-y: scroll; padding: 0.5em; background-color: var(--brand-color); border-right: 1px solid gray;"
            >
                <Stack spacing=Size::Em(
                    0.5,
                )>
                    {move || match domain_list.get() {
                        None => view! {}.into_view(),
                        Some(domain_list) => {
                            view! { <DomainStack domains=domain_list/> }.into_view()
                        }
                    }}

                </Stack>
            </Drawer>
            <Box style="padding: 0.5em; display: flex; flex-direction: column; overflow-y: scroll; width: 100%; height: 100%;">
                <Outlet/>
            </Box>
        </Box>
    }
}

#[component]
pub fn DomainStack(domains: ListDomainResponse) -> impl IntoView {
    domains
        .domains
        .keys()
        .map(|k| {
            let entries_href = format!("{k}/entries");
            let title = k.clone();
            let flags_href = format!("{k}/flags");

            view! {
                <span style="color: var(--std-text-dark);">
                    <A href=move || entries_href.clone()>{title}</A>
                    " ("
                    <A href=move || flags_href.clone()>"Flags"</A>
                    ")"
                </span>
            }
        })
        .collect_view()
}

#[component]
pub fn DomainSelected() -> impl IntoView {
    view! {
        <Box style="display: flex; flex-direction: row; justify-content: flex-start; align-items: flex-start; width: 100%">
            <Outlet/>
        </Box>
    }
}

#[component]
pub fn NoDomainSelected() -> impl IntoView {
    view! { <P>"Please select a domain on the left"</P> }
}

#[component]
pub fn PickDomainPart() -> impl IntoView {
    view! {
        <P>"Please select from:"</P>
        <ul>
            <li>
                <A href="entries">"Domain entries"</A>
            </li>
            <li>
                <A href="flags">"Domain flags"</A>
            </li>
        </ul>
    }
}

#[derive(Params, Default, PartialEq, Clone)]
struct DomainSelector {
    domain: Option<String>,
}

#[component]
pub fn DomainFlags() -> impl IntoView {
    let domain = use_params::<DomainSelector>();
    let domain =
        Signal::derive(move || domain.get().unwrap_or_default().domain.unwrap_or_default());
    let state = ApplicationState::acquire();
    let domain_flags = create_resource(
        move || (state.login_state().get().api_token(), domain.get()),
        move |(token, domain)| async move {
            match token {
                None => None,
                Some(token) => {
                    let req = SetDomainFlagsRequest {
                        domain_name: domain,
                        ..Default::default()
                    };
                    state
                        .api_post::<ListDomainResponseEntry, SetDomainFlagsRequest>(
                            &token,
                            "/domain/set-flags",
                            &req,
                        )
                        .await
                        .ok()
                }
            }
        },
    );

    let saved_flags =
        Signal::derive(move || domain_flags.get().and_then(identity).unwrap_or_default());
    let flags = create_rw_signal(ListDomainResponseEntry::default());

    create_effect(move |_| flags.set(domain_flags.get().and_then(identity).unwrap_or_default()));

    let not_changed = Signal::derive(move || saved_flags.get() == flags.get());

    let do_save = create_action(move |args: &(String, String, ListDomainResponseEntry)| {
        let (token, domain, flags) = args;
        let req = SetDomainFlagsRequest {
            domain_name: domain.clone(),
            sender_verify: Some(flags.sender_verify),
            grey_listing: Some(flags.grey_listing),
            virus_check: Some(flags.virus_check),
            spamcheck_threshold: Some(flags.spamcheck_threshold),
            ..Default::default()
        };
        let token = token.clone();
        async move {
            state
                .api_post::<ListDomainResponseEntry, SetDomainFlagsRequest>(
                    &token,
                    "/domain/set-flags",
                    &req,
                )
                .await
                .ok();
        }
    });

    create_effect(move |prev| match (prev, do_save.version().get()) {
        (None, version) => version,
        (Some(prev), version) => {
            if prev != version {
                domain_flags.refetch();
            }
            version
        }
    });

    let trigger_save = move |_| {
        do_save.dispatch((
            state.login_state().get().api_token().unwrap_or_default(),
            domain.get(),
            flags.get(),
        ));
    };

    view! {
        <Stack spacing=Size::Em(0.5) style="align-items: flex-start;">
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5)>
                "Sender Verification"
                <Toggle
                    state=Signal::derive(move || flags.get().sender_verify)
                    set_state=move |v| flags.update(|f| f.sender_verify = v)
                />
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5)>
                "Greylisting"
                <Toggle
                    state=Signal::derive(move || flags.get().grey_listing)
                    set_state=move |v| flags.update(|f| f.grey_listing = v)
                />
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5)>
                "Virus Checking"
                <Toggle
                    state=Signal::derive(move || flags.get().virus_check)
                    set_state=move |v| flags.update(|f| f.virus_check = v)
                />
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5)>
                "Spam Rejection Threshold"
                <Slider
                    style="min-width: 25em;"
                    min=0.0
                    max=250.0
                    step=1.0
                    value=Signal::derive(move || flags.get().spamcheck_threshold as f64)
                    set_value=move |v| flags.update(|f| f.spamcheck_threshold = v as i32)
                    marks=SliderMarks::Automatic {
                        create_names: false,
                    }

                    value_display=move |v| format!("{:.1}", v / 10.0)
                />
            </Stack>
            <Button variant=ButtonVariant::Filled on_click=trigger_save disabled=not_changed>
                "Save changes"
            </Button>
        </Stack>
    }
}

#[component]
pub fn DomainEntriesList() -> impl IntoView {
    view! {}
}
