use crate::system::common::config_dir_path;
use crate::{error::IoSerde, file_resource::SystemResource};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufReader},
    path::PathBuf,
};
use syre_core::types::ResourceId;

/// User settings.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserSettings {
    pub active_user: Option<ResourceId>,
    pub active_project: Option<ResourceId>,
}

impl UserSettings {
    pub fn load() -> Result<Self, IoSerde> {
        let file = fs::File::open(Self::path()?)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn load_or_default() -> Result<Self, IoSerde> {
        match fs::File::open(Self::path()?) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Ok(serde_json::from_reader(reader)?)
            }

            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save(&self) -> Result<(), IoSerde> {
        fs::write(Self::path()?, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl SystemResource<UserSettings> for UserSettings {
    /// Returns the path to the system settings file.
    fn path() -> Result<PathBuf, io::Error> {
        Ok(config_dir_path()?.join("settings.json"))
    }
}
