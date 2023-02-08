//! Commands related to `Script`s.
use crate::error::Result;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::Script as CoreScript;
use thot_core::types::ResourceId;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::ScriptCommand;
use thot_local_database::Result as DbResult;

// ***********************
// *** project scripts ***
// ***********************

#[tauri::command]
pub fn get_project_scripts(db: State<DbClient>, rid: ResourceId) -> Result<Vec<CoreScript>> {
    let scripts = db.send(ScriptCommand::LoadProjectScripts(rid).into());
    let scripts: DbResult<Vec<CoreScript>> = serde_json::from_value(scripts)
        .expect("could not convert `AddScript` result to `CoreScripts`");

    Ok(scripts?)
}

// ******************
// *** add script ***
// ******************

#[tauri::command]
pub fn add_script(db: State<DbClient>, project: ResourceId, path: PathBuf) -> Result<CoreScript> {
    let script = db.send(ScriptCommand::AddScript(project, path).into());
    let script: DbResult<CoreScript> = serde_json::from_value(script)
        .expect("could not convert `AddScript` result to `CoreScript`");

    Ok(script?)
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
