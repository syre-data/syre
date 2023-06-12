//! Bulk editors.
pub mod metadata;
pub mod standard_properties;
pub mod tags;

// Re-exports
pub use standard_properties::StandardPropertiesBulkEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
