//! Index page.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::common::{EmptyArgs, ResourceIdArgs};
use crate::common::invoke;
use crate::routes::Route;
use thot_core::system::User;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

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

        use_effect_with_deps(
            |auth_state| {
                let auth_state = auth_state.clone();
                spawn_local(async move {
                    let Ok(active_user) = invoke::<Option<User>>("get_active_user", EmptyArgs {}).await else {
                        navigator.push(&Route::SignIn);
                        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user.")));
                        return;
                    };

                    match active_user {
                        None => navigator.push(&Route::SignIn),
                        Some(user) => {
                            // set acitve user on backend
                            let _active_res = invoke::<()>(
                                "set_active_user",
                                ResourceIdArgs {
                                    rid: user.rid.clone(),
                                },
                            )
                            .await;

                            // set active user on front end
                            auth_state.dispatch(AuthStateAction::SetUser(Some(user)));
                            navigator.push(&Route::Home);
                        }
                    }
                });
            },
            auth_state,
        );
    }

    // default to sign in page
    html! {
       <Redirect<Route> to={Route::SignIn} />
    }
}

#[cfg(test)]
#[path = "./index_test.rs"]
mod index_test;
