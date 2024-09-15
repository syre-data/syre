use std::{
    fs, io,
    path::{Path, PathBuf},
};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local as local;
use syre_local_database::{self as db, common::is_root_path};

#[tauri::command]
pub fn create_child_container(
    db: tauri::State<db::Client>,
    project: ResourceId,
    path: PathBuf,
) -> Result<ResourceId, local::project::container::error::Build> {
    assert!(is_root_path(&path));
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
) -> Vec<Result<(), Vec<(PathBuf, lib::command::error::IoErrorKind)>>> {
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
                        return Err(vec![(resource.path.clone(), io::ErrorKind::AlreadyExists)]);
                    }

                    fs::rename(&resource.path, &resource.parent)
                        .map_err(|err| vec![(resource.path.clone(), err.kind())])
                }
                FsResourceAction::Copy => {
                    if to_path == resource.path {
                        return Err(vec![(resource.path.clone(), io::ErrorKind::AlreadyExists)]);
                    }

                    let to_name = local::common::unique_file_name(&to_path)
                        .map_err(|err| vec![(resource.path.clone(), err)])?;
                    let to_path = resource.parent.join(to_name);
                    if resource.path.is_file() {
                        fs::copy(&resource.path, &to_path)
                            .map(|_| ())
                            .map_err(|err| vec![(resource.path.clone(), err.kind())])

                        // TODO: Set creator. What if already a resource and current creator differs from original?
                        // TODO: If file is already a resource, copy info.
                    } else if resource.path.is_dir() {
                        copy_dir(&resource.path, &to_path)
                    } else {
                        todo!();
                    }
                }
                FsResourceAction::Reference => todo!(),
            }
        })
        .map(|result| {
            result.map_err(|errors| {
                errors
                    .into_iter()
                    .map(|(path, err)| (path, err.into()))
                    .collect()
            })
        })
        .collect()
}

pub fn copy_dir(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), Vec<(PathBuf, io::ErrorKind)>> {
    let src: &Path = src.as_ref();
    let dst: &Path = dst.as_ref();
    let results = walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let rel_path = entry.path().strip_prefix(src).unwrap();
            let dst = dst.join(rel_path);

            if entry.file_type().is_file() {
                match fs::copy(entry.path(), dst) {
                    Ok(_) => Ok(()),
                    Err(err) => Err((entry.path().to_path_buf(), err.kind())),
                }
            } else if entry.file_type().is_dir() {
                match fs::create_dir(dst) {
                    Ok(_) => Ok(()),
                    Err(err) => Err((entry.path().to_path_buf(), err.kind())),
                }
            } else {
                todo!();
            }
        })
        .collect::<Vec<_>>();

    let errors = results
        .into_iter()
        .filter_map(|result| match result {
            Ok(_) => None,
            Err(err) => Some(err),
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
