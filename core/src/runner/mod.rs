//! Functionality for running Syre projects.
pub mod common;
pub mod env;
pub mod runner;

pub use env::{CONTAINER_ID_KEY, PROJECT_ID_KEY};
pub use runner::{AnalysisExecutionContext, Error, Runner, RunnerHooks};

use crate::types::ResourceId;
use has_id::HasId;

pub trait Runnable: HasId<Id = ResourceId> {
    fn command(&self) -> std::process::Command;
}
