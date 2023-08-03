use super::auth_guard::AuthGuard;
use crate::pages::{
    authenticate::SignIn, authenticate::SignUp, dashboard::Dashboard, home::Home, index::Index,
    not_found::NotFound, settings::Settings, workspace::Workspace,
};
use yew::prelude::*;
use yew_router::prelude::*;

// Routes
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    /// Initial page, reroute to desired location.
    #[at("/")]
    Index,

    #[at("/signin")]
    SignIn,

    #[at("/signup")]
    SignUp,

    /// Initial page for authenticated users.
    #[at("/home")]
    Home,

    /// User's settings.
    #[at("/settings")]
    Settings,

    /// Home dashboard.
    #[at("/dashboard")]
    Dashboard,

    /// All user projects.
    #[at("/projects")]
    Projects,

    /// Projects workspace.
    #[at("/workspace")]
    Workspace,

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Index => html! { <Index /> },
        Route::SignIn => html! { <SignIn /> },
        Route::SignUp => html! { <SignUp /> },
        Route::Home => html! { <AuthGuard><Home /></AuthGuard> },
        Route::Settings => html! { <AuthGuard><Settings /></AuthGuard> },
        Route::Dashboard => html! { <AuthGuard><Dashboard /></AuthGuard> },
        Route::Projects => html! { <AuthGuard><Dashboard /></AuthGuard> }, // @todo: Create dedicated page.
        Route::Workspace => html! { <AuthGuard><Workspace /></AuthGuard> },
        Route::NotFound => html! { <NotFound /> },
    }
}
