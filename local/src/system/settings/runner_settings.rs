/// Runner settings.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::locked::settings::Settings;
use settings_manager::locked::system_settings::{Components, Loader, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::Result as SettingsResult;
use std::borrow::Cow;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

// *********************
// *** Runner Settings ***
// *********************

/// Represents Thot runner settings.
///
/// # Default
/// RunnerSettings::default is derived so does not automatically obtain a file lock.
/// This is done intentionally as it may not reflect the current state of the persisted settings.
/// To obtain the file lock use the `RunnerSettings#acquire_lock` method.
///
/// # Fields
/// + **python_path:** Option for the python binary path.
/// + **r_path:** Option for the r binary path.
pub struct RunnerSettings {
    file_lock: FlockLock<File>,
    settings: LocalRunnerSettings,
}

//@todo[m]: Code duplication from user_settings.rs
impl RunnerSettings {
    /// Returns the path to the users config directory for Thot.
    pub fn dir_path() -> SettingsResult<PathBuf> {
        let path = config_dir_path()?;
        Ok(path.to_path_buf())
    }
}

impl Deref for RunnerSettings {
    type Target = LocalRunnerSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl DerefMut for RunnerSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.settings
    }
}

/// Runner settings.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LocalRunnerSettings {
    pub python_path: Option<String>,
    pub r_path: Option<String>,
}

impl Settings<LocalRunnerSettings> for RunnerSettings {
    fn settings(&self) -> Cow<LocalRunnerSettings> {
        Cow::Owned(LocalRunnerSettings {
            python_path: self.python_path.clone(),
            r_path: self.r_path.clone(),
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

impl SystemSettings<LocalRunnerSettings> for RunnerSettings {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = Self::dir_path().expect("could not get settings directory");
        settings_dir.join("runner_settings.json")
    }
}

impl From<Loader<LocalRunnerSettings>> for RunnerSettings {
    fn from(loader: Loader<LocalRunnerSettings>) -> Self {
        let loader: Components<LocalRunnerSettings> = loader.into();
        Self {
            file_lock: loader.file_lock,
            settings: loader.data,
        }
    }
}

//@todo[l]: Add tests, not urgent since code logic similar to user_settings.rs
// #[cfg(test)]
// #[path = "./runner_settings_test.rs"]
// mod runner_settings_test;
