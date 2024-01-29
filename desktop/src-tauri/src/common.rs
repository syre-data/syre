//! Common functionality.
use crate::identifier::Identifier;
use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;
use syre_core::identifier::Identifier as CoreIdentifier;
use syre_core::types::ResourceId;
use syre_core::Result;
use syre_local::system::common;

/// Returns directories for the user's Syre.
pub fn system_dirs() -> Result<ProjectDirs> {
    let dirs_opt = ProjectDirs::from(
        &CoreIdentifier::qualifier(),
        &CoreIdentifier::organization(),
        &Identifier::application(),
    );

    match dirs_opt {
        Some(dirs) => Ok(dirs),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "system settings directory not found",
        )
        .into()),
    }
}

/// Returns the path to the user's config directory for Syre.
pub fn config_dir_path() -> Result<PathBuf> {
    let dirs = system_dirs()?;
    let path = dirs.config_dir();
    Ok(path.to_path_buf())
}

/// Path to user config directory.
pub fn users_config_dir() -> Result<PathBuf> {
    let mut path = common::config_dir_path()?;
    path.push("users");
    Ok(path)
}

/// Path to a user's config directory.
pub fn user_config_dir(user: &ResourceId) -> Result<PathBuf> {
    let mut path = users_config_dir()?;
    path.push(user.to_string());
    Ok(path)
}
