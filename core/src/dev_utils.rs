//! [`ResourcePath`] utilities.
use crate::types::ResourcePath;
use fake::faker::filesystem::raw::FilePath;
use fake::locales::EN;
use fake::Fake;
use std::ffi::OsStr;
use std::path::PathBuf;

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
