//! Settings meant for use on a user by user basis.
//! These settings have a fixed base path and variable relative path.
use crate::settings::{self, Settings};
use crate::Result;
use std::path::{Path, PathBuf};

// *********************
// *** User Settings ***
// *********************

pub trait UserSettings<'a>: Settings {
    fn new(base_path: PathBuf, rel_path: PathBuf);

    /// Returns the base path to the settings file.
    fn base_path() -> &'static Path;

    /// Returns the relative path for the settings.
    fn rel_path(&self) -> &'a Path;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> PathBuf {
        Self::base_path().join(self.rel_path())
    }

    /// Saves the settings to the file given by path.
    fn save(&mut self) -> Result {
        settings::save::<Self>(self)
    }
}

// **************************
// *** Lock Settings File ***
// **************************

/// Standard way to lock settings file.
pub trait LockSettingsFile<'a>: UserSettings<'a> {
    fn acquire_lock(&mut self) -> Result {
        // check lock is not already acquired
        if self.file().is_some() {
            // lock already acquired
            return Ok(());
        }

        let file = settings::ensure_file(&self.path())?;
        let file_lock = settings::lock(file)?;

        self.store_lock(file_lock);
        Ok(())
    }
}

// **************
// *** Loader ***
// **************

#[derive(Default, Clone)]
struct NoPath;

#[derive(Default, Clone)]
struct RelPath(PathBuf);

#[derive(Default, Clone)]
pub struct Loader<P> {
    base_path: PathBuf,
    rel_path: P,
}

impl Loader<NoPath> {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            rel_path: NoPath,
        }
    }

    /// Sets the relative path for the settings.
    pub fn set_rel_path(self, path: PathBuf) -> Loader<RelPath> {
        Loader {
            base_path: self.base_path,
            rel_path: RelPath(path),
        }
    }
}

impl Loader<RelPath> {
    /// Loads the settings from the file given by path.
    pub fn load_or_create<T>(self) -> Result<T>
    where
        T: Settings + Default,
    {
        let base_path = self.base_path;
        let rel_path = self.rel_path.0;
        let mut path = base_path.clone();
        path.push(self.rel_path.0);

        let mut sets = settings::load_or_create::<T>(path.as_path())?;
        sets.set_rel_path(PathBuf::from(rel_path))?;

        Ok(sets)
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
