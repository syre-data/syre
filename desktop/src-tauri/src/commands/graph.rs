use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_local::project::container;
use syre_local_database as db;

#[tauri::command]
pub fn create_child_container(
    db: tauri::State<db::Client>,
    project: ResourceId,
    path: PathBuf,
) -> Result<ResourceId, container::error::Build> {
    assert!(path.is_absolute());
    let (project_path, project_state) = db.project().get_by_id(project).unwrap().unwrap();
    let db::state::DataResource::Ok(properties) = project_state.properties() else {
        panic!("invalid state");
    };

    let container_path =
        db::common::container_system_path(project_path.join(&properties.data_root), path);
    container::new(container_path).map_err(|err| match err {
        container::error::Build::Load | container::error::Build::NotADirectory => {
            unreachable!("should not occure when creating a new container");
        }
        container::error::Build::Save(_) | container::error::Build::AlreadyResource => err,
    })
}
