use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use thot_core::types::ResourceId;

/// User settings.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserSettings {
    pub active_user: Option<ResourceId>,
    pub active_project: Option<ResourceId>,
}

impl UserSettings {
    pub fn load() -> Result<Self> {
        let file = fs::File::open(Self::path())?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save(&self) -> Result {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        Ok(serde_json::to_writer_pretty(fh, &self)?)
    }
}

impl SystemResource<UserSettings> for UserSettings {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path()
            .expect("could not get config dir")
            .to_path_buf();

        settings_dir.join("settings.json")
    }
}
