//! Common implmentation for system functionality.
use directories::ProjectDirs;
use settings_manager::{Error as SettingsError, Result as SettingsResult};
use std::io;
use std::path::PathBuf;
use thot_core::identifier::Identifier;

/// Returns directories for the user's Thot.
pub fn system_dirs() -> SettingsResult<ProjectDirs> {
    let dirs_opt = ProjectDirs::from(
        &Identifier::qualifier(),
        &Identifier::organization(),
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

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
