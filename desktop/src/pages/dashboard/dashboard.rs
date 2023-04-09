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
use yew::prelude::*;
use yew::virtual_dom::Key;
use yew_router::prelude::*;

/// Dashboard for user's [`Project`](CoreProject)s.
#[function_component(Dashboard)]
pub fn dashboard() -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let navigator = use_navigator().expect("navigator not found");

    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        return Ok(html! {{ "Redirecting to login" }});
    };

    let projects = use_user_projects(&user.rid);

    // create project
    let create_project = {
        let app_state = app_state.clone();

        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::CreateProject,
            )))
        })
    };

    // import project
    let import_project = {
        let app_state = app_state.clone();

        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(AppStateAction::SetActiveWidget(Some(
                AppWidget::ImportProject,
            )));
        })
    };

    let navigator = use_navigator().expect("navigator not found");
    let onclick_card = {
        let navigator = navigator.clone();

        // Opens the project and navigates to the workspace if needed.
        Callback::from(move |rid: Key| {
            let rid = ResourceId::from_str(&rid.to_string()).expect("invalid `ResourceId`");

            projects_state.dispatch(ProjectsStateAction::AddOpenProject(rid.clone()));
            projects_state.dispatch(ProjectsStateAction::SetActiveProject(rid.clone()));
            navigator.push(&Route::Workspace);
        })
    };

    Ok(html! {
        <>
            <MainNavigation />
            <div id={"dashboard"}>
                if projects.len() == 0 {
                    <div class={classes!("align-center")}>
                        <h2>{ "Get started" }</h2>
                        <div>
                            <button onclick={create_project.clone()}>{ "Create your first project" }</button>
                        </div>
                        <div>
                            <button class={classes!("btn-secondary")} onclick={import_project.clone()}>{ "Import project" }</button>
                        </div>
                    </div>
                } else {
                    <div id={"dashboard-container"}>
                        <div id={"dashboard-header"}>
                            <h1 class={classes!("title")}>
                                { "Dashboard" }
                            </h1>
                            <div class={classes!("new-project")}>
                                <button
                                    class={classes!("btn-primary")}
                                    onclick={create_project}>

                                    { "New" }
                                </button>
                            </div>
                            <div class={classes!("import-project")}>
                                <button
                                    class={classes!("btn-secondary")}
                                    onclick={import_project}>

                                    { "Import" }
                                </button>
                            </div>
                        </div>
                        <ProjectDeck items={(*projects).clone()} {onclick_card} />
                    </div>
                }
            </div>
        </>
    })
}

#[cfg(test)]
#[path = "./dashboard_test.rs"]
mod dashboard_test;
