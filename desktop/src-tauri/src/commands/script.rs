//! Commands related to `Script`s.
use crate::error::Result;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::{fs, io};
use syre_core::error::{Error as CoreError, Project as ProjectError, Resource as ResourceError};
use syre_core::project::{Project, Script};
use syre_core::types::ResourceId;
use syre_desktop_lib::excel_template;
use syre_local::common;
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::{ProjectCommand, ScriptCommand};
use syre_local_database::Result as DbResult;
use tauri::State;

// ***********************
// *** project scripts ***
// ***********************

#[tauri::command]
pub fn get_project_scripts(db: State<DbClient>, rid: ResourceId) -> Result<Vec<Script>> {
    let scripts = db.send(ScriptCommand::LoadProject(rid).into()).unwrap();
    let scripts: DbResult<Vec<Script>> = serde_json::from_value(scripts).unwrap();
    Ok(scripts?)
}

// ******************
// *** add script ***
// ******************

// TODO May not be used any more. Can possible remove.
/// Adds a Script to the Project.
///
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
            .send(ScriptCommand::Add(project.rid.clone(), file_name).into())
            .unwrap();

        let script: DbResult<Script> = serde_json::from_value(script).unwrap();
        Ok(Some(script?))
    }
}

// TODO: Check if file contents matches that of a file already in the analysis folder
// If so, don't need to write the contents to disk, just use existing file.
#[tauri::command]
pub fn add_script_windows(
    db: State<DbClient>,
    project: ResourceId,
    file_name: PathBuf,
    contents: Vec<u8>,
) -> Result {
    let project = get_project(&db, project)?;
    let project_path = get_project_path(&db, project.rid.clone())?;

    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::Project(ProjectError::misconfigured(
            "`Project` does not have an analysis root set",
        ))
        .into());
    };

    let mut to_path = project_path;
    to_path.push(analysis_root);
    to_path.push(file_name);
    let to_path = common::unique_file_name(to_path)?;

    fs::write(&to_path, contents)?;
    Ok(())
}

// **************************
// *** add excel template ***
// **************************

/// Add an excel template as a script.
#[tauri::command]
pub fn add_excel_template(
    db: State<DbClient>,
    project: ResourceId,
    template: excel_template::ExcelTemplate,
) -> Result<Script> {
    // copy script to analysis root
    let project = get_project(&db, project)?;
    let project_path = get_project_path(&db, project.rid.clone())?;
    let Some(analysis_root) = project.analysis_root.clone() else {
        return Err(CoreError::Project(ProjectError::misconfigured(
            "`Project` does not have an analysis root set",
        ))
        .into());
    };

    let path = template.template_params.path.clone();
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
            .send(ScriptCommand::Add(project.rid.clone(), file_name).into())
            .unwrap();

        let script: DbResult<Script> = serde_json::from_value(script).unwrap();
        Ok(Some(script?))
    }
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

fn get_project(db: &State<DbClient>, project: ResourceId) -> Result<Project> {
    let project = db
        .send(ProjectCommand::Get(project.clone()).into())
        .expect("could not get `Project`");

    let project: Option<Project> =
        serde_json::from_value(project).expect("could not convert `Get` result to `Project`");

    let Some(project) = project else {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`Project` not loaded")).into(),
        );
    };

    Ok(project)
}

fn get_project_path(db: &State<DbClient>, project: ResourceId) -> Result<PathBuf> {
    let project_path = db
        .send(ProjectCommand::GetPath(project).into())
        .expect("could not get `Project` path");

    let project_path: Option<PathBuf> =
        serde_json::from_value(project_path).expect("could not convert `GetPath` to `PathBuf`");

    let Some(project_path) = project_path else {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`Project` not loaded")).into(),
        );
    };

    Ok(project_path)
}
