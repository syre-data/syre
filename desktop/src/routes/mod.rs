//! Route functionalty.
pub mod auth_guard;
pub mod routes;

// Re-exports
pub use routes::Route;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
