//! Project hook with suspense.
use crate::app::ProjectsStateReducer;
use syre_core::project::Project;
use syre_core::types::ResourceId;
use yew::prelude::*;

/// Get the [`Project`] with the given id.
#[hook]
pub fn use_project(rid: &ResourceId) -> UseStateHandle<Option<Project>> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = use_state(|| projects_state.projects.get(&rid).cloned());
    use_effect_with(projects_state.clone(), {
        let rid = rid.clone();
        let project = project.clone();
        move |projects_state| {
            project.set(projects_state.projects.get(&rid).cloned());
        }
    });

    project
}
