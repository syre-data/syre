//! Commands related to `Script`s.
use crate::error::Result;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::Script;
use thot_core::types::ResourceId;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::ScriptCommand;
use thot_local_database::Result as DbResult;

// ***********************
// *** project scripts ***
// ***********************

#[tauri::command]
pub fn get_project_scripts(db: State<DbClient>, rid: ResourceId) -> Result<Vec<Script>> {
    let scripts = db.send(ScriptCommand::LoadProject(rid).into());
    let scripts: DbResult<Vec<Script>> = serde_json::from_value(scripts)
        .expect("could not convert `LoadProject` result to `Scripts`");

    Ok(scripts?)
}

// ******************
// *** add script ***
// ******************

#[tauri::command]
pub fn add_script(db: State<DbClient>, project: ResourceId, path: PathBuf) -> Result<Script> {
    let script = db.send(ScriptCommand::Add(project, path).into());
    let script: DbResult<Script> =
        serde_json::from_value(script).expect("could not convert `AddScript` result to `Script`");

    Ok(script?)
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
