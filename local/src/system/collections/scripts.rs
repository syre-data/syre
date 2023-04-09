//! Script collection for the system.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{Loader, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Scripts {
    file_lock: FlockLock<File>,
    pub scripts: ScriptMap,
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

impl Settings<ScriptMap> for Scripts {
    fn settings(&self) -> &ScriptMap {
        &self.scripts
    }

    fn file(&self) -> &File {
        &self.file_lock
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

impl SystemSettings<ScriptMap> for Scripts {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("scripts.json")
    }
}

impl From<Loader<ScriptMap>> for Scripts {
    fn from(loader: Loader<ScriptMap>) -> Self {
        Self {
            file_lock: loader.file_lock(),
            scripts: loader.data(),
        }
    }
}

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
