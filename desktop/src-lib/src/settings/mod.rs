//! Settings.
pub mod general;
pub mod user_app_state;
pub mod user_settings;

// Re-exports
pub use general::GeneralSettings;
pub use user_app_state::UserAppState;
pub use user_settings::UserSettings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
