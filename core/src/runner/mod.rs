//! Functionality for running Syre projects.
pub mod common;
pub mod env;
pub mod resources;
pub mod runner;

// Re-exports
pub use env::CONTAINER_ID_KEY;
pub use runner::{Runner, RunnerHooks, ScriptExecutionContext};
