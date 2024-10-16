//! Gets open projects.
use crate::app::ProjectsStateReducer;
use indexmap::IndexSet;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[hook]
pub fn use_open_projects() -> UseStateHandle<IndexSet<ResourceId>> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let open_projects = use_state(|| projects_state.open_projects.clone());

    {
        let projects_state = projects_state.clone();
        let open_projects = open_projects.clone();

        use_effect_with(projects_state, move |projects_state| {
            open_projects.set(projects_state.open_projects.clone());
        });
    };

    open_projects
}
