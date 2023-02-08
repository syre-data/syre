//! System setting resources for Thot.
pub mod project;
pub mod user_preferences;

#[cfg(feature = "fs")]
pub mod script;

// Re-exports
pub use project::Project;
pub use user_preferences::UserPreferences;

#[cfg(feature = "fs")]
pub use script::Script;
