//! Projects collection.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::system_settings::{Components, Loader, SystemSettings};
use settings_manager::Settings;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::types::ResourceMap;

/// Map from a [`Project`]'s id to its path.
pub type ProjectMap = ResourceMap<PathBuf>;

// ****************
// *** Projects ***
// ****************

#[derive(Derivative, Settings)]
#[derivative(Debug)]
pub struct Projects {
    #[settings(file_lock = "ProjectMap")]
    file_lock: FlockLock<File>,

    #[settings(priority = "User")]
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

impl SystemSettings<ProjectMap> for Projects {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("projects.json")
    }
}

impl From<Loader<ProjectMap>> for Projects {
    fn from(loader: Loader<ProjectMap>) -> Projects {
        let loader: Components<ProjectMap> = loader.into();
        Projects {
            file_lock: loader.file_lock,
            projects: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./projects_test.rs"]
mod projects_test;
