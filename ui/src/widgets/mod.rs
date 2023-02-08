//! UI Widgets
pub mod asset;
pub mod container;
pub mod metadata;
pub mod project;
pub mod script;
pub mod standard_properties_editor;
pub mod suspense;
pub mod tags_editor;

// Re-exports
pub use metadata::MetadataEditor;
pub use standard_properties_editor::StandardPropertiesEditor;
pub use tags_editor::TagsEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
