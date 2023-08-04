//! Script collection for the system.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::locked::system_settings::{Components, Loader, SystemSettings};
use settings_manager::LockedSettings;
use std::collections::HashMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::project::Script as CoreScript;
use thot_core::types::{ResourceId, ResourcePath};

pub type ScriptMap = HashMap<ResourceId, CoreScript>;

// ****************
// *** Scripts ***
// ****************

#[derive(Derivative, LockedSettings)]
#[derivative(Debug)]
pub struct Scripts {
    #[locked_settings(file_lock = "ScriptMap")]
    file_lock: FlockLock<File>,

    #[locked_settings(priority = "User")]
    scripts: ScriptMap,
}

impl Scripts {
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
        &self.scripts
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scripts
    }
}

impl SystemSettings<ScriptMap> for Scripts {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("scripts.json")
    }
}

impl From<Loader<ScriptMap>> for Scripts {
    fn from(loader: Loader<ScriptMap>) -> Self {
        let loader: Components<ScriptMap> = loader.into();
        Self {
            file_lock: loader.file_lock,
            scripts: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
