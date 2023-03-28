//! Settings meant for local use.
//! Local settings all have the same realtive path, with a variable base path.
use crate::{settings, Result};
use std::path::{Path, PathBuf};

// **********************
// *** Local Settings ***
// **********************

/// Functionality required for local settings.
///
/// # Function and Methods
/// + **`set_base_path`:** Sets the base path for the specifc object.
/// + **`base_path`:** Returns the base path of the specifc object.
/// + **`rel_path`:** Relative path to the settings file from the base path.
/// + **`path`:** Full path to the settings file.
/// + **`load`:** Locks and loads the settings file.
/// + **`save`:** Saves the object to its path.
pub trait LocalSettings: settings::Settings {
    /// Returns the relative path to the settings file.
    fn rel_path() -> Result<PathBuf>;

    /// Returns the base path for the settings.
    fn base_path(&self) -> Result<PathBuf>;

    /// Sets the base path for the settings.
    fn set_base_path(&mut self, path: PathBuf) -> Result;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> Result<PathBuf> {
        let bp = self.base_path()?;
        let rp = Self::rel_path()?;

        Ok(bp.join(rp))
    }

    /// Loads the settings from the file given by path.
    fn load_or_default(base_path: &Path) -> Result<Self>
    where
        Self: Default,
    {
        let r_path = Self::rel_path()?;
        let path = base_path.join(r_path);
        let mut sets = settings::load_or_default::<Self>(path.as_path())?;
        sets.set_base_path(PathBuf::from(base_path))?;

        Ok(sets)
    }

    /// Saves the settings to the file given by path.
    fn save(&mut self) -> Result {
        settings::save::<Self>(self)
    }
}

// ************************
// *** LockSettingsFile ***
// ************************

///  Standard way to acquire a file lock for the settings file.
pub trait LockSettingsFile: LocalSettings {
    fn acquire_lock(&mut self) -> Result {
        // check lock is not already acquired
        if self.file().is_some() {
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
#[path = "./local_settings_test.rs"]
mod local_settings_test;
