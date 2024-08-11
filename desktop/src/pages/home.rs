use crate::pages::Dashboard;
use leptos::*;
use leptos_router::*;
use syre_core::system::User;

#[component]
pub fn Home(user: User) -> impl IntoView {
    provide_context(user);
    view! {
        <MainNav/>
        <main>
            <Dashboard/>
        </main>
    }
}

#[component]
fn MainNav() -> impl IntoView {
    let prefers_dark = leptos_use::use_preferred_dark();
    let home_icon_src = move || {
        if prefers_dark() {
            "/public/logos/logo-white-icon.svg"
        } else {
            "/public/logos/logo-black-icon.svg"
        }
    };

    view! {
        <nav>
            <ol class="flex">
                <li>
                    <A href="/">
                        <img src=home_icon_src class="h-4"/>
                    </A>
                </li>
            </ol>
        </nav>
    }
}
