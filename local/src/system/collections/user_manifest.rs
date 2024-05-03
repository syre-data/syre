//! User collection.
use crate::error::IoSerde;
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufReader};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use syre_core::system::User;
use syre_core::types::ResourceId;

pub type UserMap = HashMap<ResourceId, User>;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct UserManifest {
    inner: UserMap,

    /// Path to the user manifest file.
    #[serde(skip)]
    path: PathBuf,
}

impl UserManifest {
    const FILE_NAME: &'static str = "users.json";

    pub fn load() -> Result<Self, IoSerde> {
        let path = Self::path()?;
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            inner: serde_json::from_reader(reader)?,
            path,
        })
    }

    pub fn load_or_default() -> Result<Self, IoSerde> {
        let path = Self::path()?;
        match fs::File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Ok(Self {
                    inner: serde_json::from_reader(reader)?,
                    path,
                })
            }

            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save(&self) -> Result<(), IoSerde> {
        fs::write(&Self::path()?, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl UserManifest {
    /// Load the manifest from the given path.
    pub fn load_from(path: impl Into<PathBuf>) -> Result<Self, IoSerde> {
        let path = path.into();
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            inner: serde_json::from_reader(reader)?,
            path,
        })
    }

    /// Load the manifest from the given path or create the default if the file does not exist.
    pub fn load_from_or_default(path: impl Into<PathBuf>) -> Result<Self, IoSerde> {
        let path = path.into();
        match fs::File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Ok(Self {
                    inner: serde_json::from_reader(reader)?,
                    path,
                })
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    /// Saves the manifest to the path is was loaded from.
    pub fn save_to(&self) -> Result<(), IoSerde> {
        fs::write(&self.path, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl Deref for UserManifest {
    type Target = UserMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for UserManifest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SystemResource<UserMap> for UserManifest {
    /// Returns the path to the system settings file.
    fn path() -> Result<PathBuf, io::Error> {
        Ok(config_dir_path()?.join(Self::FILE_NAME))
    }
}
