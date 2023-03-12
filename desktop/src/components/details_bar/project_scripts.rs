//! Project scripts editor.
use crate::app::ProjectsStateReducer;
use crate::hooks::use_canvas_project;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_ui::widgets::script::CreateScript;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectScriptsProps {
    /// Called when a user adds `Script`s to the `Project`.
    #[prop_or_default]
    pub onadd: Option<Callback<HashSet<PathBuf>>>,
}

#[function_component(ProjectScripts)]
pub fn project_scripts(props: &ProjectScriptsProps) -> HtmlResult {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_canvas_project();
    let Some(project_scripts) = projects_state.project_scripts.get(&*project) else {
        panic!("`Project`'s `Scripts` not loaded");
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
