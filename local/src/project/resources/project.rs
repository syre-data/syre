//! Project and project settings.
use super::super::types::ProjectSettings as PrjSettings;
use crate::common::{project_file, project_settings_file};
use crate::Result;
use cluFlock::FlockLock;
use settings_manager::error::Result as SettingsResult;
use settings_manager::local_settings::{Components, Loader as LocalLoader, LocalSettings};
use settings_manager::Settings;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
#[derive(Settings, Debug)]
pub struct Project {
    #[settings(file_lock = "CoreProject")]
    project_file_lock: FlockLock<File>,

    #[settings(file_lock = "PrjSettings")]
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,

    #[settings(priority = "Local")]
    project: CoreProject,

    #[settings(priority = "Local")]
    settings: PrjSettings,
}

impl Project {
    pub fn settings(&self) -> &PrjSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut PrjSettings {
        &mut self.settings
    }

    pub fn base_path(&self) -> &Path {
        self.base_path.as_path()
    }

    /// Save all data.
    pub fn save(&mut self) -> Result {
        <Project as Settings<CoreProject>>::save(self)?;
        <Project as Settings<PrjSettings>>::save(self)?;
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
impl LocalSettings<CoreProject> for Project {
    fn rel_path() -> PathBuf {
        project_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// --- Project Settings ---

impl LocalSettings<PrjSettings> for Project {
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

/// Settings for a Thot project.
#[derive(Settings)]
pub struct ProjectSettings {
    #[settings(file_lock = "PrjSettings")]
    file_lock: FlockLock<File>,
    base_path: PathBuf,

    #[settings(priority = "Local")]
    settings: PrjSettings,
}

impl LocalSettings<PrjSettings> for ProjectSettings {
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
    settings: PrjSettings,
}

impl Loader {
    pub fn load_or_create(path: impl Into<PathBuf>) -> SettingsResult<Self> {
        let path = path.into();
        let project_loader = LocalLoader::load_or_create::<Project>(path.clone())?;
        let settings_loader = LocalLoader::load_or_create::<ProjectSettings>(path)?;

        let project_loader: Components<CoreProject> = project_loader.into();
        let settings_loader: Components<PrjSettings> = settings_loader.into();

        Ok(Self {
            project_file_lock: project_loader.file_lock,
            settings_file_lock: settings_loader.file_lock,

            base_path: project_loader.base_path,
            project: project_loader.data,
            settings: settings_loader.data,
        })
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
