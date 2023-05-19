//! Common functionality.
use settings_manager::Result;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_local::system::common;

use crate::identifier::Identifier;
use directories::ProjectDirs;
use settings_manager::{Error as SettingsError, Result as SettingsResult};
use std::io;
use thot_core::identifier::Identifier as CoreIdentifier;

/// Returns directories for the user's Thot.
pub fn system_dirs() -> SettingsResult<ProjectDirs> {
    let dirs_opt = ProjectDirs::from(
        &CoreIdentifier::qualifier(),
        &CoreIdentifier::organization(),
        &Identifier::application(),
    );

    match dirs_opt {
        Some(dirs) => Ok(dirs),
        None => Err(SettingsError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "system settings directory not found",
        ))),
    }
}

/// Returns the path to the user's config directory for Thot.
pub fn config_dir_path() -> SettingsResult<PathBuf> {
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

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
