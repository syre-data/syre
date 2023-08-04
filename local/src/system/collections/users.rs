//! User collection.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::locked::system_settings::{Components, Loader, SystemSettings};
use settings_manager::locked::Settings;
use settings_manager::LockedSettings;
use std::collections::HashMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::User;
use thot_core::types::ResourceId;

pub type UserMap = HashMap<ResourceId, User>;

#[derive(Derivative, LockedSettings)]
#[derivative(Debug)]
pub struct Users {
    #[locked_settings(file_lock = "UserMap")]
    file_lock: FlockLock<File>,

    #[locked_settings(priority = "User")]
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

impl SystemSettings<UserMap> for Users {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("users.json")
    }
}

impl From<Loader<UserMap>> for Users {
    fn from(loader: Loader<UserMap>) -> Self {
        let loader: Components<UserMap> = loader.into();
        Self {
            file_lock: loader.file_lock,
            users: loader.data,
        }
    }
}
