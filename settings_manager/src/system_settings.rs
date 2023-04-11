//! Settings meant for system wide use.
//! These settings have a fixed path.
use crate::settings::{self, Settings};
use crate::Result;
use cluFlock::FlockLock;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::path::PathBuf;

// ***********************
// *** System Settings ***
// ***********************

/// System settings have only one file for the entire system.
pub trait SystemSettings<S>: Settings<S>
where
    S: Serialize + DeserializeOwned + Clone,
{
    /// Returns the path to the settings file.
    fn path() -> PathBuf;
}

// **************
// *** Loader ***
// **************

pub struct Loader<S> {
    data: S,
    file_lock: FlockLock<File>,
}

impl<S> Loader<S> {
    /// Loads the settings from the file given by path.
    pub fn load_or_create<T>() -> Result<Loader<S>>
    where
        T: SystemSettings<S>,
        S: Serialize + DeserializeOwned + Clone + Default,
    {
        let (data, file_lock) = settings::load_or_create::<S>(&T::path())?;

        Ok(Loader { data, file_lock })
    }
}

impl<S> Loader<S> {
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
            file_lock: self.file_lock,
        }
    }
}

pub struct Components<S> {
    pub data: S,
    pub file_lock: FlockLock<File>,
}

#[cfg(test)]
#[path = "./system_settings_test.rs"]
mod system_settings_test;
