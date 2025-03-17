use rayon::prelude::*;
use std::{assert_matches::assert_matches, fs, io, path::PathBuf};
use syre_core::{project::ScriptLang, types::ResourceId};
use syre_desktop_lib::{self as lib};
use syre_local as local;
use syre_project_watcher as db;

#[tauri::command]
pub async fn project_add_analyses(
    db: tauri::State<'_, db::Client>,
    project: ResourceId,
    mut resources: Vec<lib::types::AddFsAnalysisResourceData>,
) -> Result<(), lib::command::analyses::error::AddAnalyses> {
    use lib::command::analyses::error;
    use syre_local::types::FsResourceAction;

    resources.iter_mut().for_each(|resource| {
        assert!(resource.path.is_absolute());
        assert_matches!(
            resource.parent.components().next().unwrap(),
            std::path::Component::RootDir
        );

        resource.path = fs::canonicalize(&resource.path).unwrap();
    });

    let (project_path, project) = db.project().get_by_id(project).unwrap().unwrap();
    let analysis_root = project_path.join(
        project
            .properties()
            .unwrap()
            .analysis_root
            .as_ref()
            .unwrap(),
    );

    let (in_analysis_folder, not_in_analysis_folder): (Vec<_>, Vec<_>) = resources
        .into_iter()
        .partition(|resource| resource.path.starts_with(&analysis_root));

    let update_analysis_err = if in_analysis_folder.is_empty() {
        None
    } else {
        match local::project::Analyses::load_from(&project_path) {
            Ok(mut analyses) => {
                for resource in in_analysis_folder.iter() {
                    let rel_path = resource.path.strip_prefix(&analysis_root).unwrap();
                    if !analyses.values().any(|analysis| match analysis {
                        local::types::AnalysisKind::Script(script) => script.path == rel_path,
                        local::types::AnalysisKind::ExcelTemplate(template) => {
                            template.template.path == rel_path
                        }
                    }) {
                        let script = syre_core::project::Script::from_path(rel_path).unwrap();
                        analyses.insert_script_unique_path(script).unwrap();
                    }
                }

                analyses
                    .save()
                    .map_err(|err| error::UpdateAnalyses {
                        error: err.into(),
                        resources: in_analysis_folder
                            .into_iter()
                            .map(|resource| resource.path)
                            .collect(),
                    })
                    .err()
            }

            Err(err) => Some(error::UpdateAnalyses {
                error: err,
                resources: in_analysis_folder
                    .into_iter()
                    .map(|resource| resource.path)
                    .collect(),
            }),
        }
    };

    let (fs_resources, fs_resource_errors): (Vec<_>, Vec<_>) = not_in_analysis_folder
        .into_iter()
        .map(|resource| {
            let Some(ext) = resource.path.extension() else {
                return Err(error::FsResource {
                    path: resource.path,
                    error: io::ErrorKind::InvalidFilename.into(),
                });
            };

            let ext = ext.to_str().unwrap();
            if !ScriptLang::supported_extensions().contains(&ext) {
                return Err(error::FsResource {
                    path: resource.path,
                    error: io::ErrorKind::InvalidFilename.into(),
                });
            }

            Ok(resource)
        })
        .partition(|resource| resource.is_ok());
    let fs_resource_errors = fs_resource_errors.into_iter().map(|err| err.unwrap_err());
    let fs_resources = fs_resources.into_iter().map(|resource| resource.unwrap());

    let mut fs_results = tokio::task::JoinSet::new();
    for resource in fs_resources {
        let to = local::common::join_path_absolute(&analysis_root, &resource.parent);
        let to = to.join(resource.path.file_name().unwrap());
        let to = match local::common::unique_file_name(&to) {
            Ok(path) => path,
            Err(_) => to,
        };

        let resource_path = fs::canonicalize(resource.path).unwrap();
        assert_ne!(resource_path, to);

        match resource.action {
            FsResourceAction::Copy => {
                fs_results.spawn(async move {
                    tokio::fs::copy(&resource_path, to)
                        .await
                        .map(|_| ())
                        .map_err(|err| error::FsResource {
                            path: resource_path,
                            error: err.into(),
                        })
                });
            }
            FsResourceAction::Move => {
                fs_results.spawn(async move {
                    tokio::fs::rename(&resource_path, to)
                        .await
                        .map_err(|err| error::FsResource {
                            path: resource_path,
                            error: err.into(),
                        })
                });
            }
            FsResourceAction::Reference => todo!(),
        }
    }

    let fs_results = fs_results.join_all().await;
    let fs_errors = fs_results
        .into_iter()
        .filter_map(|result| result.err())
        .chain(fs_resource_errors)
        .collect::<Vec<_>>();

    if update_analysis_err.is_none() && fs_errors.is_empty() {
        Ok(())
    } else {
        Err(error::AddAnalyses {
            update_analyses: update_analysis_err,
            fs: fs_errors,
        })
    }
}

/// Sets all associations within `root`'s subtree with `analysis` to `enable`.
#[tauri::command]
pub async fn analysis_toggle_associations(
    db: tauri::State<'_, db::Client>,
    project: PathBuf,
    root: PathBuf,
    analysis: ResourceId,
    enable: bool,
) -> Result<(), lib::command::analyses::error::ToggleSubtreeAssociations> {
    use lib::command::analyses::error::ToggleSubtreeAssociations;

    let Some(project) = db.project().get(project).unwrap() else {
        return Err(ToggleSubtreeAssociations::ProjectNotFound);
    };
    let db::state::FolderResource::Present(project_data) = project.fs_resource().as_ref() else {
        return Err(ToggleSubtreeAssociations::ProjectNotPresent);
    };
    let project_properties = match project_data.properties() {
        db::state::DataResource::Ok(properties) => properties,
        db::state::DataResource::Err(err) => {
            return Err(ToggleSubtreeAssociations::InvalidProject(err));
        }
    };

    let data_root = project.path().join(&project_properties.data_root);
    let subtree_root = db::common::container_system_path(data_root, root);

    let containers = local::common::ignore::WalkBuilder::new(subtree_root)
        .build()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .collect::<Vec<_>>();

    let errors = containers
        .into_par_iter()
        .filter_map(|entry| {
            let mut container =
                match local::loader::container::Loader::load_from_only_properties(entry.path())
                    .map_err(|err| (entry.path().to_path_buf(), err))
                {
                    Ok(container) => container,
                    Err(err) => return Some(err),
                };

            let Some(association) = container
                .analyses
                .iter_mut()
                .find(|association| association.analysis() == &analysis)
            else {
                return None;
            };

            if association.autorun != enable {
                association.autorun = enable;
                container
                    .save(entry.path())
                    .map_err(|err| (entry.path().to_path_buf(), err.into()))
                    .err()
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ToggleSubtreeAssociations::Container(errors))
    }
}
