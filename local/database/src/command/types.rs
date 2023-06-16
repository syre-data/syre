//! Types used for `Command`s.
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

// ************
// *** Bulk ***
// ************

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ListAction<T> {
    pub add: Vec<T>,
    pub remove: Vec<T>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StandardPropertiesUpdate {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: ListAction<String>,
    pub metadata: ListAction<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkUpdatePropertiesArgs {
    pub containers: Vec<ResourceId>,
    pub update: StandardPropertiesUpdate,
}
