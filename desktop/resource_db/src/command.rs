use syre_core::types::ResourceId;
use tokio::sync::oneshot::Sender as Tx;

#[derive(Debug)]
pub enum Command {
    Query {
        /// Response channel.
        tx: Tx<surrealdb::Result<surrealdb::Response>>,
        query: String,
    },

    Search {
        /// Response channel.
        tx: Tx<surrealdb::Result<Vec<ResourceId>>>,
        query: String,
    },
}
