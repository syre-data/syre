//! Project and project settings.
use crate::common::{project_file, project_settings_file};
use crate::file_resource::LocalResource;
use crate::types::ProjectSettings;
use crate::Result;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::Project as CoreProject;

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

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
