//! Database for storing resources.
//! Because multiple local executales may need access to the same resouce,
//! the database acts as the single source of truth.
pub(self) mod database;
mod event;
pub(self) mod store;

// Re-exports
pub use database::Database;
pub(self) use event::Event;
