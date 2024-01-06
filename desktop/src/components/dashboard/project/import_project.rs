//! Import [`Project`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::project::add_project;
use crate::routes::Route;
use std::path::PathBuf;
use thot_ui::components::{file_selector::FileSelectorProps, FileSelector, FileSelectorAction};
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

// ********************************
// *** Import Project Component ***
// ********************************

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
                let project = match add_project(path.clone()).await {
                    Ok(project) => project,
                    Err(err) => {
                        let mut msg = Message::error("Could not import project");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                // update ui
                let rid = project.0.rid.clone();
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
