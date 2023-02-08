//! [`ResourcePath`] utilities.
use fake::faker::filesystem::raw::FilePath;
use fake::locales::EN;
use fake::Fake;
use std::ffi::OsStr;
use std::path::PathBuf;
use thot_core::types::ResourcePath;

/// Creates a random [`ResourcePath`].
///
/// # Arguments
/// 1. The desired path extension.
pub fn resource_path<S: AsRef<OsStr>>(ext: Option<S>) -> ResourcePath {
    let mut path = PathBuf::from(FilePath(EN).fake::<String>());
    if let Some(ext) = ext {
        path.set_extension(ext);
    }

    ResourcePath::new(path).expect("creating resource path should work")
}

#[cfg(test)]
#[path = "./resource_path_test.rs"]
mod resource_path_test;
