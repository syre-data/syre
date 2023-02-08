//! Script collection for the system.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use serde::{Deserialize, Serialize};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{LockSettingsFile, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::Result as SettingsResult;
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::project::Script as CoreScript;
use thot_core::types::{resource_map::values_only, ResourceId, ResourcePath};

pub type ScriptMap = HashMap<ResourceId, CoreScript>;

// ****************
// *** Scripts ***
// ****************

#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct Scripts {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(with = "values_only")]
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

impl Settings for Scripts {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl SystemSettings for Scripts {
    /// Returns the path to the system settings file.
    fn path() -> SettingsResult<PathBuf> {
        let settings_dir = config_dir_path()?;
        Ok(settings_dir.join("scripts.json"))
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

impl LockSettingsFile for Scripts {}

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
