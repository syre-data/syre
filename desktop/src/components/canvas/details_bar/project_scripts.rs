//! Project scripts editor.
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::common::{PathBufArgs, ResourceIdArgs};
use crate::common::invoke;
use crate::hooks::use_canvas_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::types::ResourceId;
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
                    let Ok(mut path) =
                        invoke::<PathBuf>("get_project_path", ResourceIdArgs { rid: pid }).await
                    else {
                        app_state.dispatch(AppStateAction::AddMessage(Message::error(
                            "Could not get project path",
                        )));
                        return;
                    };

                    path.push(analysis_root);
                    path.push(script_path.as_path());

                    let Ok(_) = invoke::<()>("open_file", PathBufArgs { path }).await else {
                        app_state.dispatch(AppStateAction::AddMessage(Message::error(
                            "Could not open file",
                        )));
                        return;
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
        <div class={classes!("project-scripts-widget")}>
            if let Some(onadd) = props.onadd.as_ref() {
                <CreateScript oncreate={onadd.clone()} />
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
                            <span class={classes!("clickable")}
                                ondblclick={ondblclick_script(script.rid.clone())}>

                                { name }
                            </span>
                            if props.onremove.is_some() {
                                <button class={classes!("btn-icon")} type={"button"}
                                    onclick={onclick_remove(script.rid.clone())}>

                                    <Icon class={classes!("thot-ui-add-remove-icon")}
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
