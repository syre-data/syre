use super::{Home, Landing};
use crate::invoke::invoke_result;
use futures::StreamExt;
use leptos::*;
use syre_core::system::User;
use syre_desktop_lib as lib;
use syre_local::error::IoSerde;

#[component]
pub fn Index() -> impl IntoView {
    let active_user = create_resource(|| (), |_| async move { fetch_user().await });
    let fallback = |errors| {
        tracing::debug!(?errors);
        view! { <div>"An error occurred"</div> }
    };

    view! {
        <Suspense fallback=Initializing>
            <ErrorBoundary fallback>
                {move || { active_user().map(|user| user.map(|user| view! { <IndexView user/> })) }}

            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn IndexView(user: Option<User>) -> impl IntoView {
    let (user, set_user) = create_signal(user);
    spawn_local(async move {
        let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(lib::event::topic::USER)
            .await
            .unwrap();

        while let Some(event) = listener.next().await {
            for event in event.payload {
                let lib::EventKind::User(user) = event.kind() else {
                    panic!("invalid event kind");
                };

                set_user(user.clone());
            }
        }
    });

    view! {
        <Show when=move || { user.with(|user| user.is_some()) } fallback=|| view! { <Landing/> }>
            <Home user=user().unwrap()/>
        </Show>
    }
}

#[component]
fn Initializing() -> impl IntoView {
    view! { <div>"Initializing app"</div> }
}

async fn fetch_user() -> Result<Option<User>, IoSerde> {
    invoke_result("active_user", ()).await
}
