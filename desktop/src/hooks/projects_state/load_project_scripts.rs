//! Gets a `Project`'s `Script`s.
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use thot_core::project::Script;
use thot_core::types::{ResourceId, ResourceMap};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

#[hook]
pub fn use_load_project_scripts(project: &ResourceId) -> SuspensionResult<()> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    match projects_state.project_scripts.get(project) {
        Some(_) => Ok(()),
        None => {
            let project = project.clone();
            let (s, handle) = Suspension::new();

            spawn_local(async move {
                let prj_scripts = invoke::<Vec<Script>>(
                    "get_project_scripts",
                    ResourceIdArgs {
                        rid: project.clone(),
                    },
                )
                .await
                .expect("could not invoke `get_project_scripts`");

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
            Err(s)
        }
    }
}

#[cfg(test)]
#[path = "./load_project_scripts_test.rs"]
mod load_project_scripts_test;
