//! Script collection for the system.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::project::Script as CoreScript;
use thot_core::types::{ResourceId, ResourcePath};

pub type ScriptMap = HashMap<ResourceId, CoreScript>;

#[derive(Debug)]
pub struct Scripts(ScriptMap);

impl Scripts {
    pub fn load() -> Result<Self> {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        serde_json::from_reader(fh)
    }

    pub fn save(&self) -> Result {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        serde_json::to_writer_pretty(fh, &self.0)
    }

    /// Returns whether a script with the given path is registered.
    pub fn contains_path(&self, path: &ResourcePath) -> bool {
        self.by_path(path).len() > 0
    }

    /// Gets a script by its path if it is registered.
    pub fn by_path<'a>(&'a self, path: &ResourcePath) -> HashMap<&'a ResourceId, &'a CoreScript> {
        self.iter()
            .filter(|(_rid, script)| &script.path == path)
            .collect()
    }
}

impl Deref for Scripts {
    type Target = ScriptMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SystemResource<ScriptMap> for Scripts {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("scripts.json")
    }
}

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
