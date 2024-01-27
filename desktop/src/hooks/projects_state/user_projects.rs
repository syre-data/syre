//! Retrieves a user's projects.
use crate::app::projects_state::{ProjectMap, SettingsMap};
use crate::app::ProjectsStateReducer;
use thot_core::project::Project;
use thot_core::types::{Creator, ResourceId, UserId};
use yew::prelude::*;

/// Retrieve a user's projects.
#[hook]
pub fn use_user_projects(user: &ResourceId) -> UseStateHandle<Vec<Project>> {
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let user_projects: UseStateHandle<Vec<Project>> = use_state(|| {
        filter_user_projects(&user, &projects_state.projects, &projects_state.settings)
            .clone()
            .into_iter()
            .map(|project| project.clone())
            .collect()
    });

    use_effect_with(projects_state, {
        let user = user.clone();
        let user_projects = user_projects.clone();

        move |projects_state| {
            let projects =
                filter_user_projects(&user, &projects_state.projects, &projects_state.settings)
                    .clone()
                    .into_iter()
                    .map(|project| project.clone())
                    .collect();

            user_projects.set(projects);
        }
    });

    user_projects
}

fn filter_user_projects<'a>(
    user: &ResourceId,
    projects: &'a ProjectMap,
    settings: &SettingsMap,
) -> Vec<&'a Project> {
    let creator = Creator::User(Some(UserId::Id(user.clone())));

    projects
        .values()
        .filter(|prj| {
            if prj.creator == creator {
                return true;
            }

            let Some(prj_settings) = settings.get(&prj.rid) else {
                return false;
            };

            prj_settings.permissions.contains_key(user)
        })
        .collect::<Vec<&Project>>()
}
