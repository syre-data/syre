//! Gets a `Project`'s `Script`s.
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::analysis::get_project_analyses;
use syre_core::types::ResourceId;
use syre_ui::types::Message;
use yew::prelude::*;
use yew::suspense::{use_future_with, SuspensionResult, UseFutureHandle};

/// Loads the `Project`'s analyses into the `ProjectsState` reducer.
#[hook]
pub fn use_load_project_analyses(project: &ResourceId) -> SuspensionResult<UseFutureHandle<bool>> {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    use_future_with(project.clone(), {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.dispatcher();

        move |project| async move {
            let analyses = match get_project_analyses((*project).clone()).await {
                Ok(analyses) => analyses,
                Err(err) => {
                    tracing::debug!(err);
                    let mut msg = Message::error("Could not load project analyses.");
                    msg.set_details(format!("{err:?}"));
                    app_state.dispatch(AppStateAction::AddMessage(msg));
                    return false;
                }
            };

            projects_state.dispatch(ProjectsStateAction::InsertProjectAnalyses {
                project: (*project).clone(),
                analyses,
            });

            return true;
        }
    })
}
