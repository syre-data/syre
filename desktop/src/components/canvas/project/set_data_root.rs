//! Set the data root of a project.
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::graph::init_project_graph;
use crate::commands::project::{get_project_path, update_project};
use crate::hooks::use_project;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_ui::components::file_selector::FileSelectorProps;
use thot_ui::components::{FileSelector, FileSelectorAction};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;

#[derive(Properties, PartialEq)]
pub struct SetDataRootProps {
    pub project: ResourceId,

    /// Called after the form has been successfully submitted.
    #[prop_or_default]
    pub onsuccess: Callback<()>,

    /// Called if selection is canceled.
    #[prop_or_default]
    pub oncancel: Callback<()>,
}

/// Component to set the [`Project`](CoreProject)'s `data_root`.
#[function_component(SetDataRoot)]
pub fn set_data_root(props: &SetDataRootProps) -> Html {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_project(&props.project);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let project_path = use_state(|| None);
    {
        // get project directory
        let project_path = project_path.clone();
        let pid = props.project.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let path = match get_project_path(pid).await {
                    Ok(path) => path,
                    Err(err) => {
                        tracing::debug!(err);
                        panic!("{err}");
                    }
                };

                project_path.set(Some(path));
            })
        })
    }

    let onsuccess = {
        let onsuccess = props.onsuccess.clone();
        let projects_state = projects_state.clone();
        let project = project.clone();

        Callback::from(move |path: PathBuf| {
            let onsuccess = onsuccess.clone();
            let projects_state = projects_state.clone();
            let mut project = project.clone();

            {
                // initialize data root as container
                let project = project.rid.clone();
                let path = path.clone();
                spawn_local(async move {
                    match init_project_graph(project, path).await {
                        Ok(_rid) => {}
                        Err(err) => {
                            tracing::debug!(err);
                            panic!("{err}");
                        }
                    }
                });
            }

            project.data_root = Some(path);
            {
                // save project
                let onsuccess = onsuccess.clone();
                let project = project.clone();
                let projects_state = projects_state.clone();

                spawn_local(async move {
                    match update_project(project.clone()).await {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::debug!(?err);
                            panic!("{err:?}");
                        }
                    }

                    projects_state.dispatch(ProjectsStateAction::UpdateProject(project));
                    onsuccess.emit(());
                });
            }
        })
    };

    let props = props! {
        FileSelectorProps {
            title: "Select data root",
            default_path: (*project_path).clone(),
            action: FileSelectorAction::PickFolder,
            show_cancel: false,
            onsuccess,
            oncancel: props.oncancel.clone(),
        }
    };

    html! {
        if project_path.is_some() {
            <FileSelector ..props />
        } else {
            { "Loading" }
        }
    }
}
