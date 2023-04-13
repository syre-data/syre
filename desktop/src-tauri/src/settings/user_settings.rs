//! All settings.
use crate::common;
use crate::error::{DesktopSettingsError, Result};
use cluFlock::FlockLock;
use settings_manager::user_settings::{
    Components, Loader as UserLoader, UserSettings as UserSettingsInterface,
};
use settings_manager::Settings;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_desktop_lib::settings::UserSettingsFile;
use thot_desktop_lib::settings::{HasUser, UserSettings as DesktopUserSettings};

#[derive(Settings)]
pub struct UserSettings {
    #[settings(file_lock = "DesktopUserSettings")]
    file_lock: FlockLock<File>,

    rel_path: PathBuf,

    #[settings(priority = "User")]
    settings: DesktopUserSettings,
}

impl UserSettings {
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

impl UserSettingsInterface<DesktopUserSettings> for UserSettings {
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

impl From<UserLoader<DesktopUserSettings>> for UserSettings {
    fn from(loader: UserLoader<DesktopUserSettings>) -> Self {
        let loader: Components<DesktopUserSettings> = loader.into();
        Self {
            file_lock: loader.file_lock,
            rel_path: loader.rel_path,
            settings: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
