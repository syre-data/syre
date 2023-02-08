//! Custom hooks.
pub mod preferred_theme;

// Re-exports
pub use preferred_theme::use_preferred_theme;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
