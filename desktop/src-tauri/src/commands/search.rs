//! Search functionality.
use syre_core::types::ResourceId;
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::search::{Query, SearchCommand};
use tauri::State;

#[tauri::command]
pub fn search(db: State<DbClient>, query: String) -> Result<Vec<ResourceId>, String> {
    let res = db.send(SearchCommand::Search(query).into()).unwrap();
    let res = serde_json::from_value(res).unwrap();
    Ok(res)
}

#[tauri::command]
pub fn query(db: State<DbClient>, query: Query) -> Result<Vec<ResourceId>, String> {
    let res = db.send(SearchCommand::Query(query).into()).unwrap();
    let res = serde_json::from_value(res).unwrap();
    Ok(res)
}
