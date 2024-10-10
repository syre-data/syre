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

// TODO: Clean up return type.
/// Adds file system resources to the project.
///
/// # Returns
/// `Vec` of `Result`s corresponding to each resource.
#[tauri::command]
pub async fn add_file_system_resources(
    db: tauri::State<'_, db::Client>,
    resources: Vec<lib::types::AddFsGraphResourceData>,
) -> Result<(), Vec<(PathBuf, lib::command::error::IoErrorKind)>> {
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

#[tauri::command]
pub async fn container_duplicate(
    db: tauri::State<'_, db::Client>,
    project: ResourceId,
    container: PathBuf,
) -> Result<(), Vec<(PathBuf, lib::command::error::IoErrorKind)>> {
    assert!(is_root_path(&container));
    let (project_path, project_state) = db.project().get_by_id(project).unwrap().unwrap();
    let db::state::DataResource::Ok(properties) = project_state.properties() else {
        panic!("invalid state");
    };

    let root_path =
        db::common::container_system_path(project_path.join(&properties.data_root), &container);

    let name = local::common::unique_file_name(&root_path).unwrap();
    let mut dup_path = root_path.clone();
    dup_path.set_file_name(name);
    duplicate_subgraph(root_path, dup_path).map_err(|errors| {
        errors
            .into_iter()
            .map(|(path, err)| (path, err.into()))
            .collect()
    })
}

#[tauri::command]
pub fn container_trash(
    db: tauri::State<db::Client>,
    project: ResourceId,
    container: PathBuf,
) -> Result<(), lib::command::error::IoErrorKind> {
    assert!(is_root_path(&container));
    let (project_path, project_state) = db.project().get_by_id(project).unwrap().unwrap();
    let db::state::DataResource::Ok(properties) = project_state.properties() else {
        panic!("invalid state");
    };

    let container_path =
        db::common::container_system_path(project_path.join(&properties.data_root), container);

    trash::delete(container_path).map_err(|err| match err {
        _ => todo!("{err:?}"),
    })
}

/// # Returns
/// `Err` if any path fails to be copied.
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

/// Duplicates a subgraph.
/// Removes all assets from containers.
///
/// # Returns
/// `Err` if any path fails to be copied.
pub fn duplicate_subgraph(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), Vec<(PathBuf, io::ErrorKind)>> {
    let src: &Path = src.as_ref();
    let dst: &Path = dst.as_ref();
    let Ok(graph) = local::loader::tree::Loader::load(src) else {
        todo!();
    };

    if let Err(err) = local::graph::ContainerTreeDuplicator::duplicate_without_assets_to(
        dst,
        &graph,
        graph.root(),
    ) {
        todo!("{err:?}");
    }

    let mut root = local::loader::container::Loader::load_from_only_properties(dst).unwrap();
    root.properties.name = dst.file_name().unwrap().to_string_lossy().to_string();
    if let Err(err) = root.save(dst) {
        todo!("{err:?}");
    }

    Ok(())
}
