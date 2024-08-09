use std::{fs, io, path::PathBuf};
use syre_core::{project::ScriptLang, types::ResourceId};
use syre_desktop_lib::{self as lib, command::error::IoErrorKind};
use syre_local_database::Client as DbClient;

#[tauri::command]
pub fn add_scripts(
    db: tauri::State<DbClient>,
    project: ResourceId,
    resources: Vec<lib::types::AddFsAnalysisResourceData>,
) -> Vec<Result<(), IoErrorKind>> {
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

    resources
        .iter()
        .map(|resource| {
            assert!(resource.path.is_absolute());
            assert!(resource.parent.is_absolute());

            let Some(ext) = resource.path.extension() else {
                return Err(io::ErrorKind::InvalidFilename.into());
            };

            let ext = ext.to_str().unwrap();
            if !ScriptLang::supported_extensions().contains(&ext) {
                return Err(io::ErrorKind::InvalidFilename.into());
            }

            let to = lib::utils::join_path_absolute(&analysis_root, &resource.parent);
            let to = to.join(resource.path.file_name().unwrap());

            match resource.action {
                FsResourceAction::Copy => {
                    fs::copy(&resource.path, to)?;
                    Ok(())
                }
                FsResourceAction::Move => fs::rename(&resource.path, to).map_err(|err| err.into()),
                FsResourceAction::Reference => todo!(),
            }
        })
        .collect()
}
