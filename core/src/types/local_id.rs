use super::resource_id::ResourceId;
use super::resource_path::ResourcePath;
use std::path::PathBuf;

/// Ids for local resources.
pub enum LocalId {
    ResourceId(ResourceId),
    Path(PathBuf),
    ResourcePath(ResourcePath),
}
