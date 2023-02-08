//! Settings meant for use on a user by user basis.
//! These settings have a fixed base path and variable relative path.
use crate::{settings, Result};
use std::path::{Path, PathBuf};

// *********************
// *** User Settings ***
// *********************

/// Required functionality for user settings.
///
/// # Functions and Methods
/// + **`rel_path`:** Relative path to the settings file.
/// + **`load`:** Loads the settings file into an object.
/// + **`save`:** Save an object to the settings file.
pub trait UserSettings: settings::Settings {
    /// Returns the base path to the settings file.
    fn base_path() -> Result<PathBuf>;

    /// Returns the relative path for the settings.
    fn rel_path(&self) -> Result<PathBuf>;

    /// Sets the relative path for the settings.
    fn set_rel_path(&mut self, path: PathBuf) -> Result;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> Result<PathBuf> {
        let bp = Self::base_path()?;
        let rp = self.rel_path()?;

        Ok(bp.join(rp))
    }

    /// Loads the settings from the file given by path.
    fn load(rel_path: &Path) -> Result<Self> {
        let base_path = Self::base_path()?;
        let path = base_path.join(rel_path);
        let mut sets = settings::load::<Self>(path.as_path())?;
        sets.set_rel_path(PathBuf::from(rel_path))?;

        Ok(sets)
    }

    /// Saves the settings to the file given by path.
    fn save(&self) -> Result {
        let path = self.path()?;
        settings::save::<Self>(&self, path.as_path())
    }
}

// **************************
// *** Lock Settings File ***
// **************************

/// Standard way to lock settings file.
pub trait LockSettingsFile: UserSettings {
    fn acquire_lock(&mut self) -> Result {
        // check lock is not already acquired
        if self.controls_file() {
            // lock already acquired
            return Ok(());
        }

        let path = self.path()?;
        let file = settings::ensure_file(path.as_path())?;
        let file_lock = settings::lock(file)?;

        self.store_lock(file_lock);
        Ok(())
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
