//! Project scripts editor.
use crate::hooks::{use_canvas_project, use_project_scripts};
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
pub fn project_scripts(props: &ProjectScriptsProps) -> Html {
    let project = use_canvas_project();
    let project_scripts = use_project_scripts((*project).clone());

    html! {
        <div class={classes!("project-scripts-widget")}>
            if let Some(onadd) = props.onadd.as_ref() {
                <CreateScript oncreate={onadd.clone()} />
            }

            <ul>
                if let Some(project_scripts) = project_scripts.as_ref() {{
                    project_scripts.values().map(|script| {
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
                    }).collect::<Html>()
                }}
            </ul>
        </div>
    }
}

#[cfg(test)]
#[path = "./project_scripts_test.rs"]
mod project_scripts_test;
