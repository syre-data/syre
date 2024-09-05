use std::{fs, io, path::PathBuf};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local as local;
use syre_local_database as db;

#[tauri::command]
pub fn create_child_container(
    db: tauri::State<db::Client>,
    project: ResourceId,
    path: PathBuf,
) -> Result<ResourceId, local::project::container::error::Build> {
    assert!(path.is_absolute());
    let (project_path, project_state) = db.project().get_by_id(project).unwrap().unwrap();
    let db::state::DataResource::Ok(properties) = project_state.properties() else {
        panic!("invalid state");
    };

    let container_path =
        db::common::container_system_path(project_path.join(&properties.data_root), path);
    local::project::container::new(container_path).map_err(|err| match err {
        local::project::container::error::Build::Load
        | local::project::container::error::Build::NotADirectory => {
            unreachable!("should not occure when creating a new container");
        }
        local::project::container::error::Build::Save(_)
        | local::project::container::error::Build::AlreadyResource => err,
    })
}

// TODO: Change return `Err` kind to `io::ErrorKind`
// but needs serialization.
/// Adds file system resources to the project.
///
/// # Returns
/// `Vec` of `Result`s corresponding to each resource.
#[tauri::command]
pub fn add_file_system_resources(
    db: tauri::State<db::Client>,
    resources: Vec<lib::types::AddFsGraphResourceData>,
) -> Vec<Result<(), lib::command::error::IoErrorKind>> {
    use syre_local::types::FsResourceAction;

    let mut projects = resources
        .iter()
        .map(|resource| &resource.project)
        .collect::<Vec<_>>();
    projects.sort();
    projects.dedup();

    let project_paths = projects
        .into_iter()
        .cloned()
        .map(|project| {
            let (path, state) = db.project().get_by_id(project.clone()).unwrap().unwrap();
            let db::state::DataResource::Ok(properties) = state.properties() else {
                todo!();
            };

            (project, path.join(&properties.data_root))
        })
        .collect::<Vec<_>>();

    resources
        .into_iter()
        .map(|resource| {
            let project_path = project_paths
                .iter()
                .find_map(|(project, path)| {
                    if *project == resource.project {
                        Some(path)
                    } else {
                        None
                    }
                })
                .unwrap();

            let to_path = lib::utils::join_path_absolute(project_path, &resource.parent);
            let to_path = to_path.join(resource.path.file_name().unwrap());
            match resource.action {
                FsResourceAction::Move => {
                    if to_path == resource.path {
                        return Err(io::ErrorKind::AlreadyExists.into());
                    }

                    fs::rename(&resource.path, &resource.parent).map_err(|err| err.kind().into())
                }
                FsResourceAction::Copy => {
                    if to_path == resource.path {
                        return Err(io::ErrorKind::AlreadyExists.into());
                    }

                    let to_name = local::common::unique_file_name(&to_path)?;
                    let to_path = resource.parent.join(to_name);
                    if resource.path.is_file() {
                        fs::copy(&resource.path, &to_path)
                            .map(|_| ())
                            .map_err(|err| err.kind().into())

                        // TODO: Set creator.
                        // TODO: If file is already a resource, copy info.
                    } else if resource.path.is_dir() {
                        todo!();
                    } else {
                        todo!();
                    }
                }
                FsResourceAction::Reference => todo!(),
            }
        })
        .collect()
}
