//! Runner settings.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// *********************
// *** Runner Settings ***
// *********************

/// Represents Thot runner settings.
///
/// # Default
/// RunnerSettings::default is derived so does not automatically obtain a file lock.
/// This is done intentionally as it may not reflect the current state of the persisted settings.
/// To obtain the file lock use the `RunnerSettings#acquire_lock` method.
///
/// # Fields
/// + **python_path:** Option for the python binary path.
/// + **r_path:** Option for the r binary path.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RunnerSettings {
    pub python_path: Option<String>,
    pub r_path: Option<String>,
}

impl RunnerSettings {
    pub fn load() -> Result<Self> {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        serde_json::from_reader(fh)
    }

    pub fn save(&self) -> Result {
        let fh = fs::OpenOptions::new().write(true).open(Self::path())?;
        serde_json::to_writer_pretty(fh, &self)
    }
}

impl SystemResource<RunnerSettings> for RunnerSettings {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get config directory");
        settings_dir.join("runner_settings.json")
    }
}
