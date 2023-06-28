//! Bulk editors.
pub mod script_associations;
pub mod standard_properties;
pub mod tags;
pub mod types;

// Re-exports
pub use script_associations::{RunParametersUpdate, ScriptAssociationsBulkEditor, ScriptBulkMap};
pub use standard_properties::StandardPropertiesBulkEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;