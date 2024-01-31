//! Home component
use crate::app::{AppStateAction, AppStateReducer, AppWidget};
use crate::commands::settings::{load_user_app_state, load_user_settings};
use crate::hooks::{use_user, use_user_projects};
use crate::navigation::MainNavigation;
use crate::routes::Route;
use syre_ui::types::Message;
use syre_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

// **********************
// *** Home Component ***
// **********************

/// Home page for authenticated users.
#[tracing::instrument]
#[function_component(HomeComponent)]
pub fn home_component() -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let navigator = use_navigator().unwrap();
    let user = use_user();

    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        return Ok(html! {});
    };

    let projects = use_user_projects(&user.rid);
    let create_project = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_: MouseEvent, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::CreateProject,
            )));
        }
    });

    let initialize_project = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_: MouseEvent, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::InitializeProject,
            )));
        }
    });

    let import_project = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_: MouseEvent, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::ImportProject,
            )));
        }
    });

    Ok(html! {
        <div>
            if projects.len() == 0 {
                <div class={"align-center"}>
                    <h2>{ "Get started" }</h2>
                    <div class={"mb-1rem"}>
                        <button class={"btn-primary"} onclick={create_project}>{ "Create your first project" }</button>
                    </div>
                    <div class={"mb-1rem"}>
                        <button class={"btn-secondary"} onclick={initialize_project}>{ "Initialize an existing folder" }</button>
                    </div>
                    <div>
                        <button class={"btn-secondary"} onclick={import_project}>{ "Import a project" }</button>
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
    let app_state = use_context::<AppStateReducer>().unwrap();
    let navigator = use_navigator().unwrap();
    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        app_state.dispatch(AppStateAction::AddMessage(Message::error(
            "Could not get user.",
        )));
        return html! {};
    };

    {
        let app_state = app_state.clone();
        let navigator = navigator.clone();
        let rid = user.rid.clone();

        use_effect_with((), move |_| {
            let navigator = navigator.clone();
            let app_state = app_state.clone();
            let rid = rid.clone();

            spawn_local(async move {
                let user_app_state = match load_user_app_state(rid.clone()).await {
                    Ok(app_state) => app_state,
                    Err(err) => {
                        navigator.push(&Route::SignIn);
                        let mut msg = Message::error("Could not get user app state.");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                let user_settings = match load_user_settings(rid).await {
                    Ok(settings) => settings,
                    Err(err) => {
                        let mut msg = Message::error("Could not get user settings.");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                app_state.dispatch(AppStateAction::SetUserAppState(Some(user_app_state)));
                app_state.dispatch(AppStateAction::SetUserSettings(Some(user_settings)));
            });
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
