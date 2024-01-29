//! All settings.
use crate::common;
use crate::error::{DesktopSettings as DesktopSettingsError, Result};
use std::fs;
use std::io::{self, BufReader};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use syre_core::types::ResourceId;
use syre_desktop_lib::settings::UserSettingsFile;
use syre_desktop_lib::settings::{HasUser, UserSettings as DesktopUserSettings};
use syre_local::error::IoSerde;
use syre_local::file_resource::UserResource;

pub struct UserSettings {
    rel_path: PathBuf,
    settings: DesktopUserSettings,
}

impl UserSettings {
    /// Loads the settings for the given user.
    pub fn load(user: &ResourceId) -> StdResult<Self, IoSerde> {
        let rel_path = PathBuf::from(user.to_string());
        let rel_path = rel_path.join(Self::settings_file());

        let path = Self::base_path().join(&rel_path);
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let settings = serde_json::from_reader(reader)?;

        Ok(Self {
            rel_path: rel_path.into(),
            settings,
        })
    }

    pub fn load_or_new(user: &ResourceId) -> StdResult<Self, IoSerde> {
        let rel_path = PathBuf::from(user.to_string());
        let rel_path = rel_path.join(Self::settings_file());

        let path = Self::base_path().join(&rel_path);
        match fs::File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let settings = serde_json::from_reader(reader)?;

                Ok(Self {
                    rel_path: rel_path.into(),
                    settings,
                })
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self {
                rel_path: rel_path.into(),
                settings: DesktopUserSettings::new(user.clone()),
            }),

            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self) -> StdResult<(), IoSerde> {
        fs::write(self.path(), serde_json::to_string_pretty(&self.settings)?)?;
        Ok(())
    }

    /// Updates the app state.
    pub fn update(&mut self, settings: DesktopUserSettings) -> Result {
        // verify correct user
        if settings.user() != self.settings.user() {
            return Err(
                DesktopSettingsError::InvalidUpdate("users do not match".to_string()).into(),
            );
        }

        self.settings = settings;
        Ok(())
    }
}

impl Deref for UserSettings {
    type Target = DesktopUserSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl DerefMut for UserSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.settings
    }
}

impl Into<DesktopUserSettings> for UserSettings {
    fn into(self) -> DesktopUserSettings {
        self.settings
    }
}

impl UserResource<DesktopUserSettings> for UserSettings {
    fn base_path() -> PathBuf {
        common::users_config_dir().expect("could not get config path")
    }

    fn rel_path(&self) -> &Path {
        &self.rel_path
    }
}

impl UserSettingsFile for UserSettings {
    fn settings_file() -> PathBuf {
        PathBuf::from("desktop_settings.json")
    }
}
