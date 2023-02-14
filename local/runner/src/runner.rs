//! Local runner for Thot projects.
use crate::hooks;
use thot_core::runner::{Runner as CoreRunner, RunnerHooks};

pub struct Runner();
impl Runner {
    pub fn new() -> CoreRunner {
        let hooks = RunnerHooks::new(hooks::get_script);
        CoreRunner::new(hooks)
    }
}

#[cfg(test)]
#[path = "./runner_test.rs"]
mod runner_test;
