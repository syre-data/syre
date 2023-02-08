//! States of the application.
pub mod app;

// Re-exports
pub use app::AppState;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
