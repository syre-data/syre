//! Project actions detail widget bar.
use super::project_scripts::ProjectScripts;
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::script::AddScriptArgs;
use crate::common::invoke;
use crate::components::canvas::CanvasStateReducer;
use crate::hooks::use_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::project::Script;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(ProjectActions)]
pub fn project_actions() -> HtmlResult {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project = use_project(&canvas_state.project);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let onadd_scripts = {
        let projects_state = projects_state.clone();
        let project = project.rid.clone();

        Callback::from(move |paths: HashSet<PathBuf>| {
            let project = project.clone();
            let projects_state = projects_state.clone();
            let Some(mut project_scripts) = projects_state.project_scripts.get(&project).cloned() else {
                panic!("`Project`'s `Scripts` not loaded");
            };

            spawn_local(async move {
                for path in paths {
                    let project = project.clone();
                    let script = invoke::<Script>(
                        "add_script",
                        AddScriptArgs {
                            project: project.clone(),
                            path,
                        },
                    )
                    .await
                    .expect("could not invoke `add_script`");

                    project_scripts.insert(script.rid.clone(), script);
                }

                projects_state.dispatch(ProjectsStateAction::InsertProjectScripts(
                    project,
                    project_scripts,
                ));
            });
        })
    };

    Ok(html! {
        <div>
            <ProjectScripts onadd={onadd_scripts} />
        </div>
    })
}

#[cfg(test)]
#[path = "./project_actions_test.rs"]
mod project_actions_test;
