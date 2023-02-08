//! Form components.
pub mod inline_input;
pub mod inline_textarea;

// Re-exports
pub use inline_input::InlineInput;
pub use inline_textarea::InlineTextarea;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
