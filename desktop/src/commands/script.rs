//! Script commands.
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::project::{ExcelTemplate, Script};
use syre_core::types::ResourceId;
use syre_local::types::ScriptStore;

pub async fn get_project_scripts(project: ResourceId) -> Result<ScriptStore, String> {
    invoke_result("get_project_scripts", ResourceIdArgs { rid: project }).await
}

pub async fn add_script(project: ResourceId, path: PathBuf) -> Result<Option<Script>, String> {
    invoke_result("add_script", AddScriptArgs { project, path }).await
}

pub async fn add_script_windows(
    project: ResourceId,
    file_name: PathBuf,
    contents: Vec<u8>,
) -> Result<(), String> {
    invoke_result(
        "add_script_windows",
        AddScriptWindowsArgs {
            project,
            file_name,
            contents,
        },
    )
    .await
}

/// # Returns
/// Final path of the template.
pub async fn add_excel_template(
    project: ResourceId,
    template: ExcelTemplate,
) -> Result<PathBuf, String> {
    // TODO Issue with serializing `HashMap` of `metadata`. perform manually.
    // See https://github.com/tauri-apps/tauri/issues/6078
    let template = serde_json::to_string(&template).unwrap();
    invoke_result(
        "add_excel_template",
        AddExcelTemplateArgs { project, template },
    )
    .await
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
#[serde(rename_all = "camelCase")]
pub struct AddScriptWindowsArgs {
    pub project: ResourceId,
    pub file_name: PathBuf,
    pub contents: Vec<u8>,
}

#[derive(Serialize)]
pub struct RemoveScriptArgs {
    pub project: ResourceId,
    pub script: ResourceId,
}

#[derive(Serialize)]
struct AddExcelTemplateArgs {
    project: ResourceId,
    template: String,
    // template: ExcelTemplate,
}
