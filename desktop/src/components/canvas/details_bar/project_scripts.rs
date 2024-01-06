//! Project scripts editor.
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::common::open_file;
use crate::commands::project::get_project_path;
use crate::components::excel_template::CreateExcelTemplate;
use crate::hooks::use_canvas_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_desktop_lib::excel_template;
use thot_ui::types::Message;
use thot_ui::widgets::script::CreateScript;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct ProjectScriptsProps {
    /// Called when a user adds `Script`s to the `Project`.
    #[prop_or_default]
    pub onadd: Option<Callback<HashSet<PathBuf>>>,

    /// Called when a user adds Excel Templates to the `Project`.
    #[prop_or_default]
    pub onadd_excel_template: Option<Callback<excel_template::ExcelTemplate>>,

    /// Called when a user removes a `Script`.
    #[prop_or_default]
    pub onremove: Option<Callback<ResourceId>>,
}

#[function_component(ProjectScripts)]
pub fn project_scripts(props: &ProjectScriptsProps) -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_canvas_project();
    let Some(project_scripts) = projects_state.project_scripts.get(&*project) else {
        panic!("`Project`'s `Scripts` not loaded");
    };

    let ondblclick_script = {
        let app_state = app_state.clone();
        let projects_state = projects_state.clone();
        let project = project.clone();

        move |script: ResourceId| {
            let app_state = app_state.clone();
            let project = projects_state.projects.get(&*project).unwrap();
            let pid = project.rid.clone();
            let analysis_root = project.analysis_root.clone().unwrap();
            let script_path = projects_state
                .project_scripts
                .get(&project.rid)
                .unwrap()
                .get(&script)
                .unwrap()
                .path
                .clone();

            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let app_state = app_state.clone();
                let pid = pid.clone();
                let analysis_root = analysis_root.clone();
                let script_path = script_path.clone();

                spawn_local(async move {
                    let mut path = match get_project_path(pid).await {
                        Ok(path) => path,
                        Err(err) => {
                            let mut msg = Message::error("Could not get project path");
                            msg.set_details(err);
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                            return;
                        }
                    };

                    path.push(analysis_root);
                    path.push(script_path.as_path());
                    match open_file(path).await {
                        Ok(_) => {}
                        Err(err) => {
                            let mut msg = Message::error("Could not open file");
                            msg.set_details(err);
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                            return;
                        }
                    };
                });
            })
        }
    };

    let onclick_remove = {
        let onremove = props.onremove.clone();
        move |rid: ResourceId| {
            let onremove = onremove.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let onremove = onremove.clone();
                if let Some(onremove) = onremove {
                    onremove.emit(rid.clone());
                }
            })
        }
    };

    Ok(html! {
        <div class={"project-scripts-widget"}>
            if let Some(onadd) = props.onadd.as_ref() {
                <CreateScript oncreate={onadd.clone()} />
            }
            if let Some(onadd_template) = props.onadd_excel_template.as_ref() {
                <CreateExcelTemplate oncreate={onadd_template.clone()} />
            }

            <ul>
                { project_scripts.values().map(|script| {
                    let name = match script.name.as_ref() {
                        Some(name) => name.clone(),
                        None => {
                            let path = script.path.as_path();
                            let file_name = path.file_name().expect("could not get file name");
                            let name = file_name.to_string_lossy().to_string();

                            name
                        }
                    };

                    html! {
                        <li key={script.rid.clone()}>
                            <span class={"name clickable"}
                                title={name.clone()}
                                ondblclick={ondblclick_script(script.rid.clone())}>
                                { name }
                            </span>

                            if props.onremove.is_some() {
                                <button class={"btn-icon"} type={"button"}
                                    onclick={onclick_remove(script.rid.clone())}>

                                    <Icon class={"thot-ui-add-remove-icon"}
                                        icon_id={IconId::HeroiconsSolidMinus}/>
                                </button>
                            }
                        </li>
                    }
                }).collect::<Html>() }
            </ul>
        </div>
    })
}
