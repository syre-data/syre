//! Local runner hooks.
use std::path::PathBuf;
use thot_core::error::{ResourceError, Result as CoreResult};
use thot_core::project::{Project, Script as CoreScript};
use thot_core::runner::RunnerHooks as CoreRunnerHooks;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local_database::{Client as DbClient, ProjectCommand, ScriptCommand};

/// Retrieves a local [`Script`](CoreScript) given its [`ResourceId`].
pub fn get_script(rid: &ResourceId) -> CoreResult<CoreScript> {
    let db = DbClient::new();
    let script = db.send(ScriptCommand::Get(rid.clone()).into());
    let script: Option<CoreScript> =
        serde_json::from_value(script).expect("could not convert result of `Get` to `Script`");

    let Some(mut script) = script else {
        return Err(ResourceError::DoesNotExist("`Script` not loaded").into());
    };

    // get absolute path to script
    match script.path {
        ResourcePath::Absolute(_) => {}
        ResourcePath::Relative(path) => {
            let project = db.send(ScriptCommand::GetProject(script.rid.clone()).into());
            let project: Option<Project> = serde_json::from_value(project)
                .expect("could not convert `GetProject` result to `ResourceId`");

            let project = project.expect("`Script`'s `Project` does not exist");

            let analysis_root = project
                .analysis_root
                .expect("`Project`'s analysis root not set")
                .clone();

            let project_path = db.send(ProjectCommand::GetPath(project.rid.clone()).into());
            let project_path: Option<PathBuf> = serde_json::from_value(project_path)
                .expect("could not convert result of `GetPath` to `PathBuf`");

            let project_path = project_path.expect("`Project` not loaded");

            let mut abs_path = project_path;
            abs_path.push(analysis_root);
            abs_path.push(path);

            let abs_path = ResourcePath::new(abs_path)?;
            script.path = abs_path;
        }

        ResourcePath::Root(_path, _level) => {
            todo!("root paths for `Script`s");
        }
    }

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
