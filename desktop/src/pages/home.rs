use crate::{components::Logo, pages::Dashboard};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;
use syre_core::system::User;

#[component]
pub fn Home(user: User) -> impl IntoView {
    provide_context(user);
    view! {
        <MainNav />
        <main>
            <Dashboard />
        </main>
    }
}

#[component]
fn MainNav() -> impl IntoView {
    view! {
        <nav class="px-2 border-b dark:bg-secondary-900 flex justify-between">
            <ol class="flex py-2">
                <li>
                    <A href="/">
                        <Logo class="h-4" />
                    </A>
                </li>
            </ol>

            <ol class="flex">
                <li>
                    <A href="/logout">
                        <Icon icon=icondata::IoLogOutOutline class="[&_*]:dark:!stroke-white h-4" />
                    </A>
                </li>
            </ol>
        </nav>
    }
}
