//! Index page.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::user::{get_active_user, set_active_user};
use crate::routes::Route;
use syre_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[tracing::instrument]
#[function_component(Index)]
pub fn index() -> Html {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");

    // initialize auth state
    {
        let auth_state = auth_state.clone();
        let app_state = app_state.clone();
        let navigator = navigator.clone();

        // TODO Check if any users exist. If not redirect to sign up page
        //  instead of sign in page.
        use_effect_with(auth_state, |auth_state| {
            let auth_state = auth_state.clone();
            spawn_local(async move {
                let Ok(active_user) = get_active_user().await else {
                    navigator.push(&Route::SignIn);
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not get user.",
                    )));
                    return;
                };

                match active_user {
                    None => navigator.push(&Route::SignIn),
                    Some(user) => {
                        // set active user on backend
                        match set_active_user(user.rid.clone()).await {
                            Ok(_) => {
                                // set active user on front end
                                auth_state.dispatch(AuthStateAction::SetUser(Some(user)));
                                navigator.push(&Route::Home);
                            }
                            Err(err) => {
                                let mut msg = Message::error("Could not set user.");
                                msg.set_details(format!("{err:?}"));
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }
                        };
                    }
                }
            });
        });
    }

    // default to sign in page
    html! {
       <Redirect<Route> to={Route::SignIn} />
    }
}
