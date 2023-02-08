// @remove: Module.
//! DEPRICATED
//! [`Container`](thot_core::project::Container) editor.
pub mod assets_list;
pub mod main;
pub mod properties_editor;
pub mod scripts;

// Re-exports
pub use assets_list::AssetsList;
pub use main::ContainerEditor;
pub use properties_editor::PropertiesEditor;
pub use scripts::ScriptAssociationsEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
