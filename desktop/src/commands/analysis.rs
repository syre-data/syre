//! Script commands.
use super::common::ResourceIdArgs;
use crate::invoke::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::project::{ExcelTemplate, Script};
use syre_core::types::ResourceId;
use syre_local::types::AnalysisStore;

pub async fn get_project_analyses(project: ResourceId) -> Result<AnalysisStore, String> {
    invoke_result("get_project_analyses", ResourceIdArgs { rid: project }).await
}

pub async fn add_script(project: ResourceId, path: PathBuf) -> Result<Option<Script>, String> {
    invoke_result("add_script", AddScriptArgs { project, path }).await
}

/// # Returns
/// Final path to the file relative to the project's analysis root.
pub async fn copy_contents_to_analyses(
    project: ResourceId,
    file_name: PathBuf,
    contents: Vec<u8>,
) -> Result<PathBuf, String> {
    invoke_result(
        "copy_contents_to_analyses",
        CopyContentsToAnalysesArgs {
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
    // TODO: Issue with serializing enum with Option. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/5993
    let template = serde_json::to_string(&template).unwrap();

    invoke_result(
        "add_excel_template",
        AddExcelTemplateArgs { project, template },
    )
    .await
}

pub async fn update_excel_template(template: ExcelTemplate) -> Result<(), String> {
    // TODO: Issue with serializing enum with Option. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/5993

    let template = serde_json::to_string(&template).unwrap();
    invoke_result(
        "update_excel_template",
        UpdateExcelTemplateArgs { template },
    )
    .await
}

pub async fn remove_analysis(project: ResourceId, script: ResourceId) -> Result<(), String> {
    invoke_result("remove_analysis", RemoveScriptArgs { project, script }).await
}

#[derive(Serialize)]
pub struct AddScriptArgs {
    pub project: ResourceId,
    pub path: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyContentsToAnalysesArgs {
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
    // template: ExcelTemplate, // TODO: Issue with serializing enum with Option. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/5993
}

#[derive(Serialize)]
struct UpdateExcelTemplateArgs {
    template: String,
    // template: ExcelTemplate, // TODO: Issue with serializing enum with Option. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/5993
}
