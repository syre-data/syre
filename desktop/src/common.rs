use std::path::{Path, PathBuf};
use syre_desktop_lib as lib;

pub const APPLICATION_JSON: &'static str = "application/json";

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
/// + [`syre_local_database::common::container_system_path`]
pub fn container_system_path(data_root: impl AsRef<Path>, container: impl AsRef<Path>) -> PathBuf {
    lib::utils::join_path_absolute(data_root, container)
}
