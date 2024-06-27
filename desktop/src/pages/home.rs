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
    view! {
        <nav>
            <ol class="flex">
                <li>
                    <A href="">"Home"</A>
                </li>
                <li>
                    <A href="/logout">"Log out"</A>
                </li>
            </ol>
        </nav>
    }
}
