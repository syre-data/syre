//! Commands related to `Script`s.
use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use thot_core::error::{Error as CoreError, ProjectError, ResourceError};
use thot_core::project::{Project, Script};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::{Error as LibError, Result as LibResult};
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::{ProjectCommand, ScriptCommand};
use thot_local_database::Result as DbResult;

// ***********************
// *** project scripts ***
// ***********************

#[tauri::command]
pub fn get_project_scripts(db: State<DbClient>, rid: ResourceId) -> LibResult<Vec<Script>> {
    let scripts = db
        .send(ScriptCommand::LoadProject(rid).into())
        .expect("could not load `Project` `Script`s");

    let scripts: DbResult<Vec<Script>> = serde_json::from_value(scripts)
        .expect("could not convert `LoadProject` result to `Scripts`");

    Ok(scripts.map_err(|err| LibError::Database(format!("{err:?}")))?)
}

// ******************
// *** add script ***
// ******************

#[tauri::command]
pub fn add_script(db: State<DbClient>, project: ResourceId, path: PathBuf) -> Result<Script> {
    // copy script to analysis root
    let project = db
        .send(ProjectCommand::Get(project.clone()).into())
        .expect("could not get `Project`");

    let project: Option<Project> =
        serde_json::from_value(project).expect("could not convert `Get` result to `Project`");

    let Some(project) = project else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` not loaded")).into());
    };

    let project_path = db
        .send(ProjectCommand::GetPath(project.rid.clone()).into())
        .expect("could not get `Project` path");
    let project_path: Option<PathBuf> =
        serde_json::from_value(project_path).expect("could not convert `GetPath` to `PathBuf`");

    let Some(project_path) = project_path else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` not loaded")).into());
    };

    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::ProjectError(ProjectError::Misconfigured("`Project` does not have an analysis root set")).into());
    };

    let script_name = path.file_name().expect("invalid `Script` file");
    let script_name = PathBuf::from(script_name);

    let mut to_path = project_path;
    to_path.push(analysis_root);
    to_path.push(script_name.clone());

    if to_path != path {
        fs::copy(&path, to_path)?;
    }

    // add script to project
    let script = db
        .send(ScriptCommand::Add(project.rid.clone(), script_name).into())
        .expect("could not add `Script`");
    let script: DbResult<Script> =
        serde_json::from_value(script).expect("could not convert `AddScript` result to `Script`");

    Ok(script?)
}

// *********************
// *** remove script ***
// *********************

#[tauri::command]
pub fn remove_script(db: State<DbClient>, project: ResourceId, script: ResourceId) -> Result {
    let res = db
        .send(ScriptCommand::Remove(project, script).into())
        .expect("could not remove `Script`");

    let res: DbResult =
        serde_json::from_value(res).expect("could not convert `RemoveScript` result to `Result`");

    res.expect("error removing `Script`");
    Ok(())
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
