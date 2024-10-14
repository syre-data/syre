use crate::{
    components::{self, Logo},
    pages::{Dashboard, Settings},
    types,
};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;
use syre_core::system::User;

#[derive(Clone, Copy, derive_more::Deref, derive_more::From)]
struct ShowSettings(RwSignal<bool>);
impl ShowSettings {
    pub fn new() -> Self {
        Self(create_rw_signal(false))
    }
}

#[component]
pub fn Home(user: User) -> impl IntoView {
    provide_context(user);
    let show_settings = ShowSettings::new();
    provide_context(show_settings);

    view! {
        <div class="relative">
            <MainNav />
            <main>
                <div>
                    <Dashboard />
                </div>
                <div
                    class=(["-right-full", "left-full"], move || !show_settings())
                    class=(["right-0", "left-0"], move || show_settings())
                    class="absolute top-0 bottom-0 transition-absolute-position"
                >
                    <Settings onclose=move |_| show_settings.set(false) />
                </div>
            </main>
        </div>
    }
}

#[component]
fn MainNav() -> impl IntoView {
    let show_settings = expect_context::<ShowSettings>();
    let open_settings = move |e: ev::MouseEvent| {
        if e.button() != types::MouseButton::Primary {
            return;
        }

        show_settings.set(true);
    };

    view! {
        <nav class="px-2 border-b dark:bg-secondary-900 flex justify-between">
            <ol class="flex py-2">
                <li>
                    <A href="/">
                        <Logo class="h-4" />
                    </A>
                </li>
            </ol>

            <ol class="flex gap-2">
                <li class="py-2">
                    <button
                        on:mousedown=open_settings
                        type="button"
                        class="hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded"
                    >
                        <Icon icon=components::icon::Settings />
                    </button>
                </li>
                <li>
                    <A href="/logout">
                        <Icon icon=icondata::IoLogOutOutline class="[&_*]:dark:!stroke-white h-4" />
                    </A>
                </li>
            </ol>
        </nav>
    }
}
