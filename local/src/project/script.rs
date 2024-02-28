//! High level functionality for handling `Scripts`.
use super::container;
use crate::error::{ContainerError, Result};
use crate::loader::container::Loader as ContainerLoader;
use crate::system::collections::Scripts as SystemScripts;
use std::path::Path;
use syre_core::error::{Error as CoreError, Project as CoreProjectError};
use syre_core::project::AnalysisAssociation;
use syre_core::types::ResourceId;

/// Add an associaiton with the given script to the given container.
/// Returns the resource id of the script.
///
/// # Arguments
/// + `container`: Path of the `Container` to associate the script with.
///     Must be a in a [`Project`](super::resources::project::Project).
///
/// # Errors
/// + If `container` is not in a `Project`.
pub fn add_association(script: &ResourceId, container: &Path) -> Result {
    // check script and container are valid
    if !container::path_is_container(container) {
        return Err(ContainerError::PathNotAContainer(container.to_path_buf()).into());
    }

    // get script
    let scripts = SystemScripts::load()?;
    let Some(script) = scripts.get(script) else {
        return Err(CoreError::Project(CoreProjectError::NotRegistered(
            Some(script.clone()),
            None,
        ))
        .into());
    };

    // add association
    let mut container = ContainerLoader::load(container)?;
    container.add_script_association(AnalysisAssociation::new(script.rid.clone()))?;
    Ok(container.save()?)
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
