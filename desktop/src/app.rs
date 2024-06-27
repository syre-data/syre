use crate::pages::{
    auth::{Login, Logout, Register},
    project::Workspace,
    Index,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
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
