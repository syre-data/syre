//! High level functionality for handling `Scripts`.
use super::container;
use super::resources::{Container, Script as ProjectScript, Scripts as ProjectScripts};
use crate::error::{ContainerError, Result};
use crate::system::collections::{Projects, Scripts as SystemScripts};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
use thot_core::project::ScriptAssociation;
use thot_core::types::{ResourceId, ResourcePath};

// ***************
// *** Scripts ***
// ***************

/// Initialize a file as a [`Script`](CoreScript).
pub fn init(project: ResourceId, path: PathBuf) -> Result<ResourceId> {
    let projects = Projects::load()?;
    let Some(project) = projects.get(&project) else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` does not exist")).into());
    };

    let mut scripts = ProjectScripts::load_from(project.clone())?;
    let path = ResourcePath::new(path)?;
    let script = ProjectScript::new(path)?;

    let rid = script.rid.clone();
    scripts.insert_script(script)?;
    scripts.save()?;

    Ok(rid)
}

// **************************
// *** Script Association ***
// **************************

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
        return Err(CoreError::ProjectError(
            CoreProjectError::NotRegistered(Some(script.clone()), None)).into()
        )
    };

    // add association
    let mut container = Container::load_from(container)?;
    container.add_script_association(ScriptAssociation::new(script.rid.clone()));
    container.save()
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
