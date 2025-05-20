use crate::common;
use std::path::{Path, PathBuf};
use syre_local as local;

/// Clamp a value between two others.
pub fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    assert!(min < max);
    if value <= min {
        min
    } else if value >= max {
        max
    } else {
        value
    }
}

/// Creates the absolute path from the file system root to the container.
///
/// # Arguments
/// 1. `data_root`: Absolute path from the file system root to the data root.
/// 2. `container`: Absolute path from the file system root to the container.
///
/// # Examples
/// ```rust
/// let data_root = "/user/syre/project/data"
/// let container = "/child/grandchild"
///
/// assert_eq!(container_system_path(&data_root, "/"), data_root);
/// assert_eq!(container_system_path(&data_root, &container), "/user/syre/project/data/child/grandchild");
/// ```
///
/// # See also
/// + [`syre_project_watcher::common::container_system_path`]
pub fn container_system_path(data_root: impl AsRef<Path>, container: impl AsRef<Path>) -> PathBuf {
    local::common::join_path_absolute(data_root, container)
}

/// Normalize path separators to `/`.
pub fn normalize_path_sep(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .to_string_lossy()
        .replace(common::PATH_SEP_WINDOW, common::PATH_SEP_NIX)
        .into()
}
