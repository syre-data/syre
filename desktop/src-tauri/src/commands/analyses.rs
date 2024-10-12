use std::{assert_matches::assert_matches, fs, io, path::PathBuf};
use syre_core::{project::ScriptLang, types::ResourceId};
use syre_desktop_lib::{self as lib, command::error::IoErrorKind};
use syre_local as local;
use syre_local_database as db;

#[tauri::command]
pub async fn project_add_analyses(
    db: tauri::State<'_, db::Client>,
    project: ResourceId,
    resources: Vec<lib::types::AddFsAnalysisResourceData>,
) -> Result<(), Vec<lib::command::analyses::error::AddAnalyses>> {
    use lib::command::analyses::error::AddAnalyses as Error;
    use syre_local::types::FsResourceAction;

    let (project_path, project) = db.project().get_by_id(project).unwrap().unwrap();
    let analysis_root = project_path.clone().join(
        project
            .properties()
            .unwrap()
            .analysis_root
            .as_ref()
            .unwrap(),
    );

    let mut results = tokio::task::JoinSet::new();
    for resource in resources {
        assert!(resource.path.is_absolute());
        assert_matches!(
            resource.parent.components().next().unwrap(),
            std::path::Component::RootDir
        );

        let to = lib::utils::join_path_absolute(&analysis_root, &resource.parent);
        let to = to.join(resource.path.file_name().unwrap());

        let project_path = project_path.clone();
        let analysis_root = analysis_root.clone();
        results.spawn(async move {
            let Some(ext) = resource.path.extension() else {
                return Err(Error::FsResource {
                    path: resource.path.clone(),
                    error: io::ErrorKind::InvalidFilename.into(),
                });
            };

            let ext = ext.to_str().unwrap();
            if !ScriptLang::supported_extensions().contains(&ext) {
                return Err(Error::FsResource {
                    path: resource.path.clone(),
                    error: io::ErrorKind::InvalidFilename.into(),
                });
            }

            match resource.action {
                FsResourceAction::Copy => {
                    let resource_path = fs::canonicalize(resource.path).unwrap();
                    if resource_path != to {
                        tokio::fs::copy(&resource_path, to).await.map_err(|err| {
                            Error::FsResource {
                                path: resource_path.clone(),
                                error: err.into(),
                            }
                        })?;
                    } else {
                        let mut analyses =
                            local::project::resources::Analyses::load_from(&project_path)
                                .map_err(|err| Error::UpdateAnalyses(err))?;

                        let rel_path = to.strip_prefix(analysis_root).unwrap();
                        if !analyses.values().any(|analysis| match analysis {
                            local::types::AnalysisKind::Script(script) => script.path == rel_path,
                            local::types::AnalysisKind::ExcelTemplate(template) => {
                                template.template.path == rel_path
                            }
                        }) {
                            let script = syre_core::project::Script::from_path(rel_path).unwrap();
                            analyses.insert_script_unique_path(script).unwrap();
                            analyses
                                .save()
                                .map_err(|err| Error::UpdateAnalyses(err.into()))?;
                        }
                    }

                    Ok(())
                }
                FsResourceAction::Move => {
                    fs::rename(&resource.path, to).map_err(|err| Error::FsResource {
                        path: resource.path.clone(),
                        error: err.into(),
                    })
                }
                FsResourceAction::Reference => todo!(),
            }
        });
    }

    let results = results.join_all().await;
    let errors = results
        .into_iter()
        .filter_map(|result| result.err())
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
