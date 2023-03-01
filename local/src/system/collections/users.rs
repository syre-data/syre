//! User collection.
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
use thot_core::system::User;
use thot_core::types::{resource_map::values_only, ResourceId};

pub type UserMap = HashMap<ResourceId, User>;

#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct Users {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(with = "values_only")]
    pub users: UserMap,
}

impl Settings for Users {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
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
        SettingsPriority::User
    }
}

impl SystemSettings for Users {
    /// Returns the path to the system settings file.
    fn path() -> SettingsResult<PathBuf> {
        let settings_dir = config_dir_path()?;
        Ok(settings_dir.join("users.json"))
    }
}

impl Deref for Users {
    type Target = UserMap;

    fn deref(&self) -> &Self::Target {
        &self.users
    }
}

impl DerefMut for Users {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.users
    }
}

impl LockSettingsFile for Users {}

#[cfg(test)]
#[path = "./users_test.rs"]
mod users_test;
