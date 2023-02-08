//! Settings meant for system wide use.
//! These settings have a fixed path.
use crate::{settings, Result};
use std::path::PathBuf;

// ***********************
// *** System Settings ***
// ***********************

/// Required functionality for system settings.
/// System settings have only one file for the entire system.
///
/// # Functions and Methods
/// + **`path`:** Path to the settings file.
/// + **`load`:** Loads the settings file into an object.
/// + **`save`:** Save an object to the settings file.
pub trait SystemSettings: settings::Settings {
    /// Returns the path to the settings file.
    fn path() -> Result<PathBuf>;

    /// Loads the settings from the file given by path.
    fn load() -> Result<Self> {
        let path = Self::path()?;
        settings::load::<Self>(path.as_path())
    }

    /// Saves the settings to the file given by path.
    fn save(&self) -> Result {
        let path = Self::path()?;
        settings::save::<Self>(&self, path.as_path())
    }
}

// **************************
// *** Lock Settings File ***
// **************************

/// Standard way to lock settings file.
pub trait LockSettingsFile: SystemSettings {
    fn acquire_lock(&mut self) -> Result {
        // check lock is not already acquired
        if self.controls_file() {
            // lock already acquired
            return Ok(());
        }

        let path = Self::path()?;
        let file = settings::ensure_file(path.as_path())?;
        let file_lock = settings::lock(file)?;

        self.store_lock(file_lock);
        Ok(())
    }
}

#[cfg(test)]
#[path = "./system_settings_test.rs"]
mod system_settings_test;
