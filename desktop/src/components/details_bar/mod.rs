//! Details bar for the canvas.
pub mod container_editor;
pub mod details_bar;
pub mod project_actions;
pub mod project_scripts;
pub mod script_associations_editor;
pub mod script_editor;

// Re-exports
pub use details_bar::{DetailsBar, DetailsBarWidget};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
