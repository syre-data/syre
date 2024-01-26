//! Project actions detail widget bar.
use super::project_scripts::ProjectScripts;
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::script::{add_script, remove_script};
use crate::components::canvas::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::hooks::use_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(ProjectActions)]
pub fn project_actions() -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state = use_context::<GraphStateReducer>().expect("`GraphStateReducer` not found");

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
            let Some(mut project_scripts) = projects_state.project_scripts.get(&project).cloned()
            else {
                panic!("`Project`'s `Scripts` not loaded");
            };

            spawn_local(async move {
                for path in paths {
                    let project = project.clone();
                    let script = match add_script(project.clone(), path).await {
                        Ok(script) => script,
                        Err(err) => {
                            tracing::debug!(err);
                            panic!("{err}");
                        }
                    };

                    if let Some(script) = script {
                        project_scripts.insert(script.rid.clone(), script);
                    }
                }

                projects_state.dispatch(ProjectsStateAction::InsertProjectScripts(
                    project,
                    project_scripts,
                ));
            });
        })
    };

    let onremove_script = {
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();
        let graph_state = graph_state.clone();
        let project = project.rid.clone();

        Callback::from(move |rid: ResourceId| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let graph_state = graph_state.clone();
            let project = project.clone();

            let Some(mut scripts) = projects_state.project_scripts.get(&project).cloned() else {
                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                    "Could not remove script",
                )));
                return;
            };

            spawn_local(async move {
                match remove_script(project.clone(), rid.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not remove script");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));

                        return;
                    }
                };

                // NOTE Program order does not follow logical order, should first remove from
                // containers and then projects, else containers might contain stale scripts.
                // however doing the right order makes the program crash due to `Script` not
                // found error.

                // Remove from scripts
                scripts.remove(&rid);
                projects_state
                    .dispatch(ProjectsStateAction::InsertProjectScripts(project, scripts));

                // Remove from containers
                graph_state.dispatch(GraphStateAction::RemoveContainerScriptAssociations(
                    rid.clone(),
                ));
            });
        })
    };

    Ok(html! {
        <ProjectScripts
            onadd={onadd_scripts}
            onremove={onremove_script} />
    })
}
