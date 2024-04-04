use super::Command;
use crate::command::search::ResourceKind;
use std::str::FromStr;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{error, Error, Surreal};
use syre_core::types::ResourceId;
use tokio::sync::{mpsc, oneshot};

pub type Result<T = ()> = surrealdb::Result<T>;
type Tx<T> = oneshot::Sender<Result<T>>;
type Rx = mpsc::UnboundedReceiver<Command>;

pub const NAMESPACE: &str = "syre";
pub const DATABASE: &str = "datastore";

const DEFINE_TABLE_PROJECT: &str = "
DEFINE TABLE project SCHEMAFULL;

DEFINE FIELD name ON TABLE project TYPE string;
DEFINE FIELD description ON TABLE project TYPE option<string>;
DEFINE FIELD base_path ON TABLE project TYPE string;
";

const DEFINE_TABLE_CONTAINER: &str = "
DEFINE TABLE container SCHEMAFULL;

DEFINE FIELD name ON TABLE container TYPE string;
DEFINE FIELD kind ON TABLE container TYPE option<string>;
DEFINE FIELD description ON TABLE container TYPE option<string>;
DEFINE FIELD tags ON TABLE container TYPE set<string>;
DEFINE FIELD metadata ON TABLE container TYPE object;
";

const DEFINE_TABLE_ASSET: &str = "
DEFINE TABLE asset SCHEMAFULL;

DEFINE FIELD name ON TABLE asset TYPE option<string>;
DEFINE FIELD kind ON TABLE asset TYPE option<string>;
DEFINE FIELD description ON TABLE asset TYPE option<string>;
DEFINE FIELD tags ON TABLE asset TYPE set<string>;
DEFINE FIELD metadata ON TABLE asset TYPE object;

DEFINE FIELD path ON TABLE asset TYPE string;
";

const DEFINE_SEARCH_INDICES: &str = "
DEFINE ANALYZER properties_analyzer 
    TOKENIZERS blank,class,punct 
    FILTERS lowercase,ascii,snowball(english);

DEFINE INDEX container_name ON container COLUMNS name SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_kind ON container COLUMNS kind SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_description ON container COLUMNS description SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_tags ON container COLUMNS tags SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_metadata ON container COLUMNS metadata SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);

DEFINE INDEX asset_name ON asset COLUMNS name SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_kind ON asset COLUMNS kind SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_description ON asset COLUMNS description SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_tags ON asset COLUMNS tags SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_metadata ON asset COLUMNS metadata SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_path ON asset COLUMNS path SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
";

pub struct Datastore {
    command_rx: Rx,
}

impl Datastore {
    pub fn new(command_rx: Rx) -> Self {
        Self { command_rx }
    }

    #[tokio::main]
    pub async fn run(&mut self) -> Result {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(NAMESPACE).use_db(DATABASE).await?;
        db.query(DEFINE_TABLE_PROJECT).await?;
        db.query(DEFINE_TABLE_CONTAINER).await?;
        db.query(DEFINE_TABLE_ASSET).await?;
        db.query(DEFINE_SEARCH_INDICES).await?;
        let store = Store::new(db);

        while let Some(cmd) = self.command_rx.recv().await {
            tracing::debug!(?cmd);
            match cmd {
                Command::Clear { tx } => store.clear(tx).await,
                Command::Query { query, tx } => store.handle_query(tx, query).await,
                Command::Search { tx, query } => store.handle_search(tx, query).await,
                Command::Project(cmd) => store.handle_command_project(cmd).await,
                Command::Graph(cmd) => store.handle_command_graph(cmd).await,
                Command::Container(cmd) => store.handle_command_container(cmd).await,
                Command::Asset(cmd) => store.handle_command_asset(cmd).await,
            }
        }

        tracing::debug!("shutting down datastore");
        Ok(())
    }
}

struct Store {
    db: Surreal<Db>,
}

impl Store {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    /// Clear all records in all tables.
    pub async fn clear(&self, tx: Tx<()>) {
        if let Err(err) = self.db.delete::<Vec<project::Record>>("project").await {
            Self::send_response(tx, Err(err));
            return;
        }

        Self::send_response(tx, Ok(()))
    }

    pub async fn handle_query(&self, tx: Tx<surrealdb::Response>, query: String) {
        Self::send_response(tx, self.db.query(query).await);
    }

