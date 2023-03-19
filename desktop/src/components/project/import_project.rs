//! Import [`Project`].
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
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

// ********************************
// *** Import Project Component ***
// ********************************

type ContainerTree = ResourceTree<Container>;

/// Import project component.
#[function_component(ImportProject)]
pub fn import_project() -> Html {
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

            // import and go to project
            spawn_local(async move {
                let Ok(project) = invoke::<Project>("add_project", PathBufArgs { path: path.clone() })
                    .await else {
                        web_sys::console::error_1(&"could not add project".into());
                        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not import project")));
                        return;
                    };

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
#[path = "./import_project_test.rs"]
mod import_project_test;
