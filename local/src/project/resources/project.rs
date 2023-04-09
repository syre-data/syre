//! Project and project settings.
use crate::common::{project_file, project_settings_file};
use crate::Result;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::Result as SettingsResult;
use settings_manager::local_settings::{Loader as LocalLoader, LocalSettings};
use settings_manager::settings::Settings;
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;
use thot_core::types::UserPermissions;

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
#[derive(Debug)]
pub struct Project {
    project_file_lock: FlockLock<File>,
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,
    project: CoreProject,
    settings: ProjectSettings,
}

impl Project {
    pub fn base_path(&self) -> &Path {
        self.base_path.as_path()
    }

    /// Save all data.
    pub fn save(&mut self) -> Result {
        <Project as Settings<CoreProject>>::save(self)?;
        <Project as Settings<ProjectSettings>>::save(self)?;
        Ok(())
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

// --- Core Project ---
impl Settings<CoreProject> for Project {
    fn settings(&self) -> &CoreProject {
        &self.project
    }

    fn file(&self) -> &File {
        &*self.project_file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.project_file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.project_file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<CoreProject> for Project {
    fn rel_path() -> PathBuf {
        project_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// --- Project Settings ---
impl Settings<ProjectSettings> for Project {
    fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    fn file(&self) -> &File {
        &*self.settings_file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.settings_file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.settings_file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<ProjectSettings> for Project {
    fn rel_path() -> PathBuf {
        project_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl From<Loader> for Project {
    fn from(loader: Loader) -> Self {
        Self {
            project_file_lock: loader.project_file_lock,
            settings_file_lock: loader.settings_file_lock,

            base_path: loader.base_path,
            project: loader.project,
            settings: loader.settings,
        }
    }
}

// ************************
// *** Project Settings ***
// ************************

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ProjectSettings {
    permissions: Vec<UserPermissions>,
}

/// Settings for a Thot project.
pub struct LocalProjectSettings {
    file_lock: FlockLock<File>,
    base_path: PathBuf,
    settings: ProjectSettings,
}

impl Settings<ProjectSettings> for LocalProjectSettings {
    fn settings(&self) -> &ProjectSettings {
        &self.settings
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
        SettingsPriority::Local
    }
}

impl LocalSettings<ProjectSettings> for LocalProjectSettings {
    fn rel_path() -> PathBuf {
        project_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// **************
// *** Loader ***
// **************

pub struct Loader {
    project_file_lock: FlockLock<File>,
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,
    project: CoreProject,
    settings: ProjectSettings,
}

impl Loader {
    pub fn load_or_create(path: PathBuf) -> SettingsResult<Self> {
        let project_loader = LocalLoader::load_or_create::<Project>(path.clone())?;
        let settings_loader = LocalLoader::load_or_create::<LocalProjectSettings>(path)?;

        Ok(Self {
            project_file_lock: project_loader.file_lock(),
            settings_file_lock: settings_loader.file_lock(),

            base_path: project_loader.base_path(),
            project: project_loader.data(),
            settings: settings_loader.data(),
        })
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
