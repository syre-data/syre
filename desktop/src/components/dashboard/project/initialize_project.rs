use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::PathBufArgs;
use crate::common::invoke;
use crate::hooks::use_user;
use crate::routes::Route;
use std::path::PathBuf;
use thot_core::project::Project;
use thot_core::types::ResourceId;
use thot_local::types::ProjectSettings;
use thot_ui::components::{file_selector::FileSelectorProps, FileSelector, FileSelectorAction};
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

#[function_component(InitializeProject)]
pub fn initialize_project() -> Html {
    let navigator = use_navigator().expect("navigator not found");
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        app_state.dispatch(AppStateAction::AddMessage(Message::error(
            "Could not get user.",
        )));
        return html! {};
    };

    let onsuccess = {
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();
        let navigator = navigator.clone();
        let user = user.rid.clone();

        Callback::from(move |path: PathBuf| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let navigator = navigator.clone();
            let user = user.clone();

            app_state.dispatch(AppStateAction::SetActiveWidget(None)); // close self
            spawn_local(async move {
                let Ok(_rid) =
                    invoke::<ResourceId>("init_project_from", PathBufArgs { path: path.clone() })
                        .await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not create project",
                    )));
                    return;
                };

                let Ok((mut project, settings)) = invoke::<(Project, ProjectSettings)>(
                    "load_project",
                    PathBufArgs { path: path.clone() },
                )
                .await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not load project",
                    )));
                    return;
                };

                // update ui
                let rid = project.rid.clone();
                projects_state.dispatch(ProjectsStateAction::InsertProject((project, settings)));
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
