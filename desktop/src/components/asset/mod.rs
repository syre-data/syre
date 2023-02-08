//! [`Asset`](thot_core::project::Asset) functionality.
pub mod create_assets;

// Re-export
pub use create_assets::CreateAssets;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
