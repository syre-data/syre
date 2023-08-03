//! Database for storing resources.
//! Because multiple local executales may need access to the same resouce,
//! the database acts as the single source of truth.
pub mod database;
pub mod store;

// Re-exports
pub use database::Database;
