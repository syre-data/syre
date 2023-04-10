//! Project and project settings.
use crate::common::{project_file, project_settings_file};
use crate::Result;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::Result as SettingsResult;
use settings_manager::local_settings::{Components, Loader as LocalLoader, LocalSettings};
use settings_manager::Settings;
use std::borrow::Cow;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;
use thot_core::types::UserPermissions;

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
#[derive(Settings, Debug)]
pub struct Project {
    #[settings(file_lock = "CoreProject")]
    project_file_lock: FlockLock<File>,

    #[settings(file_lock = "ProjectSettings")]
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,

    #[settings(priority = "Local")]
    project: CoreProject,

    #[settings(priority = "Local")]
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

impl LocalSettings<CoreProject> for Project {
    fn rel_path() -> PathBuf {
        project_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// --- Project Settings ---

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

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ProjectSettings {
    permissions: Vec<UserPermissions>,
}

/// Settings for a Thot project.
#[derive(Settings)]
pub struct LocalProjectSettings {
    #[settings(file_lock = "ProjectSettings")]
    file_lock: FlockLock<File>,
    base_path: PathBuf,

    #[settings(priority = "Local")]
    settings: ProjectSettings,
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

        let project_loader: Components<CoreProject> = project_loader.into();
        let settings_loader: Components<ProjectSettings> = settings_loader.into();

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
