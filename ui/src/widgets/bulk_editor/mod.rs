//! Bulk editors.
pub mod resource_properties;
pub mod script_associations;
pub mod tags;
pub mod types;

// Re-exports
pub use resource_properties::ResourcePropertiesBulkEditor;
pub use script_associations::{RunParametersUpdate, ScriptAssociationsBulkEditor, ScriptBulkMap};
