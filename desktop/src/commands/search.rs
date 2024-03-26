//! Search commands.
use crate::invoke::invoke_result;
use serde::Serialize;
use syre_core::types::ResourceId;
use syre_local_database::command::search::Query;

pub async fn search(query: String) -> Result<Vec<ResourceId>, String> {
    invoke_result("search", SearchArgs { query }).await
}

pub async fn query(query: Query) -> Result<Vec<ResourceId>, String> {
    invoke_result("query", QueryArgs { query }).await
}

#[derive(Serialize)]
struct SearchArgs {
    pub query: String,
}

#[derive(Serialize)]
struct QueryArgs {
    pub query: Query,
}
