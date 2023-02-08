//! Metadata widgets.
pub mod metadata_editor;
pub mod metadatum_editor;

// Re-exports
pub use metadata_editor::MetadataEditor;
pub use metadatum_editor::{Metadatum, MetadatumEditor};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
