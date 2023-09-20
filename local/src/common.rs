//! Common use functions.
use crate::constants::*;
use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Canonicalizes a path.
///
/// # Notes
/// Currently delegates to std::fs::canonicalize, but reserved for
/// easy future changes.
pub fn canonicalize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(err) => Err(Error::from(err)),
    }
}

/// Creates a unique file name.
pub fn unique_file_name(path: PathBuf) -> Result<PathBuf> {
    // get file name
    let Some(file_prefix) = path.file_prefix() else {
        return Err(Error::InvalidPath(path.to_path_buf()));
    };

    let Some(file_prefix) = file_prefix.to_str() else {
        return Err(Error::InvalidPath(path.to_path_buf()));
    };

    // get extension
    let Some(ext) = path.file_name() else {
        return Err(Error::InvalidPath(path.to_path_buf()));
    };

    let Some(ext) = ext.to_str() else {
        return Err(Error::InvalidPath(path.to_path_buf()));
    };

    let ext = &ext[file_prefix.len()..];

    // create unique file name
    let mut u_path = path.to_path_buf();
    let mut counter: usize = 0;
    while u_path.exists() {
        counter += 1;
        let u_file_name = format!("{file_prefix}-{counter}{ext}");

        u_path = path.to_path_buf();
        u_path.set_file_name(u_file_name);
    }

    Ok(u_path)
}

/// Replaces any non-alphanumeric or standard characters with underscore (_).
pub fn sanitize_file_path(path: impl Into<String>) -> String {
    let path: String = path.into();
    path.chars()
        .map(|char| {
            if char.is_ascii_alphanumeric()
                || char == '-'
                || char == '_'
                || char == '.'
                || char == ' '
            {
                char
            } else {
                '_'
            }
        })
        .collect()
}

// ******************
// *** file paths ***
// ******************

// --- thot directory ---
/// Returns the relative path to the Thot directory from a base path.
pub fn thot_dir() -> &'static Path {
    Path::new(THOT_DIR)
}

/// Path to the Thot directory for a given path.
/// \<path\>/\<THOT_DIR\>.
pub fn thot_dir_of(path: &Path) -> PathBuf {
    path.join(THOT_DIR)
}

// --- project ---
/// Path to the project file for a given path.
pub fn project_file() -> PathBuf {
    thot_dir().join(PROJECT_FILE)
}

/// Path to the project file for a given path.
/// thot_dir(path)/\<PROJECT_FILE\>
pub fn project_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(PROJECT_FILE)
}

// --- project settings ---
/// Path to the project settings file relative to a base path.
pub fn project_settings_file() -> PathBuf {
    thot_dir().join(PROJECT_SETTINGS_FILE)
}

/// Path to the project settings file for a given path.
/// thot_dir(path)/\<PROJECT_SETTINGS_FILE\>
pub fn project_settings_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(PROJECT_SETTINGS_FILE)
}

// --- container ---
/// Path to the Container file from a base path.
pub fn container_file() -> PathBuf {
    thot_dir().join(CONTAINER_FILE)
}

/// Path to the Container file for a given path.
/// thot_dir(path)/\<CONTAINER_FILE\>
pub fn container_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(CONTAINER_FILE)
}

// --- container settings ---
/// Path to the Container settings file from a base path.
pub fn container_settings_file() -> PathBuf {
    thot_dir().join(CONTAINER_SETTINGS_FILE)
}

/// Path to the Container settings file for a given path.
/// thot_dir(path)/\<CONTAINER_SETTINGS_FILE\>
pub fn container_settings_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(CONTAINER_SETTINGS_FILE)
}

// --- assets ---
/// Path to the Assets file from a base path.
pub fn assets_file() -> PathBuf {
    thot_dir().join(ASSETS_FILE)
}

/// Path to the Assets file for a given path.
/// thot_dir(path)/\<ASSETS_FILE\>
pub fn assets_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(ASSETS_FILE)
}

// --- scirpts ---
/// Path to the Assets file from a base path.
pub fn scripts_file() -> PathBuf {
    thot_dir().join(SCRIPTS_FILE)
}

/// Path to the Assets file for a given path.
/// thot_dir(path)/\<SCRIPTS_FILE\>
pub fn scripts_file_of(path: &Path) -> PathBuf {
    thot_dir_of(path).join(SCRIPTS_FILE)
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
