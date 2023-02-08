//! Route guard.
//! Used to protect route access, redirecting if predicate is not fulfilled.
use yew::prelude::*;
use yew_router::prelude::*;

/// Properties to setup the route guard.
#[derive(Properties, PartialEq)]
pub struct RouteGuardProps<R>
where
    R: Routable,
{
    /// Whether to render the children or redirect.
    /// Renders children if `true`, otherwise redirects.
    pub predicate: bool,

    /// Children to render if predicate is `true`.
    #[prop_or_default]
    pub children: Children,

    /// Path to redirect to if predicate is `false`.
    pub redirect: R,
}

/// Route gurad for children.
/// Render children if `predicate` is `true`,
/// otherwise redirects.
#[function_component(RouteGuard)]
pub fn route_guard<R>(props: &RouteGuardProps<R>) -> Html
where
    R: Routable,
{
    let navigator = use_navigator().expect("navigation not found");

    if !props.predicate {
        navigator.push(&props.redirect);
        return html! {};
    }

    html! {
        { for props.children.iter() }
    }
}

#[cfg(test)]
#[path = "./route_guard_test.rs"]
mod route_guard_test;
