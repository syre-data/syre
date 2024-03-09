//! Componients to edit a [`Container`]'s [`ScriptAssociation`]s.
pub mod add_association;
pub mod associations_editor;
pub mod associations_preview;

// Re-exports
pub use add_association::AddAnalysisAssociation;
pub use associations_editor::{NameMap, ScriptAssociationsEditor};
pub use associations_preview::AnalysisAssociationsPreview;
