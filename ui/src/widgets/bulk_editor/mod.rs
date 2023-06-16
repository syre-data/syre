//! Bulk editors.
pub mod standard_properties;
pub mod tags;
pub mod types;

// Re-exports
pub use standard_properties::StandardPropertiesBulkEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
