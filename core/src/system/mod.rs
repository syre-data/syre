//! System setting resources for Thot.
pub mod user;

#[cfg(feature = "serde")]
pub mod template;

// Reexports
pub use user::User;
