//! Project scripts editor.
use crate::hooks::{use_canvas_project, use_project_scripts};
use std::path::PathBuf;
use thot_ui::widgets::script::CreateScript;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectScriptsProps {
    /// Called when a user adds a `Script` to the `Project`.
    #[prop_or_default]
    pub onadd: Option<Callback<PathBuf>>,
}

#[function_component(ProjectScripts)]
pub fn project_scripts(props: &ProjectScriptsProps) -> Html {
    let project = use_canvas_project();
    let project_scripts = use_project_scripts((*project).clone());

    html! {
        <div>
            <CreateScript
                oncreate={&props.onadd} />

            <ul>
                if let Some(project_scripts) = project_scripts.as_ref() {{
                    project_scripts.values().map(|script| html! {
                        <li key={script.rid.clone()}>
                            { format!("{:?}", script.path.as_path()) } // @todo: make better
                        </li>
                    }).collect::<Html>()
                }}
            </ul>
        </div>
    }
}

#[cfg(test)]
#[path = "./project_scripts_test.rs"]
mod project_scripts_test;
