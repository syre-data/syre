use cluFlock::FlockLock;
use derivative::{self, Derivative};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{LockSettingsFile, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::{Error as SettingsError, Result as SettingsResult};
use std::default::Default;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use thot_core::identifier::Identifier;
use thot_core::types::ResourceId;

// *********************
// *** User Settings ***
// *********************

/// Represents Thot user settings.
///
/// # Default
/// UserSettings::default is derived so does not automatically obtain a file lock.
/// This is done intentionally as it may not reflect the current state of the persisted settings.
/// To obtain the file lock use the `UserSettings#acquire_lock` method.
///
/// # Fields
/// + **active_user:** Option of the active User id.
/// + **active_project:** Option of the active Project id.
#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct UserSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    pub active_user: Option<ResourceId>,
    pub active_project: Option<ResourceId>,
}

impl UserSettings {
    /// Returns directories for the user's Thot.
    pub fn dirs() -> SettingsResult<ProjectDirs> {
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

    /// Returns the path to the users config directory for Thot.
    pub fn dir_path() -> SettingsResult<PathBuf> {
        let dirs = Self::dirs()?;
        let path = dirs.config_dir();
        Ok(path.to_path_buf())
    }
}

impl Settings for UserSettings {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl SystemSettings for UserSettings {
    /// Returns the path to the system settings file.
    fn path() -> SettingsResult<PathBuf> {
        let settings_dir = Self::dir_path()?;
        Ok(settings_dir.join("settings.json"))
    }
}

impl LockSettingsFile for UserSettings {}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
