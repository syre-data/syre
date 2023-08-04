//! Settings meant for local use.
//! Local settings all have the same realtive path, with a variable base path.
use super::settings::{self, Settings};
use crate::Result;
use cluFlock::FlockLock;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

// **********************
// *** Local Settings ***
// **********************

/// Local settings have a variable base path and fixed relative path.
pub trait LocalSettings<S>: Settings<S>
where
    S: Serialize + DeserializeOwned + Clone,
{
    /// Returns the relative path to the settings file.
    fn rel_path() -> PathBuf;

    /// Returns the base path for the settings.
    fn base_path(&self) -> &Path;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> PathBuf {
        self.base_path().join(Self::rel_path())
    }
}

// **************
// *** Loader ***
// **************

pub struct Loader<S> {
    data: S,
    base_path: PathBuf,
    file_lock: FlockLock<File>,
}

impl<S> Loader<S> {
    /// Loads the settings from the file given by path.
    pub fn load_or_create<T>(base_path: PathBuf) -> Result<Loader<S>>
    where
        T: LocalSettings<S>,
        S: Serialize + DeserializeOwned + Clone + Default,
    {
        let mut path = base_path.clone();
        path.push(T::rel_path());

        let (data, file_lock) = settings::load_or_create::<S>(path.as_path())?;
        Ok(Loader {
            data,
            base_path,
            file_lock,
        })
    }

    /// Loads the settings from the file given by path.
    pub fn load_or_create_with<T>(base_path: PathBuf, default: S) -> Result<Loader<S>>
    where
        T: LocalSettings<S>,
        S: Serialize + DeserializeOwned + Clone + Default,
    {
        let mut path = base_path.clone();
        path.push(T::rel_path());

        let (data, file_lock) = settings::load_or_create_with::<S>(path.as_path(), default)?;
        Ok(Loader {
            data,
            base_path,
            file_lock,
        })
    }
}

impl<S> Loader<S> {
    pub fn base_path(self) -> PathBuf {
        self.base_path
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
            base_path: self.base_path,
            file_lock: self.file_lock,
        }
    }
}

pub struct Components<S> {
    pub data: S,
    pub base_path: PathBuf,
    pub file_lock: FlockLock<File>,
}

#[cfg(test)]
#[path = "./local_settings_test.rs"]
mod local_settings_test;
