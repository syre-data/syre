//! New user sign up.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::authenticate::{CreateUserArgs, UserCredentials};
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use crate::routes::Route;
use thot_core::system::User;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[tracing::instrument]
#[function_component(SignUp)]
pub fn sign_up() -> Html {
    let auth_state = use_context::<AuthStateReducer>().expect("`AuthStateReducer` not found");
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");
    let email = use_node_ref();
    let name = use_node_ref();

    let onsubmit = {
        let auth_state = auth_state.clone();
        let app_state = app_state.clone();
        let navigator = navigator.clone();
        let email = email.clone();
        let name = name.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            let auth_state = auth_state.clone();
            let app_state = app_state.clone();
            let navigator = navigator.clone();
            let email = email.clone();
            let name = name.clone();

            spawn_local(async move {
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

                // create user account
                let Ok(user) = invoke::<User>(
                    "create_user",
                    CreateUserArgs { email, name }
                )
                .await else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not create user.")));
                    return;
                };

                // authenticate user
                let Ok(user) = invoke::<Option<User>>(
                    "authenticate_user",
                    UserCredentials { email: user.email }
                )
                .await else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not authenticate user.")));
                    return;
                };

                auth_state.dispatch(AuthStateAction::SetUser(user.clone()));

                if let Some(user) = user {
                    // set user as active
                    navigator.push(&Route::Home);

                    // @todo[1]: Handle error from set_active_user.
                    let _active_res =
                        invoke::<()>("set_active_user", ResourceIdArgs { rid: user.rid }).await;
                }
            });
        })
    };

    html! {
        <>
        <h1>{ "Sign Up" }</h1>
        <div>
            <form class={classes!("align-center")} {onsubmit}>
                <div>
                    <input ref={email} type={"email"} placeholder={"Email"} required={true} />
                    <input ref={name} type={"text"} placeholder={"Name"} />
                </div>
                <div style={ "margin-top: 1em" }>
                    <button>{ "Get started!" }</button>
                </div>
            </form>
            <div style={"text-align: center; margin-top: 2em;"}>
                <Link<Route> to={Route::SignIn}>{ "Sign in" }</Link<Route>>
            </div>
        </div>
        </>
    }
}
