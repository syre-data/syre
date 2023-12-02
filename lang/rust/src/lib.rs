//! Base language binding functionality for Thot.
pub mod database;
pub mod error;

// Re-exports
pub use database::Database;
pub use error::{Error, Result};