    pub async fn handle_search(&self, tx: Tx<Vec<ResourceId>>, query: String) {
        #[allow(dead_code)]
        #[derive(serde::Deserialize, Debug)]
        struct Record {
            id: surrealdb::sql::Thing,
            score: f64,
        }

        let container_query = format!(
            "SELECT
                id,
                math::product([
                    search::score(0) * 3 
                    + search::score(1) * 3 
                    + search::score(2) * 1 
                    + search::score(3) * 2 
                    + search::score(4) * 2,
                    0.090909 // normalization
                ]) AS score
            FROM container
            WHERE name @0@ '{query}'
                OR kind @1@ '{query}'
                OR description @2@ '{query}'
                OR tags @3@ '{query}'
                OR metadata @4@ '{query}'
            ORDER BY score DESC"
        );

        let asset_query = format!(
            "SELECT
                id,
                math::product([
                    search::score(0) * 3 
                    + search::score(1) * 3 
                    + search::score(2) * 1 
                    + search::score(3) * 2 
                    + search::score(4) * 2
                    + search::score(5) * 3,
                    0.071429 // normalization
                ]) AS score
            FROM asset
            WHERE name @0@ '{query}'
                OR kind @1@ '{query}'
                OR description @2@ '{query}'
                OR tags @3@ '{query}'
                OR metadata @4@ '{query}'
                OR path @5@ '{query}'
            ORDER BY score DESC"
        );

        let mut container_results = match self.db.query(container_query).await {
            Ok(results) => results,
            Err(err) => {
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut asset_results = match self.db.query(asset_query).await {
            Ok(results) => results,
            Err(err) => {
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let container_results = match container_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut asset_results = match asset_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut results = container_results;
        results.append(&mut asset_results);
        results.sort_by(|ra, rb| ra.score.partial_cmp(&rb.score).unwrap());

        let results = results
            .into_iter()
            .map(|record| ResourceId::from_str(record.id.id.to_raw().as_str()).unwrap())
            .collect();

        Self::send_response(tx, Ok(results));
    }

    fn send_response<T>(tx: Tx<T>, value: Result<T>) {
        match tx.send(value) {
            Ok(_) => {}
            Err(_) => tracing::error!("could not send response"),
        }
    }

    /// Selects the projects id that contains a resource.
    ///
    /// # Returns
    /// Id of the project that the resource belongs to.
    async fn project_from_resource_id(
        &self,
        kind: ResourceKind,
        id: &ResourceId,
    ) -> Result<Option<ResourceId>> {
        let table = match kind {
            ResourceKind::Container => "container",
            ResourceKind::Asset => "asset",
        };

        let mut result = self
            .db
            .query("SELECT in FROM has_resource WHERE out = type::thing($table, $id)")
            .bind(("table", table))
            .bind(("id", id))
            .await?;

        let Some(id) = result.take::<Option<String>>(0)? else {
            return Ok(None);
        };

        let Some((_, rid)) = id.split_once(':') else {
            return Err(Error::Db(error::Db::IdInvalid { value: id }));
        };

        let Ok(rid) = ResourceId::from_str(rid) else {
            return Err(Error::Db(error::Db::IdInvalid {
                value: rid.to_string(),
            }));
        };

        Ok(Some(rid))
    }
}

pub mod project {
    use super::super::command::project::Command;
    use super::{Result, Store};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use syre_core::project::Project as CoreProject;
    use syre_core::types::ResourceId;
    use syre_local::project::resources::Project as LocalProject;

    impl Store {
        pub async fn handle_command_project(&self, cmd: Command) {
            match cmd {
                Command::Create { id, project, tx } => {
                    let resp = self.project_create(id, project).await;
                    Self::send_response(tx, resp);
                }

                Command::Update { id, project, tx } => {
                    let resp = self.project_update(id, project).await;
                    Self::send_response(tx, resp);
                }
            }
        }

        async fn project_create(&self, id: ResourceId, project: Record) -> Result {
            self.db
                .create::<Option<Record>>(("project", id.to_string()))
                .content(project)
                .await?;

            Ok(())
        }

        async fn project_update(&self, id: ResourceId, project: Record) -> Result {
            self.db
                .update::<Option<Record>>(("project", id.to_string()))
                .content(project)
                .await?;

            Ok(())
        }
    }

    #[allow(dead_code)]
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Record {
        name: String,
        description: Option<String>,
        base_path: PathBuf,
    }

    impl Record {
        pub fn new(name: String, description: Option<String>, base_path: PathBuf) -> Self {
            Self {
                name,
                description,
                base_path,
            }
        }
    }

    impl From<LocalProject> for Record {
        fn from(value: LocalProject) -> Self {
            let CoreProject {
                rid: _,
                creator: _,
                created: _,
                name,
                description,
                data_root: _,
                analysis_root: _,
                meta_level: _,
            } = value.inner().clone();

            let base_path = value.base_path().to_path_buf();

            Self {
                name,
                description,
                base_path,
            }
        }
    }
}

pub mod graph {
    use super::super::command::graph::{Command, ContainerTree};
    use super::asset::Record as AssetRecord;
    use super::container::Record as ContainerRecord;
    use super::{Result, Store};
    use std::collections::HashMap;
    use surrealdb::sql::Thing;
    use syre_core::graph::ResourceNode;
    use syre_core::project::{Asset, Container};
    use syre_core::types::ResourceId;

    type Nodes = HashMap<ResourceId, ResourceNode<Container>>;

    impl Store {
        pub async fn handle_command_graph(&self, cmd: Command) {
            match cmd {
                Command::Create { tx, graph, project } => {
                    let resp = self.graph_create(graph, project).await;
                    Self::send_response(tx, resp);
                }
            }
        }

        async fn graph_create(&self, graph: ContainerTree, project: ResourceId) -> Result {
            let (nodes, edges) = graph.into_components();
            let Resources { containers, assets } = nodes_to_records(nodes);
            for container in containers {
                let ContainerInfo { id, record } = container;
                self.db
                    .create::<Option<ContainerRecord>>(("container", id.to_string()))
                    .content(record)
                    .await?;

                self.db
                    .query("RELATE $project -> has_resource -> $id")
                    .bind((
                        "project",
                        Thing::from(("project", project.clone().into_surreal_id())),
                    ))
                    .bind(("id", Thing::from(("container", id.into_surreal_id()))))
                    .await?;
            }

            for (parent, children) in edges {
                for child in children {
                    self.db
                        .query("RELATE $parent -> has_child -> $child")
                        .bind((
                            "parent",
                            Thing::from(("container", parent.clone().into_surreal_id())),
                        ))
                        .bind((
                            "child",
                            Thing::from(("container", child.clone().into_surreal_id())),
                        ))
                        .await?;
                }
            }

            for asset in assets {
                let AssetInfo {
                    id,
                    record,
                    container,
                } = asset;
                self.db
                    .create::<Option<AssetRecord>>(("asset", id.to_string()))
                    .content(record)
                    .await?;

                self.db
                    .query("RELATE $container -> has_asset -> $id")
                    .bind((
                        "container",
                        Thing::from(("container", container.clone().into_surreal_id())),
                    ))
                    .bind(("id", Thing::from(("asset", id.clone().into_surreal_id()))))
                    .await?;

                self.db
                    .query("RELATE $project -> has_resource -> $id")
                    .bind((
                        "project",
                        Thing::from(("project", project.clone().into_surreal_id())),
                    ))
                    .bind(("id", Thing::from(("asset", id.into_surreal_id()))))
                    .await?;
            }

            Ok(())
        }
    }

    fn nodes_to_records(nodes: Nodes) -> Resources {
        let mut container_info = Vec::with_capacity(nodes.len());
        let mut asset_info = Vec::new();
        for container in nodes.into_values() {
            let Container {
                rid: cid,
                properties,
                assets,
                analyses: _,
            } = container.into_data();

            container_info.push(ContainerInfo {
                id: cid.clone(),
                record: properties.into(),
            });

            for asset in assets.into_values() {
                let Asset {
                    rid: aid,
                    properties,
                    path,
                } = asset;

                asset_info.push(AssetInfo {
                    id: aid,
                    record: AssetRecord::new(properties, path),
                    container: cid.clone(),
                });
            }
        }

        Resources {
            containers: container_info,
            assets: asset_info,
        }
    }

    struct Resources {
        pub containers: Vec<ContainerInfo>,
        pub assets: Vec<AssetInfo>,
    }

    struct ContainerInfo {
        id: ResourceId,
        record: ContainerRecord,
    }

    struct AssetInfo {
        id: ResourceId,
        record: AssetRecord,
        container: ResourceId,
    }
}

pub mod container {
    use crate::command::search::ResourceKind;

    use super::super::command::container::Command;
    use super::{Result, Store};
    use serde::{Deserialize, Serialize};
    use surrealdb::{error, sql::Thing, Error};
    use syre_core::project::{ContainerProperties, Metadata};
    use syre_core::types::ResourceId;

    impl Store {
        pub async fn handle_command_container(&self, cmd: Command) {
            match cmd {
                Command::Create {
                    tx,
                    id,
                    container,
                    parent,
                } => {
                    let resp = self.container_create(id, container, parent).await;
                    Self::send_response(tx, resp);
                }

                Command::Update { tx, id, container } => {
                    let resp = self.container_update(id, container).await;
                    Self::send_response(tx, resp);
                }
            }
        }

        async fn container_create(
            &self,
            id: ResourceId,
            container: Record,
            parent: ResourceId,
        ) -> Result {
            let Some(project) = self
                .project_from_resource_id(ResourceKind::Container, &parent)
                .await?
            else {
                return Err(Error::Db(error::Db::NoRecordFound));
            };

            self.db
                .create::<Option<Record>>(("container", id.to_string()))
                .content(container)
                .await?;

            self.db
                .query("RELATE $parent -> has_child -> $id")
                .bind((
                    "parent",
                    Thing::from(("container", parent.into_surreal_id())),
                ))
                .bind((
                    "id",
                    Thing::from(("container", id.clone().into_surreal_id())),
                ))
                .await?;

            self.db
                .query("RELATE $project -> has_resource -> $id")
                .bind((
                    "project",
                    Thing::from(("project", project.into_surreal_id())),
                ))
                .bind(("id", Thing::from(("container", id.into_surreal_id()))))
                .await?;

            Ok(())
        }

        async fn container_update(&self, id: ResourceId, container: Record) -> Result {
            self.db
                .update::<Option<Record>>(("container", id.to_string()))
                .content(container)
                .await?;

            Ok(())
        }
    }

    #[allow(dead_code)]
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Record {
        name: String,
        kind: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        metadata: Metadata,
    }

    impl From<ContainerProperties> for Record {
        fn from(value: ContainerProperties) -> Self {
            let ContainerProperties {
                creator: _,
                name,
                kind,
                description,
                tags,
                metadata,
                ..
            } = value;

            Self {
                name,
                kind,
                description,
                tags,
                metadata,
            }
        }
    }
}

pub mod asset {
    use super::super::command::asset::Command;
    use super::{Result, Store};
    use crate::command::search::ResourceKind;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use surrealdb::{error, sql::Thing, Error};
    use syre_core::project::{Asset, AssetProperties, Metadata};
    use syre_core::types::ResourceId;

    impl Store {
        pub async fn handle_command_asset(&self, cmd: Command) {
            match cmd {
                Command::Create {
                    tx,
                    id,
                    asset,
                    container,
                } => {
                    let resp = self.asset_create(id, asset, container).await;
                    Self::send_response(tx, resp);
                }

                Command::Update { tx, id, asset } => {
                    let resp = self.asset_update(id, asset).await;
                    Self::send_response(tx, resp);
                }

                Command::Remove { tx, id } => {
                    let resp = self.asset_remove(id).await;
                    Self::send_response(tx, resp);
                }
            }
        }

        async fn asset_create(
            &self,
            id: ResourceId,
            asset: Record,
            container: ResourceId,
        ) -> Result {
            let Some(project) = self
                .project_from_resource_id(ResourceKind::Container, &container)
                .await?
            else {
                return Err(Error::Db(error::Db::NoRecordFound));
            };

            self.db
                .create::<Option<Record>>(("asset", id.to_string()))
                .content(asset)
                .await?;

            self.db
                .query("RELATE $container -> has_asset -> $id")
                .bind((
                    "container",
                    Thing::from(("container", container.into_surreal_id())),
                ))
                .bind(("id", Thing::from(("asset", id.clone().into_surreal_id()))))
                .await?;

            self.db
                .query("RELATE $project -> has_resource -> $id")
                .bind((
                    "project",
                    Thing::from(("project", project.into_surreal_id())),
                ))
                .bind(("id", Thing::from(("asset", id.into_surreal_id()))))
                .await?;

            Ok(())
        }

        async fn asset_update(&self, id: ResourceId, asset: Record) -> Result {
            self.db
                .update::<Option<Record>>(("asset", id.to_string()))
                .content(asset)
                .await?;

            Ok(())
        }

        async fn asset_remove(&self, id: ResourceId) -> Result {
            self.db
                .delete::<Option<Record>>(("asset", id.into_surreal_id()))
                .await?;

            Ok(())
        }
    }

    #[allow(dead_code)]
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Record {
        name: Option<String>,
        kind: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        metadata: Metadata,
        path: PathBuf,
    }

    impl Record {
        pub fn new(properties: AssetProperties, path: PathBuf) -> Self {
            let AssetProperties {
                creator: _,
                name,
                kind,
                description,
                tags,
                metadata,
                ..
            } = properties;

            Self {
                name,
                kind,
                description,
                tags,
                metadata,
                path,
            }
        }
    }

    impl From<Asset> for Record {
        fn from(value: Asset) -> Self {
            let Asset {
                rid: _,
                properties,
                path,
            } = value;

            let AssetProperties {
                name,
                kind,
                description,
                tags,
                metadata,
                ..
            } = properties;

            Self {
                name,
                kind,
                description,
                tags,
                metadata,
                path,
            }
        }
    }
}

#[cfg(test)]
#[path = "./data_store_test.rs"]
mod data_store_test;
