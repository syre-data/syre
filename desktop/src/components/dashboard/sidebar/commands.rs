//! Main navigation for authenticated users.
use crate::app::{AppStateAction, AppStateReducer, AppWidget, AuthStateAction, AuthStateReducer};
use crate::commands::user::unset_active_user;
use crate::routes::Route;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

/// Main navigation for authenticated users.
#[function_component(Commands)]
pub fn commands() -> Html {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found:");
    let navigator = use_navigator().expect("no navigator found");

    let logout = {
        let auth_state = auth_state.clone();
        let app_state = app_state.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let app_state = app_state.clone();
            navigator.push(&Route::SignIn);
            auth_state.dispatch(AuthStateAction::UnsetUser);

            spawn_local(async move {
                match unset_active_user().await {
                    Ok(_) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not log out");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        })
    };

    let user_settings = {
        let app_state = app_state.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::UserSettings,
            )))
        })
    };

    html! {
        <div>
            <ul>
                <li>
                    <button class={classes!("text-only")} onclick={user_settings}>
                        { "\u{2699} My Settings" }
                    </button>
                </li>
                <li>
                    <button class={classes!("button-secondary")} onclick={logout}>
                        { "Log out" }
                    </button>
                </li>
            </ul>
        </div>
    }
}
