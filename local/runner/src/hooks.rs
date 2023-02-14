//! Local runner hooks.
use thot_core::error::{ResourceError, Result as CoreResult};
use thot_core::project::Script as CoreScript;
use thot_core::runner::RunnerHooks as CoreRunnerHooks;
use thot_core::types::ResourceId;
use thot_local_database::{Client as DbClient, ScriptCommand};

/// Retrieves a local [`Script`](CoreScript) given its [`ResourceId`].
pub fn get_script(rid: &ResourceId) -> CoreResult<CoreScript> {
    let db = DbClient::new();
    let script = db.send(ScriptCommand::Get(rid.clone()).into());
    let script: Option<CoreScript> =
        serde_json::from_value(script).expect("could not convert result of `Get` to `Script`");

    let Some(script) = script else {
        return Err(ResourceError::DoesNotExist("`Script` not loaded".to_string()).into());
    };

    Ok(script)
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
