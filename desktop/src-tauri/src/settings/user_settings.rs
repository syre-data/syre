//! All settings.
use crate::common;
use crate::error::{DesktopSettingsError, Result};
use std::fs;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::UserSettingsFile;
use thot_desktop_lib::settings::{HasUser, UserSettings as DesktopUserSettings};
use thot_local::file_resource::UserResource;

pub struct UserSettings {
    rel_path: PathBuf,
    settings: DesktopUserSettings,
}

impl UserSettings {
    /// Loads the settings for the given user.
    pub fn load(user: &ResourceId) -> Result<Self> {
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

    pub fn save(&self) -> Result {
        let fh = fs::OpenOptions::new().write(true).open(self.path())?;
        Ok(serde_json::to_writer_pretty(fh, &self.settings)?)
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

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
