/// User settings.
use cluFlock::FlockLock;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{Components, Loader, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::{Error as SettingsError, Result as SettingsResult};
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::ops::{Deref, DerefMut};
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
pub struct UserSettings {
    file_lock: FlockLock<File>,
    settings: LocalUserSettings,
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

impl Deref for UserSettings {
    type Target = LocalUserSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl DerefMut for UserSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.settings
    }
}

/// User settings.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LocalUserSettings {
    pub active_user: Option<ResourceId>,
    pub active_project: Option<ResourceId>,
}

impl Settings<LocalUserSettings> for UserSettings {
    fn settings(&self) -> Cow<LocalUserSettings> {
        Cow::Owned(LocalUserSettings {
            active_user: self.active_user.clone(),
            active_project: self.active_project.clone(),
        })
    }

    fn file(&self) -> &File {
        &*self.file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl SystemSettings<LocalUserSettings> for UserSettings {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = Self::dir_path().expect("could not get settings directory");
        settings_dir.join("settings.json")
    }
}

impl From<Loader<LocalUserSettings>> for UserSettings {
    fn from(loader: Loader<LocalUserSettings>) -> Self {
        let loader: Components<LocalUserSettings> = loader.into();
        Self {
            file_lock: loader.file_lock,
            settings: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
