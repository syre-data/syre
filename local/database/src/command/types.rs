//! Types used for `Command`s.
use serde::{Deserialize, Serialize};
use thot_core::project::Metadata;
use thot_core::types::ResourceId;

/// Actions to be taken on tags.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TagsAction {
    /// Values to insert.
    pub insert: Vec<String>,

    /// Values to remove.
    pub remove: Vec<String>,
}

/// Actions to be taken on metadata.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MetadataAction {
    /// Values to insert, either adding new, or updating.
    pub insert: Metadata,

    /// Values to remove.
    pub remove: Vec<String>,
}
