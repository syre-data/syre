//! Local runner hooks.
use crate::system::collections::Scripts;
use settings_manager::SystemSettings;
use thot_core::error::{ResourceError, Result as CoreResult};
use thot_core::project::Script as CoreScript;
use thot_core::runner::RunnerHooks as CoreRunnerHooks;
use thot_core::types::ResourceId;

/// Retrieves a local [`Script`](CoreScript) given its [`ResourceId`].
pub fn get_script(rid: &ResourceId) -> CoreResult<CoreScript> {
    // @todo: Handle error.
    let scripts = Scripts::load().expect("could not load system scripts");

    let Some(script) = scripts.get(rid) else {
        return Err(
            ResourceError::DoesNotExist(format!("`Script` with {} not found", rid)).into(),
        )
    };

    Ok(script.clone().into())
}

pub struct RunnerHooks {}
impl RunnerHooks {
    pub fn new() -> CoreRunnerHooks {
        CoreRunnerHooks::new(get_script)
    }
}

#[cfg(test)]
#[path = "./hooks_test.rs"]
mod hooks_test;
