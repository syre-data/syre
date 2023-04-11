//! High level functionality for handling `Scripts`.
use super::container;
use super::resources::{
    container::LocalContainerProperties, Script as ProjectScript, Scripts as ProjectScripts,
};
use crate::error::{ContainerError, Result};
use crate::system::collections::Projects;
use crate::system::collections::Scripts as SystemScripts;
use settings_manager::{
    local_settings::Loader as LocalLoader, system_settings::Loader as SystemLoader, Settings,
};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
use thot_core::project::RunParameters;
use thot_core::types::{ResourceId, ResourcePath};

// ***************
// *** Scripts ***
// ***************

/// Initialize a file as a [`Script`](CoreScript).
pub fn init(project: ResourceId, path: PathBuf) -> Result<ResourceId> {
    let projects: Projects = SystemLoader::load_or_create::<Projects>()?.into();
    let Some(project) = projects.get(&project) else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` does not exist")).into());
    };

    let mut scripts: ProjectScripts =
        LocalLoader::load_or_create::<ProjectScripts>(project.clone())?.into();

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
    let scripts: SystemScripts = SystemLoader::load_or_create::<SystemScripts>()?.into();
    let Some(script) = scripts.get(script) else {
        return Err(CoreError::ProjectError(
            CoreProjectError::NotRegistered(Some(script.clone()), None)).into()
        )
    };

    // add association
    let mut container: LocalContainerProperties =
        LocalLoader::load_or_create::<LocalContainerProperties>(container.into())?.into();
    container
        .scripts_mut()
        .insert(script.rid.clone(), RunParameters::new());

    container.save()?;
    Ok(())
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
