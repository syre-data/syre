//! Create a new [`Project`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::PathBufArgs;
use crate::commands::graph::InitProjectGraphArgs;
use crate::commands::project::UpdateProjectArgs;
use crate::common::invoke;
use crate::hooks::use_user;
use crate::routes::Route;
use std::path::{Path, PathBuf};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container, Project};
use thot_core::types::{Creator, ResourceId, UserId};
use thot_local::types::ProjectSettings;
use thot_ui::components::{file_selector::FileSelectorProps, FileSelector, FileSelectorAction};
use thot_ui::types::Message;
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
#[tracing::instrument]
#[function_component(CreateProject)]
pub fn create_project() -> Html {
    let navigator = use_navigator().expect("navigator not found");
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let user = use_user();
    let Some(user) = user.as_ref() else {
        navigator.push(&Route::SignIn);
        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not get user.")));
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

            // create and go to project
            spawn_local(async move {
                // TODO[m]: Validate path is not already a project.
                // init project
                let Ok(_rid) = invoke::<ResourceId>("init_project", PathBufArgs { path: path.clone() })
                    .await else {
                        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not create project")));
                        return;
                    };

                let Ok((mut project, settings)) =
                    invoke::<(Project, ProjectSettings)>("load_project", PathBufArgs { path: path.clone() })
                        .await else {
                        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not load project")));
                        return;
                    };

                // initialize data root as container
                let mut data_root_abs = path.clone();
                let data_root_rel = Path::new("data");
                data_root_abs.push(data_root_rel.clone());

                // TODO[l]: Could load graph here.
                let Ok(_graph) = invoke::<ContainerTree>(
                    "init_project_graph",
                    InitProjectGraphArgs {
                        path: data_root_abs.clone(),
                        project: project.rid.clone(),
                    },
                ).await else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not create project graph.")));
                    return;
                };

                // save project
                project.data_root = Some(data_root_rel.to_path_buf());
                project.creator = Creator::User(Some(UserId::Id(user)));
                tracing::debug!(?project);
                let Ok(_) = invoke::<()>(
                    "update_project",
                    UpdateProjectArgs {
                        project: project.clone(),
                    },
                )
                .await else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not update project")));
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
