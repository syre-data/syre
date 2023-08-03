//! System settings for Thot.
//!
//! This includes modules for tracking
//! + Projects
//! + Scripts
//! + Users
pub mod projects;
pub mod scripts;
pub mod templates;
pub mod users;

// Reexports
pub use projects::Projects;
pub use scripts::Scripts;
pub use templates::Templates;
pub use users::Users;
