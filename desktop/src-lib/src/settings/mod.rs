//! Settings and related traits.
pub mod general;
pub mod has_user;
pub mod user_app_state;
pub mod user_settings;
pub mod user_settings_file;

// Re-exports
pub use general::GeneralSettings;
pub use has_user::HasUser;
pub use user_app_state::UserAppState;
pub use user_settings::UserSettings;
pub use user_settings_file::UserSettingsFile;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
