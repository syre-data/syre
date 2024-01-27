//! Import [`Project`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::project;
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
    let navigator = use_navigator().unwrap();
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let onsuccess = use_callback((app_state.dispatcher(), projects_state.dispatcher()), {
        let navigator = navigator.clone();
        move |path: PathBuf, (app_state, projects_state)| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let navigator = navigator.clone();

            app_state.dispatch(AppStateAction::SetActiveWidget(None)); // close self

            // import and go to project
            spawn_local(async move {
                match project::import_project(path.clone()).await {
                    Ok(project) => project,
                    Err(err) => {
                        let mut msg = Message::error("Could not import project");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                let (project, settings) = match project::load_project(path.clone()).await {
                    Ok(project) => project,
                    Err(err) => {
                        let mut msg = Message::error("Could not load project");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                projects_state.dispatch(ProjectsStateAction::InsertProject((project, settings)));
            });
        }
    });

    let oncancel = use_callback(app_state.dispatcher(), move |_, app_state| {
        app_state.dispatch(AppStateAction::SetActiveWidget(None));
    });

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
