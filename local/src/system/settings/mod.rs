//! System settings for Thot.
pub mod user_settings;

// Re-exports
pub use user_settings::UserSettings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
