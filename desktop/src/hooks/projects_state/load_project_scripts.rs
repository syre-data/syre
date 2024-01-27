//! Gets a `Project`'s `Script`s.
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::script::get_project_scripts;
use thot_core::project::Script;
use thot_core::types::{ResourceId, ResourceMap};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

/// Loads the `Project`'s scripts into the `ProjectsState` reducer.
#[hook]
pub fn use_load_project_scripts(project: &ResourceId) -> SuspensionResult<()> {
    tracing::debug!(?project);
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    if projects_state.project_scripts.contains_key(project) {
        return Ok(());
    }

    let (s, handle) = Suspension::new();
    {
        let project = project.clone();
        let projects_state = projects_state.dispatcher();
        spawn_local(async move {
            let prj_scripts = match get_project_scripts(project.clone()).await {
                Ok(scripts) => scripts,
                Err(err) => {
                    tracing::debug!(err);
                    panic!("{err}");
                }
            };

            let prj_scripts = prj_scripts
                .into_iter()
                .map(|script| (script.rid.clone(), script))
                .collect::<ResourceMap<Script>>()
                .into();

            projects_state.dispatch(ProjectsStateAction::InsertProjectScripts(
                project.clone(),
                prj_scripts,
            ));

            handle.resume();
        });
    }

    Err(s)
}
