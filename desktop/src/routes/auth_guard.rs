//! Authetnication guard.
//! Route guard that verifies user authentication.
use crate::app::AuthStateReducer;
use crate::routes::Route;
use thot_ui::components::route_guard::RouteGuard;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AuthGuardProps {
    /// Children to render if user is authenticated.
    #[prop_or_default]
    pub children: Children,

    /// Route to redirect to if user is not authenticated.
    /// If not provided, redirects to [`Route::Index`].
    #[prop_or(Route::Index)]
    pub redirect: Route,
}

#[function_component(AuthGuard)]
pub fn auth_guard(props: &AuthGuardProps) -> Html {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    html! {
        <RouteGuard<Route>
            predicate={auth_state.is_authenticated()}
            redirect={props.redirect.clone()}>

            { for props.children.iter() }
        </RouteGuard<Route>>
    }
}

#[cfg(test)]
#[path = "./auth_guard_test.rs"]
mod auth_guard_test;
