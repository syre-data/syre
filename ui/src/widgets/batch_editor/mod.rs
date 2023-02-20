//! Batch editors.
pub mod standard_properties;
pub mod tags;
pub mod metadata;

// Re-exports
pub use standard_properties::StandardPropertiesBatchEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
