use std::{fs, path::PathBuf};
use syre_core::{
    project::Project,
    types::{ResourceId, UserId, UserPermissions},
};
use syre_local::project::{
    project,
    resources::{Container as LocalContainer, Project as LocalProject},
};
use syre_local_database as db;

#[tauri::command]
pub fn create_project(user: ResourceId, path: PathBuf) -> syre_local::Result<Project> {
    project::init(&path)?;

    // create analysis folder
    let analysis_root = "analysis";
    let mut analysis = path.to_path_buf();
    analysis.push(analysis_root);
    fs::create_dir(&analysis).unwrap();

    let mut project = LocalProject::load_from(path)?;
    let settings = project.settings_mut();
    settings.creator = Some(UserId::Id(user.clone()));
    settings
        .permissions
        .insert(user.clone(), UserPermissions::all());
    project.analysis_root = Some(PathBuf::from(analysis_root));
    project.save()?;

    let mut root = LocalContainer::new(project.data_root_path());
    root.settings_mut().creator = Some(UserId::Id(user.clone()));
    root.save()?;

    Ok(project.into())
}

/// # Returns
/// Tuple of (project path, project data, project graph).
#[tauri::command]
pub fn project_resources(
    db: tauri::State<db::Client>,
    project: ResourceId,
) -> Option<(
    PathBuf,
    db::state::ProjectData,
    db::state::FolderResource<db::state::Graph>,
)> {
    let resources = db.project().resources(project).unwrap();
    assert!(if let Some((_, data, _)) = resources.as_ref() {
        data.properties().is_ok()
    } else {
        true
    });

    resources
}
