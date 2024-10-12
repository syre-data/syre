use super::{Home, Landing};
use crate::types::Messages;
use futures::StreamExt;
use leptos::*;
use syre_core::system::User;
use syre_desktop_lib as lib;
use syre_local::error::IoSerde;

#[component]
pub fn Index() -> impl IntoView {
    let active_user = create_resource(|| (), |_| async move { fetch_user().await });

    view! {
        <Suspense fallback=Initializing>
            <ErrorBoundary fallback=|errors| {
                view! { <ActiveUserErrors errors /> }
            }>
                {move || {
                    active_user().map(|user| user.map(|user| view! { <IndexView user /> }))
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ActiveUserErrors(errors: RwSignal<Errors>) -> impl IntoView {
    tracing::error!(?errors);

    view! { <div class="text-lg text-center p-4">"An error occurred."</div> }
}

#[component]
fn IndexView(user: Option<User>) -> impl IntoView {
    let (user, set_user) = create_signal(user);
    spawn_local(async move {
        let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(lib::event::topic::USER)
            .await
            .unwrap();

        while let Some(events) = listener.next().await {
            tracing::debug!(?events);
            for event in events.payload {
                let lib::EventKind::User(user) = event.kind() else {
                    panic!("invalid event kind");
                };

                set_user(user.clone());
            }
        }
    });

    view! {
        <Show when=move || { user.with(|user| user.is_some()) } fallback=|| view! { <Landing /> }>
            <Home user=user().unwrap() />
        </Show>
    }
}

#[component]
fn Initializing() -> impl IntoView {
    view! { <div class="text-center pt-4">"Initializing app"</div> }
}

async fn fetch_user() -> Result<Option<User>, IoSerde> {
    tauri_sys::core::invoke_result("active_user", ()).await
}
