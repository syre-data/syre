//! Metadata widgets.
pub mod metadata_editor;
pub mod metadata_preview;
pub mod metadatum_builder;
pub mod metadatum_editor;
pub mod types;

// Re-exports
pub use metadata_editor::MetadataEditor;
pub use metadata_preview::MetadataPreview;
pub use metadatum_builder::MetadatumBuilder;
pub use metadatum_editor::MetadatumEditor;
pub use types::{type_from_string, type_of_value, Metadatum, MetadatumType};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
