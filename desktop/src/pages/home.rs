//! Home component
use crate::app::{AppStateAction, AppStateReducer, AppWidget};
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use crate::hooks::{use_user, use_user_projects};
use crate::navigation::MainNavigation;
use crate::routes::Route;
use serde_wasm_bindgen as swb;
use thot_desktop_lib::settings::{UserAppState, UserSettings};
use thot_ui::types::Message;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

// **********************
// *** Home Component ***
// **********************

/// Home page for authenticated users.
#[function_component(HomeComponent)]
pub fn home_component() -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");
    let user = use_user();

    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user.")));
        return Ok(html! {});
    };

    let projects = use_user_projects(&user.rid);

    // create project
    let create_project = {
        let app_state = app_state.clone();

        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::CreateProject,
            )));
        })
    };

    Ok(html! {
        <div>
            if (*projects).len() == 0 {
                <div class={classes!("align-center")}>
                    <h2>{ "Get started" }</h2>
                    <div>
                        <button onclick={create_project.clone()}>{ "Create your first project" }</button>
                    </div>
                </div>
            } else {
                <Redirect<Route> to={Route::Dashboard} />
            }
        </div>
    })
}

// *****************
// *** Home Page ***
// *****************

// Wrapper for [`HomeComponent`] to handle suspense.
#[function_component(Home)]
pub fn home() -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");
    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user.")));
        return html! {};
    };

    {
        // user settings
        let app_state = app_state.clone();
        let rid = user.rid.clone();

        spawn_local(async move {
            let Ok(user_settings) = invoke("load_user_settings", ResourceIdArgs { rid }).await else {
                        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user settings.")));
                        return;
            };

            let user_settings: UserSettings = swb::from_value(user_settings)
                .expect("could not convert reuslt of `load_user_settings` to `UserSettings`");

            app_state.dispatch(AppStateAction::SetUserSettings(Some(user_settings)));
        });
    }

    {
        // user app state
        let app_state = app_state.clone();
        let navigator = navigator.clone();
        let rid = user.rid.clone();

        spawn_local(async move {
            let Ok(user_app_state) = invoke(
                "load_user_app_state",
                ResourceIdArgs { rid }
            )
            .await else {
                navigator.push(&Route::SignIn);
                app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user app state.")));
                return;
            };

            let user_app_state: UserAppState = swb::from_value(user_app_state)
                .expect("could not convert reuslt of `load_user_app_state` to `UserAppState`");

            app_state.dispatch(AppStateAction::SetUserAppState(Some(user_app_state)));
        });
    }

    let fallback = html! { <Loading text={"Loading projects"} /> };

    html! {
        <>
            <MainNavigation />
            <Suspense {fallback}>
                <HomeComponent />
            </Suspense>
        </>
    }
}

#[cfg(test)]
#[path = "./home_test.rs"]
mod home_test;
