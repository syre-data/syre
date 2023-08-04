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
pub mod error;
pub mod types;

#[cfg(feature = "locked")]
pub mod locked;

// Re-exports
pub use error::{Error, Result};
pub use types::Priority;

#[cfg(feature = "derive_locked")]
use settings_manager_derive_locked;

#[cfg(feature = "derive_locked")]
pub use settings_manager_derive_locked::LockedSettings;
