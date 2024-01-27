//! Home dashboard.
use crate::app::{
    AppStateAction, AppStateReducer, AppWidget, ProjectsStateAction, ProjectsStateReducer,
};
use crate::hooks::{use_user, use_user_projects};
use crate::navigation::MainNavigation;
use crate::routes::Route;
use std::str::FromStr;
use thot_core::types::ResourceId;
use thot_ui::widgets::project::ProjectDeck;
use thot_ui::widgets::suspense::Loading;
use yew::prelude::*;
use yew::virtual_dom::Key;
use yew_router::prelude::*;

/// Dashboard for user's [`Project`]s.
#[function_component(DashboardView)]
pub fn dashboard_view() -> HtmlResult {
    let navigator = use_navigator().unwrap();
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        return Ok(html! {{ "Redirecting to login" }});
    };

    let projects = use_user_projects(&user.rid);
    let create_project = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_: MouseEvent, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::CreateProject,
            )))
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

    let init_project = use_callback((), {
        let app_state = app_state.dispatcher();
        move |_: MouseEvent, _| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::InitializeProject,
            )));
        }
    });

    let onclick_card = use_callback((), {
        let navigator = navigator.clone();
        let projects_state = projects_state.dispatcher();

        move |rid: Key, _| {
            let rid = ResourceId::from_str(&rid.to_string()).expect("invalid `ResourceId`");

            projects_state.dispatch(ProjectsStateAction::AddOpenProject(rid.clone()));
            projects_state.dispatch(ProjectsStateAction::SetActiveProject(rid.clone()));
            navigator.push(&Route::Workspace);
        }
    });

    Ok(html! {
        <>
            <MainNavigation />
            <div id={"dashboard"}>
                    <div id={"dashboard-container"}>
                        <div id={"dashboard-header"}>
                            <h1 class={"title"}>
                                { "Dashboard" }
                            </h1>
                            <div>
                                <button
                                    class={"btn-primary"}
                                    title={"Create a new project."}
                                    onclick={create_project}>

                                    { "New" }
                                </button>
                            </div>
                            <div>
                                <button
                                    class={"btn-secondary"}
                                    title={"Initialize an existing folder as a project."}
                                    onclick={init_project}>

                                    { "Initialize" }
                                </button>
                            </div>
                            <div>
                                <button
                                    class={"btn-secondary"}
                                    title={"Import an existing project."}
                                    onclick={import_project}>

                                    { "Import" }
                                </button>
                            </div>
                        </div>

                        <ProjectDeck items={(*projects).clone()} {onclick_card} />
                    </div>
            </div>
        </>
    })
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let fallback = html! { <Loading text={"Loading resources"} />  };

    html! {
        <Suspense {fallback}>
            <DashboardView />
        </Suspense>
    }
}
