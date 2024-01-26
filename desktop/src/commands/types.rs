//! Common types for `Command`s.
use serde::Serialize;
use thot_local_database::command::types::{MetadataAction, TagsAction};

#[derive(Serialize, Clone, Default, Debug)]
pub struct ResourcePropertiesUpdate {
    pub name: Option<String>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}
