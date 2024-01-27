use super::resource_id::ResourceId;
use std::path::PathBuf;

/// Ids for local resources.
pub enum LocalId {
    ResourceId(ResourceId),
    Path(PathBuf),
}
