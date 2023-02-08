use crate::constants::ROOT_DRIVE_ID;
use crate::error::ResourcePathError;
use crate::{Error, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// @todo: Implement custom (de)serialization to interpret type by path.
/// Path types to a given resource.
///
/// # Variants
/// + **Absolute:** An absolute path.
/// + **Relative:** A path relative to the given resource.
/// + **Root:** A path relative to the project root.
///   + First field is the path.
///   + Second field is the `metalevel`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Hash)]
pub enum ResourcePath {
    Absolute(PathBuf),
    Relative(PathBuf),
    Root(PathBuf, usize),
    // @todo: Url may be needed for remote scripts.
    // Url(url::Url),
}

impl ResourcePath {
    /// Create a new ResourcePath based off the format of the path.
    ///
    /// + If `path` begins with the root drive id (`root:`) returns `Root(path, 0)`.
    /// + If `path` begins with the root drive id with a metalevel (`root[<level>]:`) returns
    ///     `Root(path, level).
    /// + If `path` is relative returns `Relative(path)`.
    /// + If `path` is absolute returns `Absolute(path)`.
    ///
    /// # Notes
    /// ## Root paths
    /// Only the relative path from the root is stored for root paths.
    pub fn new(path: PathBuf) -> Result<Self> {
        if Self::is_root(&path) {
            // parse metalevel
            let path_str = path.to_str();
            if path_str.is_none() {
                return Err(Error::ResourcePathError(
                    ResourcePathError::CouldNotParseMetalevel(String::from(
                        "could not convert path to string",
                    )),
                ));
            }

            let rd_pattern = Self::root_drive_regex();
            let caps = rd_pattern.captures(path_str.unwrap());
            if caps.is_none() {
                return Err(Error::ResourcePathError(
                    ResourcePathError::CouldNotParseMetalevel(String::from(
                        "path did not match root pattern",
                    )),
                ));
            }

            let caps = caps.unwrap();
            let metalevel = match caps.get(1) {
                None => 0, // metalevel not set, default ot 0
                Some(m) => match m.as_str().parse::<usize>() {
                    Err(_) => {
                        return Err(Error::ResourcePathError(
                            ResourcePathError::CouldNotParseMetalevel(String::from(
                                "invalid metalevel, could not parse as integer",
                            )),
                        ))
                    }
                    Ok(ml) => ml,
                },
            };

            // extract relative path
            let prefix = caps.get(0).unwrap().as_str();
            let rel_path = path.strip_prefix(prefix);
            if rel_path.is_err() {
                return Err(Error::ResourcePathError(
                    ResourcePathError::CouldNotParseMetalevel(String::from(
                        "could not remove root drive from path",
                    )),
                ));
            }

            let rel_path = rel_path.unwrap().to_path_buf();
            return Ok(Self::Root(rel_path, metalevel));
        }

        if Self::is_relative(&path) {
            return Ok(Self::Relative(path));
        } else {
            return Ok(Self::Absolute(path));
        }
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// Returns whether a path is a root path.
    ///
    /// # See also
    /// + [`Self.is_realtive_path`]
    /// + [`Self.is_absolute_path`]
    pub fn is_root(path: &Path) -> bool {
        let path_str = path.to_str().unwrap();
        path_str.starts_with(ROOT_DRIVE_ID)
    }

    /// Returns whether a path is a relative path.
    ///
    /// # See also
    /// + [`Self.is_root_path`]
    /// + [`Self.is_absolute_path`]
    pub fn is_relative(path: &Path) -> bool {
        !Self::is_root(path) && path.is_relative()
    }

    /// Returns whether a path is an absolute path.
    ///
    /// # See also
    /// + [`Self.is_root_path`]
    /// + [`Self.is_relative_path`]
    pub fn is_absolute(path: &Path) -> bool {
        !(Self::is_root(path) || path.is_relative())
    }

    /// Returns a regex for matching a root path.
    /// First capturing group is the metalevel.
    fn root_drive_regex() -> Regex {
        let level_pattern = r"(?:\[(\d+)\])?";
        let rd = format!("^{ROOT_DRIVE_ID}{level_pattern}:");
        Regex::new(&rd).unwrap()
    }
}

impl Into<PathBuf> for ResourcePath {
    /// # Notes
    /// + For `Root` paths, only the realtive path is converted.
    fn into(self) -> PathBuf {
        match self {
            Self::Absolute(p) => p,
            Self::Relative(p) => p,
            Self::Root(p, _) => p,
        }
    }
}

impl AsRef<Path> for ResourcePath {
    /// # Notes
    /// + For `Root` paths, only the realtive path is converted.
    fn as_ref(&self) -> &Path {
        match &self {
            Self::Absolute(p) => p.as_path(),
            Self::Relative(p) => p.as_path(),
            Self::Root(p, _) => p.as_path(),
        }
    }
}

impl PartialEq for ResourcePath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Absolute(me), Self::Absolute(you)) => me == you,
            (Self::Relative(me), Self::Relative(you)) => me == you,
            (Self::Root(me, ml), Self::Root(you, yl)) => me == you && ml == yl,
            _ => false,
        }
    }
}

impl Eq for ResourcePath {}

#[cfg(test)]
#[path = "./resource_path_test.rs"]
mod resource_path_test;
