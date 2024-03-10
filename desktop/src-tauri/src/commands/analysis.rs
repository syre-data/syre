//! Commands related to `Script`s.
use crate::error::Result;
use std::path::PathBuf;
use std::{fs, io};
use syre_core::error::{Error as CoreError, Project as ProjectError, Resource as ResourceError};
use syre_core::project::{ExcelTemplate, Project, Script};
use syre_core::types::ResourceId;
use syre_local::common;
use syre_local::types::AnalysisStore;
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::{AnalysisCommand, ProjectCommand};
use syre_local_database::Result as DbResult;
use tauri::State;

#[tauri::command]
pub fn get_project_analyses(db: State<DbClient>, rid: ResourceId) -> Result<AnalysisStore> {
    let analyses = db.send(AnalysisCommand::LoadProject(rid).into()).unwrap();
    let analyses: DbResult<AnalysisStore> = serde_json::from_value(analyses)?;
    Ok(analyses?)
}

/// # Returns
/// + `None` if the Script was not already in the Project's analysis path,
///     so was copied in and the file system watcher will handle it.
/// + `Some(Script)` if the Script was already in the Project's analysis path,
///     so was only added to the Project with no file system interaction.
#[tauri::command]
pub fn add_script(
    db: State<DbClient>,
    project: ResourceId,
    path: PathBuf,
) -> Result<Option<Script>> {
    // copy script to analysis root
    let project = get_project(&db, project)?;
    let project_path = get_project_path(&db, project.rid.clone())?;
    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::Project(ProjectError::misconfigured(
            "`Project` does not have an analysis root set",
        ))
        .into());
    };

    let Some(file_name) = path.file_name() else {
        return Err(
            io::Error::new(io::ErrorKind::InvalidFilename, "could not get file name").into(),
        );
    };

    let file_name = PathBuf::from(file_name);

    let mut to_path = project_path;
    to_path.push(analysis_root);
    to_path.push(file_name.clone());

    let from_path = fs::canonicalize(path)?;
    if to_path != from_path {
        fs::copy(&from_path, to_path)?;
        Ok(None)
    } else {
        // add script to project
        let script = db
            .send(AnalysisCommand::AddScript(project.rid.clone(), file_name).into())
            .unwrap();

        let script: DbResult<Script> = serde_json::from_value(script).unwrap();
        Ok(Some(script?))
    }
}

/// # Returns
/// Final path to file relative to project's analysis root.
#[tauri::command]
pub fn copy_contents_to_analyses(
    db: State<DbClient>,
    project: ResourceId,
    file_name: PathBuf,
    contents: Vec<u8>,
) -> Result<PathBuf> {
    let project = get_project(&db, project)?;
    let project_path = get_project_path(&db, project.rid.clone())?;

    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::Project(ProjectError::misconfigured(
            "`Project` does not have an analysis root set",
        ))
        .into());
    };

    let analysis_path = project_path.join(analysis_root);
    let to_path = analysis_path.join(file_name);
    let to_path = common::unique_file_name(to_path)?;

    fs::write(&to_path, contents)?;
    let final_path = to_path.strip_prefix(analysis_path).unwrap();
    Ok(final_path.to_path_buf())
}

/// Add an excel template.
///
/// # Returns
/// Final path of the template.
#[tauri::command]
pub fn add_excel_template(
    db: State<DbClient>,
    project: ResourceId,
    template: String,
    // mut template: ExcelTemplate,
) -> Result<PathBuf> {
    // TODO Issue with serializing `HashMap` of `metadata`. perform manually.
    // See https://github.com/tauri-apps/tauri/issues/6078
    let mut template: ExcelTemplate = serde_json::from_str(&template).unwrap();

    // copy script to analysis root
    let project = get_project(&db, project)?;
    let project_path = get_project_path(&db, project.rid.clone())?;
    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::Project(ProjectError::misconfigured(
            "`Project` does not have an analysis root set",
        ))
        .into());
    };

    let path = template.template.path.clone();
    let Some(file_name) = path.file_name() else {
        return Err(
            io::Error::new(io::ErrorKind::InvalidFilename, "could not get file name").into(),
        );
    };

    let file_name = PathBuf::from(file_name);
    let analysis_path = project_path.join(analysis_root.clone());
    let mut to_path = analysis_path.join(&file_name);

    let from_path = fs::canonicalize(path)?;
    if to_path != from_path {
        to_path = common::unique_file_name(to_path)?;
        fs::copy(&from_path, &to_path)?;
    }

    let template_path = to_path.strip_prefix(&analysis_path).unwrap().to_path_buf();
    template.template.path = template_path.clone();

    // add template to project
    let res = db
        .send(
            AnalysisCommand::AddExcelTemplate {
                project: project.rid.clone(),
                template,
            }
            .into(),
        )
        .unwrap();

    let res: DbResult = serde_json::from_value(res).unwrap();
    res?;
    Ok(template_path)
}

#[tauri::command]
pub fn update_excel_template(
    db: State<DbClient>,
    template: String, /*ExcelTemplate*/
) -> Result {
    // TODO Issue with serializing `HashMap` of `metadata`. perform manually.
    // See https://github.com/tauri-apps/tauri/issues/6078
    let template: ExcelTemplate = serde_json::from_str(&template).unwrap();
    let res = db
        .send(AnalysisCommand::UpdateExcelTemplate(template).into())
        .unwrap();

    let res: DbResult = serde_json::from_value(res).unwrap();
    Ok(res?)
}

// TODO: Let file system watcher take care of removal.
//      Must be careful of removing file for excel templates if multiple templates based on same file.
#[tauri::command]
pub fn remove_analysis(db: State<DbClient>, project: ResourceId, script: ResourceId) -> Result {
    let res = db
        .send(AnalysisCommand::Remove { project, script }.into())
        .unwrap();

    let res: DbResult = serde_json::from_value(res).unwrap();
    Ok(res?)
}

fn get_project(db: &State<DbClient>, project: ResourceId) -> Result<Project> {
    let project = db
        .send(ProjectCommand::Get(project.clone()).into())
        .unwrap();

    let project: Option<Project> = serde_json::from_value(project).unwrap();

    let Some(project) = project else {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`Project` not loaded")).into(),
        );
    };

    Ok(project)
}

fn get_project_path(db: &State<DbClient>, project: ResourceId) -> Result<PathBuf> {
    let project_path = db.send(ProjectCommand::GetPath(project).into()).unwrap();
    let project_path: Option<PathBuf> = serde_json::from_value(project_path).unwrap();
    let Some(project_path) = project_path else {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`Project` not loaded")).into(),
        );
    };

    Ok(project_path)
}
