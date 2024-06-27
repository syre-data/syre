use leptos::*;
use leptos_router::use_navigate;

#[component]
pub fn Logout() -> impl IntoView {
    let navigate = use_navigate();
    navigate("/", Default::default());

    view! { "Logging out..." }
}
