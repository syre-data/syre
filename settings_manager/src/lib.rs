//! `settings_manager` provides multiple interfaces for reading and writing
//! settings files to disk.
//!
//! # Types of settings
//! Settings are distinguished by how the path referencing the file is constructed.
//! Each setting is composed of two parts: a base path and relative path.
//! Together, these components create the path referencing the settings file on disk:
//! > \<path\> = \<base path\>/\<relative path\>.
//!
//! + `SystemSettings`: Have a fixed path.
//! + `UserSettings`: Have a fixed base path with variable relative path.
//! + `LocalSettings`: Have a variable base path with fixed relative path.
//!
//! # Locking settings files
//! In order to load or save a settings file a file lock must be acquired.
//! This ensures that another process can not overwrite the settings while
//! it is in use, thus poisoning the settings.
pub mod error;
pub mod local_settings;
pub mod settings;
pub mod system_settings;
pub mod types;
pub mod user_settings;

// Re-exports
pub use error::{Error, Result};
pub use local_settings::LocalSettings;
pub use settings::Settings;
pub use system_settings::SystemSettings;
pub use types::Priority;
pub use user_settings::UserSettings;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
