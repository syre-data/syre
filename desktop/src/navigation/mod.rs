//! Navigation resources and functionality.
pub mod main;

// Re-exports
pub use main::MainNavigation;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
