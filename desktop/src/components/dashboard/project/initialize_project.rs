use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::project::{init_project_from, load_project};
use std::path::PathBuf;
use syre_ui::components::{file_selector::FileSelectorProps, FileSelector, FileSelectorAction};
use syre_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;

#[function_component(InitializeProject)]
pub fn initialize_project() -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let onsuccess = use_callback((), {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.dispatcher();

        move |path: PathBuf, _| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();

            app_state.dispatch(AppStateAction::SetActiveWidget(None)); // close self
            spawn_local(async move {
                match init_project_from(path.clone()).await {
                    Ok(_rid) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not create project");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                let (project, settings) = match load_project(path).await {
                    Ok(project) => project,
                    Err(err) => {
                        let mut msg = Message::error("Could not load project");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                // update ui
                projects_state.dispatch(ProjectsStateAction::InsertProject((project, settings)));
            });
        }
    });

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
