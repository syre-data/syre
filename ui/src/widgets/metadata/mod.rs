//! Metadata widgets.
pub mod common;
pub mod metadata_bulk_editor;
pub mod metadata_editor;
pub mod metadata_preview;
pub mod metadatum_builder;
pub mod metadatum_bulk_editor;
pub mod metadatum_bulk_value_editor;
pub mod metadatum_editor;
pub mod metadatum_value_editor;
pub mod types;

// Re-exports
pub use metadata_bulk_editor::MetadataBulkEditor;
pub use metadata_editor::MetadataEditor;
pub use metadata_preview::MetadataPreview;
pub use metadatum_builder::MetadatumBuilder;
pub use metadatum_bulk_editor::MetadatumBulkEditor;
pub use metadatum_bulk_value_editor::MetadatumBulkValueEditor;
pub use metadatum_editor::MetadatumEditor;
pub use metadatum_value_editor::MetadatumValueEditor;
pub use types::{type_from_string, type_of_value, MetadataBulk, Metadatum, MetadatumType};
