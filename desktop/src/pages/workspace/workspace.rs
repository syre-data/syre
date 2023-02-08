//! Project workspace.
use crate::components::canvas::ProjectCanvas;
use crate::hooks::{use_active_project, use_open_projects};
use yew::prelude::*;

/// Project workspace.
#[function_component(Workspace)]
pub fn workspace() -> Html {
    let open_projects = use_open_projects();
    let active_project = use_active_project();

    if open_projects.is_empty() {
        return html! {
            { "No project open" }
        };
    }

    html! {
        <div id={"workspace"}>
            { open_projects.iter().map(|rid| {
                let mut class = Classes::new();
                if let Some(active_project) = active_project.as_ref() {
                    if rid == active_project {
                        class.push("active");
                    }
                }

                html! {
                    <ProjectCanvas
                        key={rid.clone()}
                        project={rid.clone()}
                        {class} />
                }
            }).collect::<Html>() }
        </div>
    }
}

#[cfg(test)]
#[path = "./workspace_test.rs"]
mod workspace_test;
