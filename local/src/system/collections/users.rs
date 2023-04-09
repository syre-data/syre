//! User collection.
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
use thot_core::system::User;
use thot_core::types::ResourceId;

pub type UserMap = HashMap<ResourceId, User>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Users {
    file_lock: FlockLock<File>,
    pub users: UserMap,
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

impl Settings<UserMap> for Users {
    fn settings(&self) -> &UserMap {
        &self.users
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

impl SystemSettings<UserMap> for Users {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("users.json")
    }
}

impl From<Loader<UserMap>> for Users {
    fn from(loader: Loader<UserMap>) -> Self {
        Self {
            file_lock: loader.file_lock(),
            users: loader.data(),
        }
    }
}

#[cfg(test)]
#[path = "./users_test.rs"]
mod users_test;
