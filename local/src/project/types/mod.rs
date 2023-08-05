//! Basic types for local `Project`s.
pub mod container;
pub mod project_settings;

// Re-exports
pub use container::{ContainerProperties, ContainerSettings};
pub use project_settings::ProjectSettings;
