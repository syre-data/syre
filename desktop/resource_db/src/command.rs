use std::path::PathBuf;
use syre_core::types::ResourceId;
use tokio::sync::oneshot::Sender as Tx;

#[derive(Debug)]
pub enum Command {
    /// Perform a generic db query.
    Query {
        /// Response channel.
        tx: Tx<surrealdb::Result<surrealdb::Response>>,
        query: String,
    },

    /// Search for resources within the database.
    Search {
        /// Response channel.
        tx: Tx<surrealdb::Result<Vec<ResourceId>>>,
        project: Option<PathBuf>,
        query: String,
    },
}
