//! Runner settings.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufReader},
    ops::Deref,
    path::PathBuf,
    result::Result as StdResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    pub python_path: Option<String>,
    pub r_path: Option<String>,
}

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
#[serde(transparent)]
pub struct RunnerSettings {
    #[serde(skip)]
    path: PathBuf,
    inner: Settings,
}

impl RunnerSettings {
    const FILE_NAME: &'static str = "runner_settings.json";

    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let inner = serde_json::from_reader(reader)?;
        Ok(Self { path, inner })
    }

    pub fn save(&self) -> Result {
        fs::create_dir_all(self.path().parent().unwrap())?;
        fs::write(self.path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

// TODO Should probably be a `UserResource`.
impl SystemResource<Settings> for RunnerSettings {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the path to the system settings file.
    fn default_path() -> StdResult<PathBuf, io::Error> {
        Ok(config_dir_path()?.join(Self::FILE_NAME))
    }
}

impl Deref for RunnerSettings {
    type Target = Settings;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
