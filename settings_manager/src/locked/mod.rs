//! # Locking settings files
//! In order to load or save a settings file a file lock must be acquired.
//! This ensures that another process can not overwrite the settings while
//! it is in use, thus poisoning the settings
pub mod local_settings;
pub mod settings;
pub mod system_settings;
pub mod user_settings;

// Re-exports
pub use local_settings::LocalSettings;
pub use settings::Settings;
pub use system_settings::SystemSettings;
pub use user_settings::UserSettings;
