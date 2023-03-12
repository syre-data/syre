//! Local [`Script`].
use crate::common::scripts_file_of;
use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::{
    Error as SettingsError, Result as SettingsResult, SettingsError as LocalSettingsError,
};
use settings_manager::local_settings::LockSettingsFile;
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::{LocalSettings, Settings, SystemSettings};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Script as CoreScript, Scripts as CoreScripts};
use thot_core::types::{ResourceMap, ResourcePath};

// **************
// *** Script ***
// **************

pub struct Script;
impl Script {
    /// Creates a new [`Script`] with the `creator` field matching the current active creator.
    pub fn new(path: ResourcePath) -> Result<CoreScript> {
        let settings = UserSettings::load()?;
        let creator = settings.active_user.map(|c| c.into());

        let mut script = CoreScript::new(path)?;
        script.creator = creator;
        Ok(script)
    }
}

// ***************
// *** Scripts ***
// ***************

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Scripts {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    pub scripts: CoreScripts,
}

impl Scripts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a script.
    ///
    /// # Errors
    /// + [`ResourceError::AlreadyExists`] if a script with the same path is
    /// already present.
    pub fn insert_script(&mut self, script: CoreScript) -> Result {
        if self.scripts.contains_path(&script.path) {
            return Err(CoreError::ResourceError(ResourceError::AlreadyExists(
                "`Script` with same path is already present",
            ))
            .into());
        }

        self.scripts.insert(script.rid.clone(), script);
        Ok(())
    }
}

impl Deref for Scripts {
    type Target = ResourceMap<CoreScript>;

    fn deref(&self) -> &Self::Target {
        &*self.scripts
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.scripts
    }
}

impl Into<CoreScripts> for Scripts {
    fn into(self) -> CoreScripts {
        self.scripts
    }
}

impl Settings for Scripts {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
    }

    fn file(&self) -> Option<&File> {
        match self._file_lock.as_ref() {
            None => None,
            Some(lock) => Some(&*lock),
        }
    }

    fn file_mut(&mut self) -> Option<&mut File> {
        match self._file_lock.as_mut() {
            None => None,
            Some(lock) => Some(lock),
        }
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings for Scripts {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(scripts_file_of(Path::new("")))
    }

    fn base_path(&self) -> SettingsResult<PathBuf> {
        self._base_path
            .clone()
            .ok_or(SettingsError::SettingsError(LocalSettingsError::PathNotSet))
    }

    fn set_base_path(&mut self, path: PathBuf) -> SettingsResult {
        self._base_path = Some(path);
        Ok(())
    }
}

impl LockSettingsFile for Scripts {}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
