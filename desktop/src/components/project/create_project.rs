//! Create a new [`Project`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::PathBufArgs;
use crate::commands::graph::InitProjectGraphArgs;
use crate::commands::project::UpdateProjectArgs;
use crate::common::invoke;
use crate::routes::Route;
use std::path::PathBuf;
use thot_core::graph::ResourceTree;
use thot_core::project::{Container, Project};
use thot_core::types::ResourceId;
use thot_ui::components::{file_selector::FileSelectorProps, FileSelector, FileSelectorAction};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

// ********************************
// *** Create Project Component ***
// ********************************

type ContainerTree = ResourceTree<Container>;

/// New project component.
/// Consists of three steps:
/// 1. Select folder
///     May select an existing folder, or create a new one.
/// 2. Assign properties with optional advanced properties.
/// 3. Build the project tree.
#[function_component(CreateProject)]
pub fn create_project() -> Html {
    let navigator = use_navigator().expect("navigator not found");
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let onsuccess = {
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();
        let navigator = navigator.clone();

        Callback::from(move |path: PathBuf| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let navigator = navigator.clone();

            app_state.dispatch(AppStateAction::SetActiveWidget(None)); // close self

            // create and go to project
            spawn_local(async move {
                // todo: Validate path is not already a project.
                // todo: Set project creator.
                // init project
                let rid = invoke::<ResourceId>("init_project", PathBufArgs { path: path.clone() })
                    .await
                    .expect("could not invoke `init_project`");

                let mut project =
                    invoke::<Project>("load_project", PathBufArgs { path: path.clone() })
                        .await
                        .expect("could not invoke `load_project`");

                // initialize data root as container
                let mut data_root = path.clone();
                data_root.push("data");

                let rid = invoke::<ResourceId>(
                    "init_project_graph",
                    InitProjectGraphArgs {
                        path: data_root.clone(),
                        project: project.rid.clone(),
                    },
                )
                .await
                .expect("could not invoke `init_project_graph`");

                // save project
                project.data_root = Some(data_root);
                let res = invoke::<()>(
                    "update_project",
                    UpdateProjectArgs {
                        project: project.clone(),
                    },
                )
                .await
                .expect("could not invoke `update_project`");

                // update ui
                let rid = project.rid.clone();
                projects_state.dispatch(ProjectsStateAction::InsertProject(project));
                projects_state.dispatch(ProjectsStateAction::AddOpenProject(rid.clone()));
                projects_state.dispatch(ProjectsStateAction::SetActiveProject(rid));

                navigator.push(&Route::Workspace);
            });
        })
    };

    let oncancel = {
        let app_state = app_state.clone();

        Callback::from(move |_| {
            app_state.dispatch(AppStateAction::SetActiveWidget(None));
        })
    };

    let props = props! {
        FileSelectorProps {
            title: "Select project directory",
            action: FileSelectorAction::PickFolder,
            show_cancel: false,
            onsuccess,
            oncancel,
        }
    };

    html! {
        <FileSelector ..props />
    }
}

#[cfg(test)]
#[path = "./create_project_test.rs"]
mod create_project_test;
