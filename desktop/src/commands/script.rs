//! Script commands.
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use thot_core::project::Script;
use thot_core::types::ResourceId;

pub async fn get_project_scripts(project: ResourceId) -> Result<Vec<Script>, String> {
    invoke_result("get_project_scripts", ResourceIdArgs { rid: project }).await
}

pub async fn add_script(project: ResourceId, path: PathBuf) -> Result<Script, String> {
    invoke_result("add_script", AddScriptArgs { project, path }).await
}

pub async fn remove_script(project: ResourceId, script: ResourceId) -> Result<(), String> {
    invoke_result("remove_script", RemoveScriptArgs { project, script }).await
}

#[derive(Serialize)]
pub struct AddScriptArgs {
    pub project: ResourceId,
    pub path: PathBuf,
}

#[derive(Serialize)]
pub struct RemoveScriptArgs {
    pub project: ResourceId,
    pub script: ResourceId,
}
