//! Common types for `Command`s.
pub use serde::Serialize;
pub use thot_core::project::Metadata;

// ************
// *** Bulk ***
// ************

#[derive(Serialize, Clone, Default, Debug)]
pub struct TagsAction {
    pub insert: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct MetadataAction {
    pub insert: Metadata,
    pub remove: Vec<String>,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct ResourcePropertiesUpdate {
    pub name: Option<String>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}
