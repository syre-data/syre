//! Suspense components.
pub mod fallback_loading;

// Re-exports
pub use fallback_loading::Loading;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
