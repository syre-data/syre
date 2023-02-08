//! Project actions detail widget bar.
use super::project_scripts::ProjectScripts;
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::script::AddScriptArgs;
use crate::common::invoke;
use crate::components::canvas::CanvasStateReducer;
use crate::hooks::{use_project, use_project_scripts};
use serde_wasm_bindgen as swb;
use std::path::PathBuf;
use thot_core::project::Script as CoreScript;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(ProjectActions)]
pub fn project_actions() -> Html {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project = use_project(&canvas_state.project);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let project_scripts = use_project_scripts(canvas_state.project.clone());

    let onadd_script = {
        let projects_state = projects_state.clone();
        let project_scripts = project_scripts.clone();
        let project = project.rid.clone();

        Callback::from(move |path: PathBuf| {
            let projects_state = projects_state.clone();
            let project_scripts = project_scripts.clone();
            let project = project.clone();

            spawn_local(async move {
                let script = invoke(
                    "add_script",
                    AddScriptArgs {
                        project: project.clone(),
                        path,
                    },
                )
                .await
                .expect("could not invoke `add_script`");

                let script: CoreScript = swb::from_value(script)
                    .expect("could not convert result of `add_script` to `Script`");

                let Some(mut scripts) = (*project_scripts).clone() else {
                    panic!("`Project` `Script`s not loaded");
                };

                scripts.insert(script.rid.clone(), script);
                projects_state
                    .dispatch(ProjectsStateAction::InsertProjectScripts(project, scripts));
            });
        })
    };

    html! {
        <div>
            <ProjectScripts onadd={onadd_script} />
        </div>
    }
}

#[cfg(test)]
#[path = "./project_actions_test.rs"]
mod project_actions_test;
