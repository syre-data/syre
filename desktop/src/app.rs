use crate::{
    pages::{
        auth::{Login, Logout, Register},
        project::Workspace,
        Index,
    },
    types::Messages,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

/// For Tailwind to include classes
/// they must appear as string literals in at least one place.
/// This array is used to include them when needed.
static _TAILWIND_CLASSES: &'static [&'static str] = &["hidden", "invisible"];

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(Messages::new()); // TODO: Only provide after user is logged in.

    view! {
        <Title formatter=|text| text text="Syre"/>

        <Router>
            <Routes>
                <Route path="" view=Index/>
                <Route path="register" view=move || view! { <Register/> }/>
                <Route path="login" view=Login/>
                <Route path="logout" view=Logout/>
                <Route path=":id" view=Workspace/>
            </Routes>
        </Router>
    }
}
