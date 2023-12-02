//! Indicate a resource is backed by a file.
use std::path::{Path, PathBuf};

// **********************
// *** Local Resource ***
// **********************

/// Local resources have a variable base path and fixed relative path.
/// i.e. <variable>/<fixed>
pub trait LocalResource<T> {
    /// Returns the (fixed) relative path to the settings file.
    fn rel_path() -> PathBuf;

    /// Returns the (variable) base path for the settings.
    fn base_path(&self) -> &Path;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> PathBuf {
        self.base_path().join(Self::rel_path())
    }
}

// *********************
// *** User Resource ***
// *********************

/// User resources have a fixed base path with a variable relative path.
/// i.e. <fixed>/<variable>
pub trait UserResource<T> {
    /// Returns the (fixed) base path to the settings file.
    fn base_path() -> PathBuf;

    /// Returns the (variable) relative path for the settings.
    fn rel_path(&self) -> &Path;

    /// Returns the absolute path to the settings file.
    fn path(&self) -> PathBuf {
        Self::base_path().join(self.rel_path())
    }
}

// ***********************
// *** System Resource ***
// ***********************

/// System resources have only one file for the entire system that resides at a fixed path.
pub trait SystemResource<T> {
    /// Returns the path to the settings file.
    fn path() -> PathBuf;
}
