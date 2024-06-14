//! Database that watches file system for changes, publishing them for clients.
pub(self) mod database;
pub(self) mod state;
pub(self) mod store;
mod types;

pub use database::{Builder, Database};
