//! Tag widgets.
pub mod tags_editor;
pub mod tags;

// Re-exports
pub use tags::Tags;
pub use tags_editor::TagsEditor;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
