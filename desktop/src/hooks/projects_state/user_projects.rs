//! Retrieves a user's projects.
use crate::app::{projects_state::ProjectMap, ProjectsStateReducer};
use thot_core::project::Project as CoreProject;
use thot_core::types::{Creator, ResourceId, UserId};
use yew::prelude::*;

/// Retrieve a user's projects.
#[hook]
pub fn use_user_projects(user: &ResourceId) -> UseStateHandle<Vec<CoreProject>> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let user_projects = use_state(|| filter_user_projects(&user, projects_state.projects.clone()));

    {
        let user = user.clone();
        let user_projects = user_projects.clone();

        use_effect_with_deps(
            move |projects_state| {
                let projects = filter_user_projects(&user, projects_state.projects.clone());
                user_projects.set(projects);
            },
            projects_state,
        );
    }

    user_projects
}

fn filter_user_projects(user: &ResourceId, projects: ProjectMap) -> Vec<CoreProject> {
    projects
        .into_values()
        .filter(|prj| match &prj.creator {
            Creator::User(Some(UserId::Id(creator))) => creator == user,
            _ => false,
        })
        .collect::<Vec<CoreProject>>()
}

#[cfg(test)]
#[path = "./user_projects_test.rs"]
mod user_projects_test;
