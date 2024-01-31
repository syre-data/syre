//! New user sign up.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::authenticate::{authenticate_user, create_user};
use crate::commands::user::set_active_user;
use crate::routes::Route;
use syre_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[tracing::instrument]
#[function_component(SignUp)]
pub fn sign_up() -> Html {
    let auth_state = use_context::<AuthStateReducer>().unwrap();
    let app_state = use_context::<AppStateReducer>().unwrap();
    let allow_submit = use_state(|| true);
    let navigator = use_navigator().unwrap();
    let email = use_node_ref();
    let name = use_node_ref();

    let onsubmit = use_callback((email.clone(), name.clone()), {
        let auth_state = auth_state.dispatcher();
        let app_state = app_state.dispatcher();
        let allow_submit = allow_submit.setter();
        let navigator = navigator.clone();

        move |e: web_sys::SubmitEvent, (email, name)| {
            e.prevent_default();

            let auth_state = auth_state.clone();
            let app_state = app_state.clone();
            let allow_submit = allow_submit.clone();
            let navigator = navigator.clone();

            allow_submit.set(false);

            // get input values
            let email = email
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast input elm");

            let name = name
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast input elm");

            let email = email.value().trim().to_string();
            let name_val = name.value().trim().to_string();
            let name = if name_val.is_empty() {
                None
            } else {
                Some(name_val)
            };

            spawn_local(async move {
                // create user account
                let user = match create_user(email, name).await {
                    Ok(user) => user,
                    Err(err) => {
                        tracing::debug!(?err);
                        let mut msg = Message::error("Could not create user.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        allow_submit.set(true);
                        return;
                    }
                };

                // authenticate user
                let user = match authenticate_user(user.email).await {
                    Ok(user) => user,
                    Err(err) => {
                        let mut msg = Message::error("Could not authenticate user.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        allow_submit.set(true);
                        return;
                    }
                };

                auth_state.dispatch(AuthStateAction::SetUser(user.clone()));
                if let Some(user) = user {
                    navigator.push(&Route::Home);

                    if let Err(_err) = set_active_user(user.rid).await {
                        // TODO Handle error from set_active_user.
                    };
                }

                allow_submit.set(true);
            });
        }
    });

    html! {
        <>
        <h1>{ "Sign Up" }</h1>
        <div>
            <form class={classes!("align-center")} {onsubmit}>
                <div>
                    <input ref={email}
                        type={"email"}
                        class={"mx-m"}
                        placeholder={"Email"}
                        required={true} />

                    <input ref={name}
                        type={"text"}
                        class={"mx-m"}
                        placeholder={"Name"} />
                </div>
                <div class={"mt-1rem"}>
                    <button disabled={!*allow_submit}>{ "Get started!" }</button>
                </div>
            </form>
            <div class={"align-center mt-2rem"}>
                <Link<Route> to={Route::SignIn}>{ "Sign in" }</Link<Route>>
            </div>
        </div>
        </>
    }
}
