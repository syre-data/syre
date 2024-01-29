//! System setting resources for Syre.
pub mod user;

#[cfg(feature = "serde")]
pub mod template;

// Reexports
pub use user::User;
