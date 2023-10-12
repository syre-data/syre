//! (events::Project)[thot_local_database::events::Project] handlers.
use crate::Result;
use thot_core::types::ResourceId;
use thot_local_database::events::{Container, Project};

/// Delegate event to handlers.
#[tracing::instrument]
pub fn handle_event_project(project: ResourceId, event: Project) -> Result {
    match event {
        Project::Container(container) => handle_event_container(container),
    }
}

/// Delegate event to handlers.
#[tracing::instrument]
fn handle_event_container(event: Container) -> Result {
    match event {
        Container::Properties {
            container,
            properties,
        } => update_container_properties(container, properties),
    }
}

/// Update a Container's properties.
#[tracing::instrument]
fn update_container_properties(
    container: ResourceId,
    properties: thot_core::project::ContainerProperties,
) -> Result {
    tracing::debug!("RUN");
    Ok(())
}
