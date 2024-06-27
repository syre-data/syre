//! 404 Not Found page.
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
    <>
        <h1>{ "Not found" }</h1>
        <p>{ "Oops... Looks like we couldn't find the page you were looking for." }</p>
        <div>
            <Link<Route> to={Route::Index}>{ "Home" }</Link<Route>>
        </div>
    </>
    }
}
