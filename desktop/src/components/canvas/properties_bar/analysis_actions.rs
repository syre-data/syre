//! Project actions detail widget bar.
use super::project_analyses::ProjectAnalyses;
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::analysis::{add_excel_template, add_script, remove_analysis};
use crate::components::canvas::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::hooks::use_project;
use std::collections::HashSet;
use std::path::PathBuf;
use syre_core::project::ExcelTemplate;
use syre_core::types::ResourceId;
use syre_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(ProjectAnalysisActions)]
pub fn project_analysis_actions() -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let project = use_project(&canvas_state.project);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let onadd_scripts = use_callback((project.rid.clone(), projects_state.clone()), {
        let app_state = app_state.dispatcher();
        move |paths: HashSet<PathBuf>, (project, projects_state)| {
            let project = project.clone();
            let projects_state = projects_state.clone();
            let Some(mut project_scripts) = projects_state.project_analyses.get(&project).cloned()
            else {
                let msg = Message::error("Could not load project's scripts.");
                app_state.dispatch(AppStateAction::AddMessage(msg));
                return;
            };

            let app_state = app_state.clone();
            spawn_local(async move {
                for path in paths {
                    let project = project.clone();
                    let script = match add_script(project.clone(), path).await {
                        Ok(script) => script,
                        Err(err) => {
                            let mut msg = Message::error("Could not create script {path:?}.");
                            msg.set_details(err);
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                            continue;
                        }
                    };

                    if let Some(script) = script {
                        project_scripts.insert(script.rid.clone(), script.into());
                    }
                }

                projects_state.dispatch(ProjectsStateAction::InsertProjectAnalyses {
                    project,
                    analyses: project_scripts,
                });
            });
        }
    });

    let onadd_excel_template = use_callback((project.rid.clone(), projects_state.clone()), {
        let app_state = app_state.dispatcher();
        move |mut template: ExcelTemplate, (project, projects_state)| {
            let project = project.clone();
            let projects_state = projects_state.clone();
            let app_state = app_state.clone();
            spawn_local(async move {
                let path = match add_excel_template(project.clone(), template.clone()).await {
                    Ok(template) => template,
                    Err(err) => {
                        let mut msg = Message::error("Could not create template script.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                template.template.path = path;
                projects_state.dispatch(ProjectsStateAction::InsertProjectExcelTemplate {
                    project,
                    template,
                });
            });
        }
    });

    let onremove_script = use_callback((project.rid.clone(), projects_state.clone()), {
        let app_state = app_state.dispatcher();
        let graph_state = graph_state.dispatcher();

        move |rid: ResourceId, (project, projects_state)| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let graph_state = graph_state.clone();
            let project = project.clone();

            let Some(mut analyses) = projects_state.project_analyses.get(&project).cloned() else {
                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                    "Could not remove script",
                )));
                return;
            };

            spawn_local(async move {
                match remove_analysis(project.clone(), rid.clone()).await {
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
                analyses.remove(&rid);
                projects_state
                    .dispatch(ProjectsStateAction::InsertProjectAnalyses { project, analyses });

                // Remove from containers
                graph_state.dispatch(GraphStateAction::RemoveContainerAnalysisAssociations(
                    rid.clone(),
                ));
            });
        }
    });

    Ok(html! {
        <ProjectAnalyses
            onadd={onadd_scripts}
            onadd_excel_template={onadd_excel_template}
            onremove={onremove_script} />
    })
}
