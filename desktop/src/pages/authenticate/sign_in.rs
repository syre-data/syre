//! User sign in.
use crate::app::{AppStateAction, AppStateReducer, AuthStateAction, AuthStateReducer};
use crate::commands::authenticate::authenticate_user;
use crate::commands::user::set_active_user;
use crate::routes::Route;
use syre_ui::components::Message as MessageUi;
use syre_ui::types::Message;
use syre_ui::types::MessageType;
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
                let user = match authenticate_user(email).await {
                    Ok(user) => user,
                    Err(err) => {
                        let mut msg = Message::error("Could not authenticate user.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
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
                        let active_res = set_active_user(user.rid).await;

                        if let Err(err) = active_res {
                            let mut msg = Message::error("Could not set active user.");
                            msg.set_details(format!("{err:?}"));
                            app_state.dispatch(AppStateAction::AddMessage(msg));
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

    // TODO: Copy email value to sign up if link clicked.
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
                <input type={"email"}
                    class={"mx-m"}
                    placeholder={"Email"}
                    onchange={email_on_change} />

                <button class={"mx-m"}>{ "Sign in" }</button>
            </form>
            <div class={"align-center mt-2rem"}>
                <Link<Route> to={Route::SignUp}>{ "Sign up" }</Link<Route>>
            </div>
        </div>
        </>
    }
}
