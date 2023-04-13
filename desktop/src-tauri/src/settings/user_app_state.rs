//! Application state for startup.
use super::loader::UserSettingsFile;
use crate::common;
use crate::error::{DesktopSettingsError, Result};
use cluFlock::FlockLock;
use settings_manager::user_settings::{
    Components, Loader as UserLoader, UserSettings as UserSettingsInterface,
};
use settings_manager::Settings;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use thot_desktop_lib::settings::UserAppState as DesktopUserAppState;

#[derive(Settings)]
pub struct UserAppState {
    #[settings(file_lock = "DesktopUserAppState")]
    file_lock: FlockLock<File>,

    rel_path: PathBuf,

    #[settings(priority = "User")]
    app_state: DesktopUserAppState,
}

impl UserAppState {
    /// Updates the app state.
    pub fn update(&mut self, app_state: DesktopUserAppState) -> Result {
        // verify correct user
        if app_state.user != self.app_state.user {
            return Err(
                DesktopSettingsError::InvalidUpdate("users do not match".to_string()).into(),
            );
        }

        self.app_state = app_state;
        Ok(())
    }
}

impl Deref for UserAppState {
    type Target = DesktopUserAppState;

    fn deref(&self) -> &Self::Target {
        &self.app_state
    }
}

impl Into<DesktopUserAppState> for UserAppState {
    fn into(self) -> DesktopUserAppState {
        self.app_state
    }
}

impl UserSettingsInterface<DesktopUserAppState> for UserAppState {
    fn base_path() -> PathBuf {
        common::users_config_dir().expect("could not get config path")
    }

    fn rel_path(&self) -> &Path {
        &self.rel_path
    }
}

impl UserSettingsFile for UserAppState {
    fn settings_file() -> PathBuf {
        PathBuf::from("desktop_app_state.json")
    }
}

impl From<UserLoader<DesktopUserAppState>> for UserAppState {
    fn from(loader: UserLoader<DesktopUserAppState>) -> Self {
        let loader: Components<DesktopUserAppState> = loader.into();
        Self {
            file_lock: loader.file_lock,
            rel_path: loader.rel_path,
            app_state: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./user_app_state_test.rs"]
mod user_app_state_test;
