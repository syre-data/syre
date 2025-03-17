use std::path::{Path, PathBuf};

/// Removes the root path (`/`) from a path.
///
/// # Examples
/// ```rust
/// let absolute_path = "/path/to/folder";
/// assert_eq!(remove_root_path(absolute_path, "path/to/folder"));
/// ```
pub fn remove_root_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().components().skip(1).collect()
}

