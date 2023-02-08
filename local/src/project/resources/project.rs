//! Project and project settings.
use crate::common::{project_file_of, project_settings_file_of};
use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::{
    Error as SettingsError, Result as SettingsResult, SettingsError as LocalSettingsError,
};
use settings_manager::local_settings::{LocalSettings, LockSettingsFile};
use settings_manager::settings::Settings;
use settings_manager::system_settings::SystemSettings;
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;
use thot_core::types::{Creator, UserId, UserPermissions};

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    pub project: CoreProject,
}

impl Project {
    /// Creates a new Project.
    pub fn new(name: &str) -> Result<Self> {
        // get active user
        let settings = UserSettings::load()?;
        let creator = match settings.active_user {
            None => None,
            Some(uid) => Some(UserId::Id(uid.into())),
        };
        let creator = Creator::User(creator);

        let mut project = CoreProject::new(name);
        project.creator = creator;

        Ok(Project {
            _file_lock: None,
            _base_path: None,
            project,
        })
    }
}

impl Clone for Project {
    /// Clones the `Project`'s `project` and `base_path`.
    /// `base_path` of the cloned `Project` is set to `None`
    ///     if an `Err` is retuned when calculating it from the original `Project`.
    /// Sets the cloned object's `file_lock` to `None`.
    fn clone(&self) -> Self {
        Self {
            _file_lock: None,
            _base_path: self.base_path().ok(),
            project: self.project.clone(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        // attempt to create new project
        let new_prj = Project::new("");
        if let Ok(prj) = new_prj {
            return prj;
        }

        // if fail, create manually
        Project {
            _file_lock: None,
            _base_path: None,
            project: CoreProject::default(),
        }
    }
}

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

impl Into<CoreProject> for Project {
    fn into(self: Self) -> CoreProject {
        self.project
    }
}

impl Settings for Project {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings for Project {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(project_file_of(Path::new("")))
    }

    fn base_path(&self) -> SettingsResult<PathBuf> {
        self._base_path
            .clone()
            .ok_or(SettingsError::SettingsError(LocalSettingsError::PathNotSet))
    }

    fn set_base_path(&mut self, path: PathBuf) -> SettingsResult {
        self._base_path = Some(path);
        Ok(())
    }
}

impl LockSettingsFile for Project {}

// ************************
// *** Project Settings ***
// ************************

/// Settings for a Thot project.
///
/// # Fields
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ProjectSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    permissions: Vec<UserPermissions>,
}

impl ProjectSettings {
    /// Creates a new project settings.
    pub fn new() -> Self {
        ProjectSettings {
            _file_lock: None,
            _base_path: None,

            permissions: Vec::new(),
        }
    }
}

impl Settings for ProjectSettings {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings for ProjectSettings {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(project_settings_file_of(Path::new("")))
    }

    fn base_path(&self) -> SettingsResult<PathBuf> {
        match self._base_path.clone() {
            Some(path) => Ok(path),
            None => Err(SettingsError::SettingsError(LocalSettingsError::PathNotSet)),
        }
    }

    fn set_base_path(&mut self, path: PathBuf) -> SettingsResult {
        self._base_path = Some(path);
        Ok(())
    }
}

impl LockSettingsFile for ProjectSettings {}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
