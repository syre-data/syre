use crate::pages::{Dashboard, Settings};
use leptos::{either::either, ev::MouseEvent, prelude::*};
use leptos_icons::Icon;
use leptos_router::components::A;
use syre_core::system::User;
use syre_desktop_lib as lib;
use syre_desktop_ui_components::Logo;
use syre_desktop_ui_lib as ui_lib;
use syre_local as local;

#[derive(Clone, Copy, derive_more::Deref, derive_more::From)]
struct ShowSettings(RwSignal<bool>);
impl ShowSettings {
    pub fn new() -> Self {
        Self(RwSignal::new(false))
    }
}

#[component]
pub fn Home(user: User) -> impl IntoView {
    provide_context(user);
    let user_settings = LocalResource::new(fetch_user_settings);
    view! {
        <Suspense fallback=Loading>
            {move || Suspend::new(async move {
                either!(
                    user_settings.await,
                    None => view! { <NoSettings /> },
                    Some(user_settings) => view! { <HomeView user_settings=user_settings.clone() /> },
                )
            })}
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="text-center pt-4">"Loading home"</div> }
}

#[component]
fn NoSettings() -> impl IntoView {
    let messages = expect_context::<ui_lib::message::Messages>();
    let navigate = leptos_router::hooks::use_navigate();

    let msg = ui_lib::message::Builder::error("Could not get user settings.");
    messages.push_message(msg.build());
    navigate("/login", Default::default());

    view! {
        <div class="text-center pt-4">
            <p>"Could not get user settings."</p>
            <p>"Redirecting to login."</p>
        </div>
    }
}

#[component]
fn HomeView(user_settings: lib::settings::user::Settings) -> impl IntoView {
    let messages = expect_context::<ui_lib::message::Messages>();
    provide_context(ui_lib::state::settings::User::new_store(
        user_settings.clone(),
    ));
    let show_settings = ShowSettings::new();
    provide_context(show_settings);

    match (user_settings.desktop, user_settings.runner) {
        (Ok(_), Ok(_)) => {}
        (Err(err), Ok(_)) => {
            let msg = ui_lib::message::Builder::error("Could not load desktop settings.");
            let msg = msg.body(format!("{err:?}"));
            messages.push_message(msg.build_str());
        }
        (Ok(_), Err(err)) => {
            let msg = ui_lib::message::Builder::error("Could not load runner settings.");
            let msg = msg.body(format!("{err:?}"));
            messages.push_message(msg.build_str());
        }
        (Err(desktop), Err(runner)) => {
            let msg = ui_lib::message::Builder::error("Could not load settings.");
            let msg = msg.body(LoadSettingsError { desktop, runner });
            messages.push_message(msg.build());
        }
    }

    view! {
        <div class="relative h-full w-full flex flex-col">
            <MainNav />
            <main class="grow min-h-0">
                <Dashboard />
                <div
                    class=(["-right-full", "left-full"], move || !show_settings())
                    class=(["right-0", "left-0"], move || show_settings())
                    class="absolute top-0 bottom-0 transition-absolute-position"
                >
                    <Settings onclose=move || show_settings.set(false) />
                </div>
            </main>
        </div>
    }
}

#[component]
fn MainNav() -> impl IntoView {
    let show_settings = expect_context::<ShowSettings>();
    let open_settings = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        show_settings.set(true);
    };

    view! {
        <nav class="px-2 border-b dark:bg-secondary-900 flex justify-between">
            <ol>
                <li>
                    <A href="/" attr:class="inline-block align-middle" attr:title="Dashboard">
                        <Logo attr:class="h-4" />
                    </A>
                </li>
            </ol>

            <ol class="inline-flex gap-2 items-center">
                <li>
                    <button
                        on:mousedown=open_settings
                        type="button"
                        class="align-middle p-1 hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded \
                        border border-transparent hover:border-secondary-200 dark:hover:border-white cursor-pointer"
                        title="Settings"
                    >
                        <Icon icon=ui_lib::icon::Settings />
                    </button>
                </li>
                <li>
                    <A
                        href="/logout"
                        attr:class="inline-block align-middle p-1 hover:bg-secondary-100 dark:hover:bg-secondary-800 \
                        rounded border border-transparent hover:border-secondary-200 dark:hover:border-white"
                        attr:title="Log out"
                    >
                        <Icon
                            icon=icondata::IoLogOutOutline
                            attr:class="dark:**:stroke-white! h-4"
                        />
                    </A>
                </li>
            </ol>
        </nav>
    }
}

async fn fetch_user_settings() -> Option<lib::settings::user::Settings> {
    tauri_sys::core::invoke("user_settings", ()).await
}

#[derive(Clone)]
struct LoadSettingsError {
    desktop: local::error::IoSerde,
    runner: local::error::IoSerde,
}
impl ui_lib::message::AsAnyView for LoadSettingsError {
    fn as_any_view(&self) -> AnyView {
        view! {
            <ul>
                <li>"Desktop: " {format!("{:?}", self.desktop)}</li>
                <li>"Runner: " {format!("{:?}", self.runner)}</li>
            </ul>
        }
        .into_any()
    }
}
