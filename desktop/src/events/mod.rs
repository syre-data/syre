//! [`Event`](thot_local_database::events) handlers.
use crate::Result;
use thot_local_database::events::Update;

mod project;

/// Delegate event to handlers.
#[tracing::instrument]
pub fn handle_event(event: Update) -> Result {
    match event {
        Update::Project { project, update } => project::handle_event_project(project, update),
    }
}
