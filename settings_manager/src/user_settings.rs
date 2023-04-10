//! Settings meant for use on a user by user basis.
//! These settings have a fixed base path and variable relative path.
use crate::settings::{self, Settings};
use crate::Result;
use cluFlock::FlockLock;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

// *********************
// *** User Settings ***
// *********************

/// User settings have a fixed base path with a variable relative path.
pub trait UserSettings<S>: Settings<S>
where
    S: Serialize + DeserializeOwned + Clone,
{
    /// Returns the base path to the settings file.
    fn base_path() -> &'static Path;

    /// Returns the relative path for the settings.
    fn rel_path(&self) -> &Path;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> PathBuf {
        Self::base_path().join(self.rel_path())
    }
}

// **************
// *** Loader ***
// **************

pub struct Loader<S> {
    data: S,
    rel_path: PathBuf,
    file_lock: FlockLock<File>,
}

impl<S> Loader<S> {
    /// Loads the settings from the file given by path.
    pub fn load_or_create<T>(rel_path: PathBuf) -> Result<Loader<S>>
    where
        T: UserSettings<S>,
        S: Serialize + DeserializeOwned + Clone + Default,
    {
        let mut path = T::base_path().to_path_buf();
        path.push(rel_path.clone());

        let (data, file_lock) = settings::load_or_create::<S>(path.as_path())?;
        Ok(Loader {
            data,
            rel_path,
            file_lock,
        })
    }
}

impl<S> Loader<S> {
    pub fn rel_path(self) -> PathBuf {
        self.rel_path
    }

    pub fn data(self) -> S {
        self.data
    }

    pub fn file_lock(self) -> FlockLock<File> {
        self.file_lock
    }
}

impl<S> Into<Components<S>> for Loader<S> {
    fn into(self) -> Components<S> {
        Components {
            data: self.data,
            rel_path: self.rel_path,
            file_lock: self.file_lock,
        }
    }
}

pub struct Components<S> {
    pub data: S,
    pub rel_path: PathBuf,
    pub file_lock: FlockLock<File>,
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
