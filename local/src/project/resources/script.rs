//! Local [`Script`].
use crate::common::scripts_file;
use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use cluFlock::FlockLock;
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::{
    local_settings::Loader, system_settings::Loader as SystemLoader, LocalSettings, Settings,
};
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
        let settings: UserSettings = SystemLoader::load_or_create::<UserSettings>()?.into();
        let creator = settings.active_user.clone().map(|c| c.into());

        let mut script = CoreScript::new(path)?;
        script.creator = creator;
        Ok(script)
    }
}

// ***************
// *** Scripts ***
// ***************

#[derive(Debug)]
pub struct Scripts {
    file_lock: FlockLock<File>,
    base_path: PathBuf,

    pub scripts: CoreScripts,
}

impl Scripts {
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

impl Settings<CoreScripts> for Scripts {
    fn settings(&self) -> &CoreScripts {
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
        SettingsPriority::Local
    }
}

impl LocalSettings<CoreScripts> for Scripts {
    fn rel_path() -> PathBuf {
        scripts_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl From<Loader<CoreScripts>> for Scripts {
    fn from(loader: Loader<CoreScripts>) -> Self {
        Self {
            file_lock: loader.file_lock(),
            base_path: loader.base_path(),
            scripts: loader.data(),
        }
    }
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
