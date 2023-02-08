//! Settings components.
pub mod general;
pub mod settings;

// Re-exports
pub use settings::Settings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
