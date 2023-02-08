//! Database client.
pub mod client;

// Re-exports
pub use client::Client;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
