//! Common use functions.
use crate::constants::*;
use crate::{Error, Result};
use regex::Regex;
use std::path::{Component, Path, PathBuf, Prefix, MAIN_SEPARATOR};
use std::{fs, io};

/// Creates a unique file name.
pub fn unique_file_name(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    match fs::canonicalize(path) {
        Ok(canon_path) if path != canon_path => return Ok(path.to_path_buf()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(path.to_path_buf()),
        Err(err) => return Err(err.into()),
        _ => {}
    }

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

    let Some(parent) = path.parent() else {
        return Err(Error::InvalidPath(path.to_path_buf()));
    };

    // get highest counter
    let name_pattern = Regex::new(&format!(r" \((\d+)\){ext}$")).unwrap();
    let mut highest = None;
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let entry_path = entry.path();

        let Some(entry_path_str) = entry_path.to_str() else {
            continue;
        };

        let Some(captures) = name_pattern.captures(entry_path_str) else {
            continue;
        };

        if let Some(n) = captures.get(1) {
            let Ok(n) = n.as_str().parse::<u32>() else {
                continue;
            };

            match highest {
                None => {
                    let n = std::cmp::max(n, 1);
                    let _ = highest.insert(n);
                }
                Some(m) if n > m => {
                    let _ = highest.insert(n);
                }
                _ => {}
            }
        }
    }

    // set unique file name
    let mut file_name = file_prefix.to_string();
    match highest {
        None => file_name.push_str(" (1)"),

        Some(n) => {
            let match_len = &format!("({n})").len();
            let replace_range = (file_prefix.len() - match_len)..;
            file_name.replace_range(replace_range, &format!("({})", n + 1));
        }
    };
    file_name.push_str(ext);

    let mut unique_path = path.to_path_buf();
    unique_path.set_file_name(file_name);
    Ok(unique_path)
}

/// Replaces any non-alphanumeric or standard characters with underscore (_).
pub fn sanitize_file_path(path: impl Into<String>) -> String {
    let path: String = path.into();
    let char_whitelist = vec!['-', '_', '.', ' ', '(', ')', '[', ']'];
    path.chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char_whitelist.contains(&char) {
                char
            } else {
                '_'
            }
        })
        .collect()
}

/// Normalizes path separators to the current systems.
///
/// On Windows this is `\\`.
/// On all other systems this is `/`.
pub fn normalize_path_separators(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .components()
        .fold(PathBuf::new(), |path, component| match component {
            Component::RootDir => path.join(MAIN_SEPARATOR.to_string()),
            Component::Prefix(prefix) => path.join(prefix.as_os_str()),
            Component::Normal(segment) => path.join(segment),
            _ => {
                panic!("invalid path component");
            }
        })
}

/// Prefixes the path with the [Windows UNC](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats#unc-paths) path if it is not already there.
pub fn ensure_windows_unc(path: impl Into<PathBuf>) -> PathBuf {
    let path: PathBuf = path.into();
    if path.to_str().unwrap().starts_with(WINDOWS_UNC_PREFIX) {
        path
    } else {
        // Must prefix UNC path as `str` because using `Path`s strips it.
        let mut p = WINDOWS_UNC_PREFIX.to_string();
        p.push_str(path.to_str().unwrap());
        PathBuf::from(p)
    }
}

/// Strip the UNC prefix from a Windows path.
/// If the UNC prefix is not present, the path is returned as is.
pub fn strip_windows_unc(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .components()
        .filter(|component| match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::Disk(_) => true,
                _ => false,
            },
            _ => true,
        })
        .fold(PathBuf::new(), |path, component| path.join(component))
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
pub fn thot_dir_of(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().join(THOT_DIR)
}

// --- project ---
/// Path to the project file for a given path.
pub fn project_file() -> PathBuf {
    thot_dir().join(PROJECT_FILE)
}

/// Path to the project file for a given path.
/// thot_dir(path)/\<PROJECT_FILE\>
pub fn project_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(PROJECT_FILE)
}

// --- project settings ---
/// Path to the project settings file relative to a base path.
pub fn project_settings_file() -> PathBuf {
    thot_dir().join(PROJECT_SETTINGS_FILE)
}

/// Path to the project settings file for a given path.
/// thot_dir(path)/\<PROJECT_SETTINGS_FILE\>
pub fn project_settings_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(PROJECT_SETTINGS_FILE)
}

// --- container ---
/// Path to the Container file from a base path.
pub fn container_file() -> PathBuf {
    thot_dir().join(CONTAINER_FILE)
}

/// Path to the Container file for a given path.
/// thot_dir(path)/\<CONTAINER_FILE\>
pub fn container_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(CONTAINER_FILE)
}

// --- container settings ---
/// Path to the Container settings file from a base path.
pub fn container_settings_file() -> PathBuf {
    thot_dir().join(CONTAINER_SETTINGS_FILE)
}

/// Path to the Container settings file for a given path.
/// thot_dir(path)/\<CONTAINER_SETTINGS_FILE\>
pub fn container_settings_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(CONTAINER_SETTINGS_FILE)
}

// --- assets ---
/// Path to the Assets file from a base path.
pub fn assets_file() -> PathBuf {
    thot_dir().join(ASSETS_FILE)
}

/// Path to the Assets file for a given path.
/// thot_dir(path)/\<ASSETS_FILE\>
pub fn assets_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(ASSETS_FILE)
}

// --- scirpts ---
/// Path to the Assets file from a base path.
pub fn scripts_file() -> PathBuf {
    thot_dir().join(SCRIPTS_FILE)
}

/// Path to the Assets file for a given path.
/// thot_dir(path)/\<SCRIPTS_FILE\>
pub fn scripts_file_of(path: impl AsRef<Path>) -> PathBuf {
    thot_dir_of(path).join(SCRIPTS_FILE)
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
