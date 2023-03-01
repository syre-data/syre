//! All settings.
use crate::error::{DesktopSettingsError, Result};
use cluFlock::FlockLock;
use core::str::FromStr;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use settings_manager::error::SettingsError as UserSettingsError;
use settings_manager::{
    Error as SettingsError, Priority as SettingsPriority, Result as SettingsResult, Settings,
    UserSettings as UserSettingsInterface,
};
use std::fs::File;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::identifier::Identifier;
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::UserSettings as DesktopUserSettings;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _user: Option<ResourceId>,

    pub settings: DesktopUserSettings,
}

impl UserSettings {
    pub fn new(user: ResourceId) -> Self {
        Self {
            _file_lock: None,
            _user: None,
            settings: DesktopUserSettings::new(user),
        }
    }

    pub fn load_user(user: ResourceId) -> SettingsResult<Self> {
        // @todo: Verify loaded user and user in file name match.
        let mut settings = Self::load(&Self::user_path(user.clone()))?;
        settings.user = user;
        Ok(settings)
    }

    /// Updates the app state.
    pub fn update(&mut self, settings: DesktopUserSettings) -> Result {
        // verify correct user
        if settings.user != self.user {
            return Err(
                DesktopSettingsError::InvalidUpdate("users do not match".to_string()).into(),
            );
        }

        self.settings = settings;
        Ok(())
    }

    fn user_path(user: ResourceId) -> PathBuf {
        let mut path = PathBuf::from(user.to_string());
        path.set_extension("json");
        path
    }

    /// Returns directories for the user's Thot.
    fn dirs() -> SettingsResult<ProjectDirs> {
        let dirs_opt = ProjectDirs::from(
            &Identifier::qualifier(),
            &Identifier::organization(),
            &Identifier::application(),
        );

        match dirs_opt {
            Some(dirs) => Ok(dirs),
            None => Err(SettingsError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "system settings directory not found",
            ))),
        }
    }

    /// Returns the path to the users config directory for Thot.
    fn dir_path() -> SettingsResult<PathBuf> {
        let dirs = Self::dirs()?;
        let mut path = dirs.config_dir().to_path_buf();
        path.push("user_settings");
        Ok(path.to_path_buf())
    }
}

impl Clone for UserSettings {
    fn clone(&self) -> Self {
        Self {
            _file_lock: None,
            _user: self._user.clone(),
            settings: self.settings.clone(),
        }
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

impl Settings for UserSettings {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
    }

    fn file(&self) -> Option<&File> {
        match self._file_lock.as_ref() {
            None => None,
            Some(lock) => Some(&*lock),
        }
    }

    fn file_mut(&mut self) -> Option<&mut File> {
        match self._file_lock.as_mut() {
            None => None,
            Some(lock) => Some(lock),
        }
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl UserSettingsInterface for UserSettings {
    fn base_path() -> SettingsResult<PathBuf> {
        let d = Self::dir_path()?;
        Ok(d)
    }

    fn rel_path(&self) -> SettingsResult<PathBuf> {
        Ok(Self::user_path(self.user.clone()))
    }

    fn set_rel_path(&mut self, path: PathBuf) -> SettingsResult {
        // get user id from path
        let Some(rid) = path.file_prefix() else {
            return Err(SettingsError::SettingsError(UserSettingsError::InvalidPath(path)));
        };

        let Some(rid) = rid.to_str() else {
            return Err(SettingsError::SettingsError(UserSettingsError::InvalidPath(path)));
        };

        let Ok(rid) = ResourceId::from_str(rid) else {
            return Err(SettingsError::SettingsError(UserSettingsError::InvalidPath(path)));
        };

        self._user = Some(rid);
        Ok(())
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
