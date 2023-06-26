//! Componients to edit a [`Container`]'s [`ScriptAssociation`]s.
pub mod add_script_association;
pub mod script_associations_editor;
pub mod script_associations_preview;

// Re-exports
pub use add_script_association::AddScriptAssociation;
pub use script_associations_editor::{NameMap, ScriptAssociationsEditor};
pub use script_associations_preview::ScriptAssociationsPreview;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
