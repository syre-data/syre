use syre_desktop_resource_db as db;
use tauri::Manager;

// TODO: Should not reutrn `Result`, but currenlty gives error otherwise.
// See [https://github.com/tauri-apps/tauri/issues/2533].
#[tauri::command]
pub async fn search_project(
    app: tauri::AppHandle,
    query: String,
    project: std::path::PathBuf,
) -> Result<db::SearchResult, ()> {
    let task = tauri::async_runtime::spawn_blocking({
        move || {
            let db = app.state::<db::Client>();
            db.search_project(query, project).unwrap()
        }
    });

    let result = task.await.unwrap();
    Ok(result)
}
