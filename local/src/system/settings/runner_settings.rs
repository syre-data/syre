//! Runner settings.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufReader},
    path::PathBuf,
    result::Result as StdResult,
};

/// Represents Syre runner settings.
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
    const FILE_NAME: &'static str = "runner_settings.json";

    pub fn load() -> Result<Self> {
        let file = fs::File::open(Self::path()?)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save(&self) -> Result {
        fs::create_dir_all(Self::path()?.parent().expect("invalid path"))?;
        fs::write(Self::path()?, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

// TODO Should probably be a `UserResource`.
impl SystemResource<RunnerSettings> for RunnerSettings {
    /// Returns the path to the system settings file.
    fn path() -> StdResult<PathBuf, io::Error> {
        Ok(config_dir_path()?.join(Self::FILE_NAME))
    }
}
