//! Projects collection.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{Loader, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::types::ResourceMap;

/// Map from a [`Project`]'s id to its path.
pub type ProjectMap = ResourceMap<PathBuf>;

// ****************
// *** Projects ***
// ****************

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Projects {
    file_lock: FlockLock<File>,
    projects: ProjectMap,
}

impl Deref for Projects {
    type Target = ProjectMap;

    fn deref(&self) -> &Self::Target {
        &self.projects
    }
}

impl DerefMut for Projects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.projects
    }
}

impl Settings<ProjectMap> for Projects {
    fn settings(&self) -> &ProjectMap {
        &self.projects
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

impl SystemSettings<ProjectMap> for Projects {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("projects.json")
    }
}

impl From<Loader<ProjectMap>> for Projects {
    fn from(loader: Loader<ProjectMap>) -> Projects {
        Projects {
            file_lock: loader.file_lock(),
            projects: loader.data(),
        }
    }
}

#[cfg(test)]
#[path = "./projects_test.rs"]
mod projects_test;
