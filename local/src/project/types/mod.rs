//! Basic types for local `Project`s.
pub mod container;
pub mod project_settings;

// Re-exports
pub use container::{ContainerProperties, ContainerSettings};
pub use project_settings::ProjectSettings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
