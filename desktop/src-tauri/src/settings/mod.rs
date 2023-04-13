//! Settings for Thot desktop app.
pub mod user_app_state;
pub mod user_settings;
pub mod loader;

// Re-exports
pub use user_app_state::UserAppState;
pub use user_settings::UserSettings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
