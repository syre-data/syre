//! Common types for `Command`s.
pub use serde::Serialize;

// ************
// *** Bulk ***
// ************

#[derive(Serialize, Clone, Default, Debug)]
pub struct ListAction<T> {
    pub add: Vec<T>,
    pub remove: Vec<T>,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct StandardPropertiesUpdate {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: ListAction<String>,
    pub metadata: ListAction<String>,
}
