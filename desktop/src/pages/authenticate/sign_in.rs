//! User sign in.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::authenticate::UserCredentials;
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use crate::routes::Route;
use thot_core::system::User;
use thot_ui::components::Message as MessageUi;
use thot_ui::types::Message;
use thot_ui::types::MessageType;
use tracing::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[tracing::instrument]
#[function_component(SignIn)]
pub fn sign_in() -> Html {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");
    let email = use_state(|| String::new());
    let invalid_credentials = use_state(|| false);

    let onsubmit = {
        let auth_state = auth_state.clone();
        let app_state = app_state.clone();
        let navigator = navigator.clone();
        let email = email.clone();
        let invalid_credentials = invalid_credentials.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            let auth_state = auth_state.clone();
            let app_state = app_state.clone();
            let navigator = navigator.clone();
            let email = (*email).clone();
            let invalid_credentials = invalid_credentials.clone();

            spawn_local(async move {
                debug!(email);
                let Ok(user) =
                    invoke::<Option<User>>("authenticate_user", UserCredentials { email }).await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not authenticate user.",
                    )));
                    return;
                };

                if user.is_none() {
                    // TODO Alert user.
                    // user not found, alert user
                    invalid_credentials.set(true);
                } else {
                    // user found, sign in
                    navigator.push(&Route::Home);
                    auth_state.dispatch(AuthStateAction::SetUser(user.clone()));
                    if let Some(user) = user {
                        let active_res =
                            invoke::<()>("set_active_user", ResourceIdArgs { rid: user.rid }).await;

                        if active_res.is_err() {
                            app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                "Could not set active user.",
                            )));
                        }
                    }
                }
            });
        })
    };

    // @todo: Possibly use `use_node_ref` to get value when needed instead of updating
    let email_on_change = {
        let email = email.clone();
        Callback::from(move |e: Event| {
            let target = e.target().expect("email change even should have target");
            let target = target
                .dyn_ref::<web_sys::HtmlInputElement>()
                .expect("cast email to input elm should work");

            let value = target.value();
            email.set(value);
        })
    };

    // @todo: Copy email value to sign up if link clicked.
    html! {
        <>
        <h1>{ "Sign In" }</h1>
        <div>
            <form class={classes!("align-center")} {onsubmit}>
                if *invalid_credentials {
                    <div>
                        <MessageUi kind={MessageType::Error} message={"Invalid credentials"} />
                    </div>
                }
                <input type={"email"} placeholder={"Email"} onchange={email_on_change}/>
                <button>{ "Sign in" }</button>
            </form>
            <div class={classes!("align-center")} style={ "margin-top: 2em;" }>
                <Link<Route> to={Route::SignUp}>{ "Sign up" }</Link<Route>>
            </div>
        </div>
        </>
    }
}
