//! Create a new [`Project`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::PathBufArgs;
use crate::common::invoke;
use crate::routes::Route;
use serde_wasm_bindgen as swb;
use tauri_sys::dialog::FileDialogBuilder;
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

// ********************************
// *** Create Project Component ***
// ********************************

/// New project component.
/// Consists of three steps:
/// 1. Select folder
///     May select an existing folder, or create a new one.
/// 2. Assign properties with optional advanced properties.
/// 3. Build the project tree.
#[function_component(CreateProject)]
pub fn create_project() -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let navigator = use_navigator().expect("navigator not found");
    let path = use_state(|| None);

    let onclick = {
        let path = path.clone();

        Callback::from(move |_: MouseEvent| {
            let path = path.clone();

            spawn_local(async move {
                let prj_path = FileDialogBuilder::new()
                    .set_title("Select project directory")
                    .pick_folder()
                    .await;

                let prj_path = prj_path.expect("could not retrieve directory");
                path.set(prj_path);
            });
        })
    };

    let onsubmit = {
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();
        let navigator = navigator.clone();
        let path = path.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let navigator = navigator.clone();
            let path = path.clone();

            app_state.dispatch(AppStateAction::SetActiveWidget(None)); // close self

            // create and go to project
            spawn_local(async move {
                let path = (*path).clone().expect("path not set");
                // todo: Validate path is not already a project.
                // todo: Set project creator.
                let rid = invoke("init_project", PathBufArgs { path: path.clone() })
                    .await
                    .expect("could not invoke `init_project`");

                let _rid: ResourceId = swb::from_value(rid)
                    .expect("could not convert result of `init_project` to `ResourceId`");

                let project = invoke("load_project", PathBufArgs { path })
                    .await
                    .expect("could not invoke `load_project`");

                let project: CoreProject = swb::from_value(project)
                    .expect("could not convert result of `load_project` to `Project`");

                let rid = project.rid.clone();
                projects_state.dispatch(ProjectsStateAction::InsertProject(project));
                projects_state.dispatch(ProjectsStateAction::AddOpenProject(rid.clone()));
                projects_state.dispatch(ProjectsStateAction::SetActiveProject(rid));

                navigator.push(&Route::Workspace);
            });
        })
    };

    html! {
        <div>
            <form {onsubmit}>
                <div class={classes!("align-center")}>
                    <div style={"margin-bottom: 1em"}>
                        if let Some(path) = (*path).clone() {
                            { path.to_str().unwrap() }
                        } else {
                            { "Set project directory" }
                        }
                    </div>

                    <button type={"button"} {onclick}>
                        if path.is_none() {
                            { "Set" }
                        } else {
                            { "Change" }
                        }
                    </button>
                </div>
                <div class={classes!("align-center")}>
                    <button disabled={path.is_none()}>{ "Create" }</button>
                </div>
            </form>
        </div>
    }
}

#[cfg(test)]
#[path = "./create_project_test.rs"]
mod create_project_test;
