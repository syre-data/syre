//! Local runner for Syre projects.
use crate::hooks;
use syre_core::runner::{Runner as CoreRunner, RunnerHooks};

pub struct Runner();
impl Runner {
    pub fn new() -> CoreRunner {
        let hooks = RunnerHooks::new(hooks::get_script);
        CoreRunner::new(hooks)
    }
}
