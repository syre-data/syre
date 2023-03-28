//! Project template.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::{Error as SettingsMgrError, Result as SettingsResult, SettingsError};
use settings_manager::{Priority as SettingsPriority, Settings, UserSettings};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::template::Project as CoreProject;

#[derive(Serialize, Deserialize)]
pub struct Project {
    #[serde(skip)]
    file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    rel_path: Option<PathBuf>,

    project: CoreProject,
}

impl Project {}

impl Deref for Project {
    type Target = CoreProject;

    fn deref(&self) -> &Self::Target {
        &self.project
    }
}

impl DerefMut for Project {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.project
    }
}

impl Settings for Project {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self.file_lock = Some(file_lock);
    }

    fn file(&self) -> Option<&File> {
        match self.file_lock.as_ref() {
            None => None,
            Some(lock) => Some(&*lock),
        }
    }

    fn file_mut(&mut self) -> Option<&mut File> {
        match self.file_lock.as_mut() {
            None => None,
            Some(lock) => Some(lock),
        }
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl UserSettings for Project {
    /// Returns the base path to the settings file.
    fn base_path() -> SettingsResult<PathBuf> {
        let mut path = config_dir_path()?;
        path.push("templates");

        Ok(path)
    }

    /// Returns the relative path for the settings.
    fn rel_path(&self) -> SettingsResult<PathBuf> {
        match self.rel_path.as_ref() {
            Some(path) => Ok(path.clone()),
            None => Err(SettingsMgrError::SettingsError(SettingsError::PathNotSet)),
        }
    }

    /// Sets the relative path for the settings.
    fn set_rel_path(&mut self, path: PathBuf) -> SettingsResult {
        self.rel_path = Some(path);
        Ok(())
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
