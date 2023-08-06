//! Projects collection.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::types::ResourceMap;

/// Map from a [`Project`]'s id to its path.
pub type ProjectMap = ResourceMap<PathBuf>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Projects(ProjectMap);

impl Projects {
    pub fn load() -> Result<Self> {
        let file = fs::File::open(Self::path())?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save(&self) -> Result {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        Ok(serde_json::to_writer_pretty(fh, &self.0)?)
    }
}

impl Deref for Projects {
    type Target = ProjectMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Projects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SystemResource<ProjectMap> for Projects {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("projects.json")
    }
}

#[cfg(test)]
#[path = "./projects_test.rs"]
mod projects_test;
