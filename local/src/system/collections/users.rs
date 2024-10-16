//! User collection.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::User;
use thot_core::types::ResourceId;

pub type UserMap = HashMap<ResourceId, User>;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct Users(UserMap);

impl Users {
    pub fn load() -> Result<Self> {
        let file = fs::File::open(Self::path())?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn load_or_default() -> Result<Self> {
        match fs::File::open(Self::path()) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Ok(serde_json::from_reader(reader)?)
            }

            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save(&self) -> Result {
        fs::write(Self::path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl Deref for Users {
    type Target = UserMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Users {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SystemResource<UserMap> for Users {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("users.json")
    }
}
