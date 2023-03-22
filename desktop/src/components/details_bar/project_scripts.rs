//! Project scripts editor.
use crate::app::ProjectsStateReducer;
use crate::hooks::use_canvas_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_ui::widgets::script::CreateScript;
use yew::prelude::*;

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
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_canvas_project();
    let Some(project_scripts) = projects_state.project_scripts.get(&*project) else {
        panic!("`Project`'s `Scripts` not loaded");
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
                            let path = script.path.as_path().clone();
                            let file_name = path.file_name().expect("could not get file name");
                            let name = file_name.to_string_lossy().to_string();

                            name
                        }
                    };

                    html! {
                        <li key={script.rid.clone()}>
                            { name }
                            if props.onremove.is_some() {
                                <button onclick={onclick_remove(script.rid.clone())}>{ "x" }</button>
                            }
                        </li>
                    }
                }).collect::<Html>() }
            </ul>
        </div>
    })
}

#[cfg(test)]
#[path = "./project_scripts_test.rs"]
mod project_scripts_test;
