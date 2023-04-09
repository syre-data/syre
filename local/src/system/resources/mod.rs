//! System setting resources for Thot.
pub mod user_preferences;

#[cfg(feature = "fs")]
pub mod script;

// Re-exports
pub use user_preferences::UserPreferences;

#[cfg(feature = "fs")]
pub use script::Script;
