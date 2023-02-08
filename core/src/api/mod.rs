//! API definitions.
pub mod database;

// Re-exports
pub use database::Database;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
