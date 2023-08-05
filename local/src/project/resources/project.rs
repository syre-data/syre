//! Project and project settings.
use super::super::PROJECT_FORMAT_VERSION;
use crate::common::{project_file, project_settings_file};
use crate::file_resource::LocalResource;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;
use thot_core::types::{ResourceMap, UserPermissions};

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
pub struct Project {
    base_path: PathBuf,
    project: CoreProject,
    settings: ProjectSettings,
}

impl Project {
    pub fn load_from(path: impl Into<PathBuf>) -> Result<Self> {
        todo!();
    }

    /// Save all data.
    pub fn save(&mut self) -> Result {
        todo!();
        <Project as LocalResource<CoreProject>>::path(self);
        <Project as LocalResource<ProjectSettings>>::path(self);
        Ok(())
    }

    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ProjectSettings {
        &mut self.settings
    }

    pub fn base_path(&self) -> &Path {
        self.base_path.as_path()
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

impl LocalResource<CoreProject> for Project {
    fn rel_path() -> PathBuf {
        project_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<ProjectSettings> for Project {
    fn rel_path() -> PathBuf {
        project_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// ************************
// *** Project Settings ***
// ************************

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectSettings {
    /// Format standard for the Project.
    pub local_format_version: String,
    pub permissions: ResourceMap<UserPermissions>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            local_format_version: PROJECT_FORMAT_VERSION.to_string(),
            permissions: ResourceMap::default(),
        }
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
