use crate::constants::ROOT_DRIVE_ID;
use crate::error::ResourcePathError;
use crate::{Error, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// TODO: Implement custom (de)serialization to interpret type by path.
// TODO: Potentially make separate types. `Absolute` would always be canonicalized.
/// Path types to a given resource.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Hash)]
pub enum ResourcePath {
    /// An absolute path.
    Absolute(PathBuf),

    /// A path relative to the given resource.
    Relative(PathBuf),

    /// A path relative to the project root.
    ///
    /// # Fields
    /// 1. Path.
    /// 2. Metalevel.
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
        if let Some((metalevel, rel_path)) = Self::parse_root_path(&path)? {
            return Ok(Self::Root(rel_path, metalevel));
        } else if Self::is_relative(&path) {
            return Ok(Self::Relative(path));
        } else if Self::is_absolute(&path) {
            return Ok(Self::Absolute(path));
        }

        unreachable!("could not parse path as a resource path");
    }

    /// Coerces to a [`Path`] slice.
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    /// Attempts to parse a path as a Root path.
    ///
    /// # Returns
    /// `Some((meta level, relative path))` if successfully parsed, otherwise `None`.
    pub fn parse_root_path(path: &Path) -> Result<Option<(usize, PathBuf)>> {
        let path_str = path.to_str().unwrap();
        let rd_pattern = Self::root_drive_regex();
        let Some(caps) = rd_pattern.captures(path_str) else {
            return Ok(None);
        };

        let metalevel = match caps.get(1) {
            None => 0, // metalevel not set, default ot 0
            Some(m) => match m.as_str().parse::<usize>() {
                Err(_) => {
                    return Err(Error::ResourcePathError(
                        ResourcePathError::could_not_parse_meta_level(
                            "invalid metalevel, could not parse as integer",
                        ),
                    ));
                }
                Ok(ml) => ml,
            },
        };

        // extract relative path
        let prefix = caps.get(0).unwrap().as_str();
        let Ok(rel_path) = path.strip_prefix(prefix) else {
            return Err(Error::ResourcePathError(
                ResourcePathError::could_not_parse_meta_level(
                    "could not remove root drive from path",
                ),
            ));
        };

        Ok(Some((metalevel, rel_path.to_path_buf())))
    }

    /// Returns whether a path is a root path.
    pub fn is_root(path: &Path) -> bool {
        Self::parse_root_path(path).unwrap().is_some()
    }

    /// Returns whether a path is a relative path.
    pub fn is_relative(path: &Path) -> bool {
        !Self::is_root(path) && path.is_relative()
    }

    /// Returns whether a path is an absolute path.
    pub fn is_absolute(path: &Path) -> bool {
        !Self::is_root(path) && path.is_absolute()
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
