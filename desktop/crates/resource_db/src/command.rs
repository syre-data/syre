use serde::{Deserialize, Serialize};
use syre_core::types::ResourceId;

#[cfg(any(feature = "server", feature = "client"))]
pub use inner::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    containers: Vec<ResourceId>,
    assets: Vec<ResourceId>,
    container_scores: Vec<f64>,
    asset_scores: Vec<f64>,
}

impl SearchResult {
    pub fn new(
        containers: Vec<ResourceId>,
        assets: Vec<ResourceId>,
        container_scores: Vec<f64>,
        asset_scores: Vec<f64>,
    ) -> Self {
        Self {
            containers,
            assets,
            container_scores,
            asset_scores,
        }
    }

    /// Create a new empty result.
    pub fn empty() -> Self {
        Self {
            containers: vec![],
            assets: vec![],
            container_scores: vec![],
            asset_scores: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.containers.is_empty() && self.assets.is_empty()
    }

    pub fn containers(&self) -> &Vec<ResourceId> {
        &self.containers
    }

    pub fn assets(&self) -> &Vec<ResourceId> {
        &self.assets
    }

    pub fn container_scores(&self) -> &Vec<f64> {
        &self.container_scores
    }

    pub fn asset_scores(&self) -> &Vec<f64> {
        &self.asset_scores
    }
}

#[cfg(any(feature = "server", feature = "client"))]
mod inner {
    use super::SearchResult;
    use std::path::PathBuf;
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
            tx: Tx<surrealdb::Result<SearchResult>>,
            query: String,
        },

        /// Search for project resources within the database.
        SearchProject {
            /// Response channel.
            tx: Tx<surrealdb::Result<SearchResult>>,
            project: PathBuf,
            query: String,
        },
    }
}
