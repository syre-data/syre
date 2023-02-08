//! Project hook with suspense.
use crate::app::ProjectsStateReducer;
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[hook]
pub fn use_project(rid: &ResourceId) -> UseStateHandle<Option<CoreProject>> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_state(|| projects_state.projects.get(&rid).cloned());

    {
        let rid = rid.clone();
        let projects_state = projects_state.clone();
        let project = project.clone();

        use_effect_with_deps(
            move |projects_state| {
                project.set(projects_state.projects.get(&rid).cloned());
            },
            projects_state,
        );
    }

    project
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
