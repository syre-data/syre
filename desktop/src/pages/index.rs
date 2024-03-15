//! Index page.
use std::io;

use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::user::{get_active_user, set_active_user};
use crate::routes::Route;
use syre_local::error::{Error as LocalError, IoSerde as IoSerdeError};
use syre_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Index)]
pub fn index() -> Html {
    let auth_state = use_context::<AuthStateReducer>().unwrap();
    let app_state = use_context::<AppStateReducer>().unwrap();
    let navigator = use_navigator().unwrap();

    // initialize auth state
    // TODO: Check if any users exist. If not redirect to sign up page
    //  instead of sign in page.
    use_effect_with((), {
        let auth_state = auth_state.dispatcher();
        let app_state = app_state.dispatcher();
        let navigator = navigator.clone();

        move |_| {
            let auth_state = auth_state.clone();
            let app_state = app_state.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let active_user = match get_active_user().await {
                    Ok(user) => user,
                    Err(err) => {
                        match err {
                            LocalError::IoSerde(IoSerdeError::Io(io::ErrorKind::NotFound)) => {}
                            _ => {
                                let mut msg = Message::error("Could not get user.");
                                msg.set_details(format!("{err:?}"));
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }
                        }

                        navigator.push(&Route::SignIn);
                        return;
                    }
                };

                match active_user {
                    None => navigator.push(&Route::SignIn),
                    Some(user) => {
                        // TODO: Backend user should be set when `get_user` called.
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
        }
    });

    // default to sign in page
    html! {
       <Redirect<Route> to={Route::SignIn} />
    }
}
