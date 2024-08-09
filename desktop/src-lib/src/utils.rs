use std::path::{Path, PathBuf};

/// Joins an absolute path as if it were a relative path.
///
/// # Arguments
/// 1. `root`: Base path.
/// 2. `child`: Path to be joined.
///
/// # Examples
/// ```rust
/// let root = "/user/syre/project/data"
/// let child = "/child/grandchild"
///
/// assert_eq!(join_path_absolute(&root, "/"), root);
/// assert_eq!(join_path_absolute(&root, &child), "/user/syre/project/data/child/grandchild");
/// ```
///
/// # See also
/// + [`syre_local_database::common::container_system_path`]
pub fn join_path_absolute(root: impl AsRef<Path>, child: impl AsRef<Path>) -> PathBuf {
    root.as_ref()
        .components()
        .chain(child.as_ref().components().skip(1))
        .collect()
}
