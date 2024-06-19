use crate::system::common::config_dir_path;
use crate::{error::IoSerde, file_resource::SystemResource};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::{
    fs,
    io::{self, BufReader},
    path::PathBuf,
};
use syre_core::types::ResourceId;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub active_user: Option<ResourceId>,
    pub active_project: Option<ResourceId>,
}

/// User settings.
#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct UserSettings {
    #[serde(skip)]
    path: PathBuf,
    inner: Settings,
}

impl UserSettings {
    pub fn load() -> Result<Self, IoSerde> {
        let path = Self::default_path()?;
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let inner = serde_json::from_reader(reader)?;
        Ok(Self { path, inner })
    }

    pub fn load_or_default() -> Result<Self, IoSerde> {
        let path = Self::default_path()?;
        match fs::File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let inner = serde_json::from_reader(reader)?;
                Ok(Self { path, inner })
            }

            Err(_) => Ok(Self {
                path,
                inner: Settings::default(),
            }),
        }
    }

    pub fn save(&self) -> Result<(), IoSerde> {
        fs::write(self.path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl SystemResource<UserSettings> for UserSettings {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the path to the system settings file.
    fn default_path() -> Result<PathBuf, io::Error> {
        Ok(config_dir_path()?.join("settings.json"))
    }
}

impl Deref for UserSettings {
    type Target = Settings;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for UserSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
