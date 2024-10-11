use std::{assert_matches::assert_matches, fs, io, path::PathBuf};
use syre_core::{project::ScriptLang, types::ResourceId};
use syre_desktop_lib::{self as lib, command::error::IoErrorKind};
use syre_local_database as db;

#[tauri::command]
pub async fn add_scripts(
    db: tauri::State<'_, db::Client>,
    project: ResourceId,
    resources: Vec<lib::types::AddFsAnalysisResourceData>,
) -> Result<(), Vec<(PathBuf, IoErrorKind)>> {
    use syre_local::types::FsResourceAction;

    let (project_path, project) = db.project().get_by_id(project).unwrap().unwrap();
    let analysis_root = project_path.join(
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

        results.spawn(async move {
            let Some(ext) = resource.path.extension() else {
                return Err((resource.path.clone(), io::ErrorKind::InvalidFilename));
            };

            let ext = ext.to_str().unwrap();
            if !ScriptLang::supported_extensions().contains(&ext) {
                return Err((resource.path.clone(), io::ErrorKind::InvalidFilename));
            }

            match resource.action {
                FsResourceAction::Copy => {
                    tokio::fs::copy(&resource.path, to)
                        .await
                        .map_err(|err| (resource.path.clone(), err.kind()))?;
                    Ok(())
                }
                FsResourceAction::Move => fs::rename(&resource.path, to)
                    .map_err(|err| (resource.path.clone(), err.kind())),
                FsResourceAction::Reference => todo!(),
            }
        });
    }
    let results = results.join_all().await;

    if results.iter().any(|res| res.is_err()) {
        let errors = results
            .into_iter()
            .filter_map(|result| result.err())
            .map(|(path, err)| (path, err.into()))
            .collect();
        Err(errors)
    } else {
        Ok(())
    }
}
