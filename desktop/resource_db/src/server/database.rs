use crate::Command;
use std::{path::PathBuf, str::FromStr, thread};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use syre_core::types::ResourceId;
use syre_project_watcher as project_watcher;
use tokio::sync::{mpsc, oneshot};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum ResourceKind {
    Container,
    Asset,
}

type Tx<T> = oneshot::Sender<surrealdb::Result<T>>;

pub const NAMESPACE: &str = "syre";
pub const DATABASE: &str = "resource_db";

#[derive(Debug, serde::Deserialize)]
struct IdRecord {
    pub id: surrealdb::RecordId,
}

const DEFINE_TABLE_USER: &str = "
DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD email ON TABLE user TYPE string;
";

const DEFINE_TABLE_PROJECT: &str = "
DEFINE TABLE project SCHEMAFULL;

DEFINE FIELD path       ON TABLE project TYPE string;
DEFINE FIELD properties ON TABLE project TYPE option<record<project_properties>>;
DEFINE FIELD settings   ON TABLE project TYPE option<record<project_settings>>;
";

const DEFINE_TABLE_PROJECT_PROPERTIES: &str = "
DEFINE TABLE project_properties SCHEMAFULL;

DEFINE FIELD _project       ON TABLE project_properties TYPE record<project>;
DEFINE FIELD name           ON TABLE project_properties TYPE string;
DEFINE FIELD description    ON TABLE project_properties TYPE option<string>;
DEFINE FIELD data_root      ON TABLE project_properties TYPE string;
DEFINE FIELD analysis_root  ON TABLE project_properties TYPE option<string>;
";

const DEFINE_TABLE_PROJECT_SETTINGS: &str = "
DEFINE TABLE project_settings SCHEMAFULL;

DEFINE FIELD _project   ON TABLE project_settings TYPE record<project>;
DEFINE FIELD creator    ON TABLE project_settings TYPE option<{ Email: string } | { Id: bytes }>;
DEFINE FIELD created    ON TABLE project_settings TYPE datetime;
";

// NB: `env` field may need to be marked as `FLEXIBLE`.
// https://surrealdb.com/docs/surrealdb/surrealql/statements/define/field#flexible-data-types
const DEFINE_TABLE_ANALYSIS: &str = "
DEFINE TABLE analysis SCHEMAFULL;

DEFINE FIELD _project       ON TABLE analysis TYPE record<project>;
DEFINE FIELD path           ON TABLE analysis TYPE string;
DEFINE FIELD name           ON TABLE analysis TYPE option<string>;
DEFINE FIELD description    ON TABLE analysis TYPE option<string>;

DEFINE FIELD language       ON TABLE analysis TYPE string;
DEFINE FIELD cmd            ON TABLE analysis TYPE string;
DEFINE FIELD args           ON TABLE analysis TYPE array<string>;
DEFINE FIELD env            ON TABLE analysis TYPE object;

DEFINE FIELD creator        ON TABLE analysis TYPE option<bytes>;
DEFINE FIELD created        ON TABLE analysis TYPE datetime;
";

const DEFINE_TABLE_CONTAINER: &str = "
DEFINE TABLE container SCHEMAFULL;

DEFINE FIELD _project   ON TABLE container TYPE record<project>;
DEFINE FIELD path       ON TABLE container TYPE string;
DEFINE FIELD properties ON TABLE container TYPE option<record<container_properties>>;
DEFINE FIELD settings   ON TABLE container TYPE option<record<container_settings>>;
";

const DEFINE_TABLE_CONTAINER_PROPERTIES: &str = "
DEFINE TABLE container_properties SCHEMAFULL;

DEFINE FIELD _project       ON TABLE container_properties TYPE record<project>;
DEFINE FIELD _container     ON TABLE container_properties TYPE record<container>;
DEFINE FIELD name           ON TABLE container_properties TYPE string;
DEFINE FIELD kind           ON TABLE container_properties TYPE option<string>;
DEFINE FIELD description    ON TABLE container_properties TYPE option<string>;
DEFINE FIELD tags           ON TABLE container_properties TYPE set<string>;
DEFINE FIELD metadata       ON TABLE container_properties TYPE object;
";

const DEFINE_TABLE_CONTAINER_SETTINGS: &str = "
DEFINE TABLE container_settings SCHEMAFULL;

DEFINE FIELD _project   ON TABLE container_settings TYPE record<project>;
DEFINE FIELD _container ON TABLE container_settings TYPE record<container>;
DEFINE FIELD creator    ON TABLE container_settings TYPE option<{ Email: string } | { Id: bytes }>;
DEFINE FIELD created    ON TABLE container_settings TYPE datetime;
";

const DEFINE_TABLE_ASSET: &str = "
DEFINE TABLE asset SCHEMAFULL;

DEFINE FIELD _project       ON TABLE asset TYPE record<project>;
DEFINE FIELD _container     ON TABLE asset TYPE record<container>;
DEFINE FIELD name           ON TABLE asset TYPE option<string>;
DEFINE FIELD kind           ON TABLE asset TYPE option<string>;
DEFINE FIELD description    ON TABLE asset TYPE option<string>;
DEFINE FIELD tags           ON TABLE asset TYPE set<string>;
DEFINE FIELD metadata       ON TABLE asset TYPE object;

DEFINE FIELD created        ON TABLE asset TYPE datetime;
DEFINE FIELD creator        ON TABLE asset TYPE
      { User: option<{ Email: string } | { Id: bytes }> }
    | { Script: bytes };
    
    DEFINE FIELD path                   ON TABLE asset TYPE string;
    DEFINE FIELD fs_resource_present    ON TABLE asset TYPE bool;
    ";

const DEFINE_TABLE_FLAG: &str = "
DEFINE TABLE flag SCHEMAFULL;

DEFINE FIELD _project       ON TABLE flag TYPE record<project>;
DEFINE FIELD resource       ON TABLE flag TYPE option<record<container> | record<asset>>;

DEFINE FIELD path           ON TABLE flag TYPE string;
DEFINE FIELD severity       ON TABLE flag TYPE 'Info' | 'Warning' | 'Error';
DEFINE FIELD message        ON TABLE flag TYPE string;

";

const DEFINE_TABLE_PERMISSIONS: &str = "
DEFINE TABLE permissions SCHEMAFULL;

DEFINE FIELD resource   ON TABLE permissions TYPE record<string>;
DEFINE FIELD user       ON TABLE permissions TYPE record<user>;
DEFINE FIELD owner      ON TABLE permissions TYPE bool;
DEFINE FIELD read       ON TABLE permissions TYPE bool;
DEFINE FIELD write      ON TABLE permissions TYPE bool;
DEFINE FIELD execute    ON TABLE permissions TYPE bool;

DEFINE INDEX id         ON TABLE permissions FIELDS resource, user UNIQUE;
";

const DEFINE_SEARCH_INDICES: &str = "
DEFINE ANALYZER properties_analyzer 
    TOKENIZERS blank, class, punct 
    FILTERS lowercase, ascii, snowball(english), ngram(1, 15);

DEFINE INDEX container_name         ON container_properties COLUMNS name SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_kind         ON container_properties COLUMNS kind SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_description  ON container_properties COLUMNS description SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_tags         ON container_properties COLUMNS tags SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_metadata     ON container_properties COLUMNS metadata SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);

DEFINE INDEX asset_name         ON asset COLUMNS name SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_kind         ON asset COLUMNS kind SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_description  ON asset COLUMNS description SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_tags         ON asset COLUMNS tags SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_metadata     ON asset COLUMNS metadata SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_path         ON asset COLUMNS path SEARCH ANALYZER properties_analyzer BM25(1.2, 0.75);
";

pub struct Builder {
    query_rx: mpsc::UnboundedReceiver<Command>,
}

impl Builder {
    pub fn new(query_rx: mpsc::UnboundedReceiver<Command>) -> Self {
        Self { query_rx }
    }

    #[tokio::main]
    pub async fn run(self) -> surrealdb::Result<()> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(NAMESPACE).use_db(DATABASE).await?;
        db.query(DEFINE_TABLE_PROJECT).await?;
        db.query(DEFINE_TABLE_PROJECT_PROPERTIES).await?;
        db.query(DEFINE_TABLE_PROJECT_SETTINGS).await?;
        db.query(DEFINE_TABLE_CONTAINER).await?;
        db.query(DEFINE_TABLE_CONTAINER_PROPERTIES).await?;
        db.query(DEFINE_TABLE_CONTAINER_SETTINGS).await?;
        db.query(DEFINE_TABLE_ASSET).await?;
        db.query(DEFINE_TABLE_FLAG).await?;
        db.query(DEFINE_SEARCH_INDICES).await?;

        let (project_event_tx, project_event_rx) = mpsc::unbounded_channel();
        let project_actor = super::project_watcher_actor::Builder::new(project_event_tx);
        thread::Builder::new()
            .name("syre desktop resource db project watcher actor".to_string())
            .spawn(move || {
                project_actor.run();
            })
            .expect("could not launch project watcher actor");

        let mut db = Database::new(db, self.query_rx, project_event_rx);
        db.run().await;
        Ok(())
    }
}

struct Database {
    store: Store,
    query_rx: mpsc::UnboundedReceiver<Command>,
    project_event_rx: mpsc::UnboundedReceiver<Vec<project_watcher::Update>>,
}

impl Database {
    fn new(
        store: Surreal<Db>,
        query_rx: mpsc::UnboundedReceiver<Command>,
        project_event_rx: mpsc::UnboundedReceiver<Vec<project_watcher::Update>>,
    ) -> Self {
        Self {
            store: Store::new(store),
            query_rx,
            project_event_rx,
        }
    }

    async fn run(&mut self) {
        self.init_state().await.unwrap();

        loop {
            tokio::select! {
                events = self.project_event_rx.recv() => {
                    let Some(events) = events else {
                        tracing::debug!("`progress_event` channel closed");
                        break;
                    };
                    tracing::debug!(?events);

                    self.handle_project_events(events).await;
                }
                cmd = self.query_rx.recv() => {
                    let Some(cmd) = cmd else {
                        tracing::debug!("`query` channel closed");
                        break;
                    };
                    tracing::debug!(?cmd);

                    match cmd {
                        Command::Query { query, tx } => self.store.handle_query(tx, query).await,
                        Command::Search { tx, query } => self.store.handle_search(tx, query).await,
                    }
                },
            }
        }

        tracing::trace!("shutting down");
    }

    async fn init_state(&self) -> surrealdb::Result<()> {
        let db = project_watcher::Client::new();
        let projects = db.state().projects().expect("could not get project states");
        for project in projects.iter() {
            if let Err(err) = self.init_project(&db, project).await {
                tracing::error!("could not load project {:?}: {err:?}", project.path());
            }
        }

        Ok(())
    }

    async fn init_project(
        &self,
        db: &project_watcher::Client,
        project: &project_watcher::state::Project,
    ) -> Result<(), error::ProjectInit> {
        #[derive(serde::Serialize)]
        struct ProjectRecordLinks {
            properties: Option<surrealdb::RecordId>,
            settings: Option<surrealdb::RecordId>,
        }

        let project_record_id = self
            .store
            .insert_project(project.path().clone())
            .await
            .unwrap();

        let project_watcher::state::FolderResource::Present(project_data) = project.fs_resource()
        else {
            tracing::trace!(
                "project {:?} file system resource is absent",
                project.path()
            );
            return Ok(());
        };

        let project_watcher::state::DataResource::Ok(properties) = project_data.properties() else {
            tracing::trace!("project {:?} properties is corrupt", project.path());
            return Err(error::ProjectInit::PropertiesCorrupt);
        };
        let properties_record_id = self
            .store
            .insert_project_properties(project_record_id.clone(), properties.clone())
            .await
            .unwrap();

        let settings_record_id = match project_data.settings() {
            project_watcher::state::DataResource::Ok(settings) => {
                let id = self
                    .store
                    .insert_project_settings(
                        project_record_id.clone(),
                        settings.creator.clone(),
                        settings.created.clone(),
                    )
                    .await
                    .unwrap();

                Some(id)
            }
            Err(err) => {
                tracing::trace!("project {:?} settings is corrupt: {err:?}", project.path());
                None
            }
        };

        let _update: Option<project::Record> = self
            .store
            .db
            .update(project_record_id.clone())
            .merge(ProjectRecordLinks {
                properties: Some(properties_record_id.clone()),
                settings: settings_record_id,
            })
            .await
            .unwrap();
        assert!(_update.is_some());

        let (containers, assets) = match self
            .init_project_resources(db, project_record_id.clone(), properties.rid().clone())
            .await
        {
            Ok(resources) => resources,
            Err(error::ProjectResourcesInit::ProjectNotFound) => panic!("project not found"),
        };

        // TODO: Update project with resources.

        Ok(())
    }

    /// Initialize a project's resources, i.e. containers and assets.
    ///
    /// # Returns
    /// `(container record ids, asset record ids)`
    async fn init_project_resources(
        &self,
        db: &project_watcher::Client,
        project_id: surrealdb::RecordId,
        project: ResourceId,
    ) -> Result<(Vec<surrealdb::RecordId>, Vec<surrealdb::RecordId>), error::ProjectResourcesInit>
    {
        #[derive(serde::Serialize)]
        struct ContainerRecordLinks {
            properties: Option<surrealdb::RecordId>,
            settings: Option<surrealdb::RecordId>,
        }

        let Some((_, _, graph)) = db.project().resources(project).unwrap() else {
            return Err(error::ProjectResourcesInit::ProjectNotFound);
        };

        let project_watcher::state::FolderResource::Present(graph) = graph else {
            return Ok((vec![], vec![]));
        };

        let paths = paths_from_graph_data(&graph);
        let mut containers = Vec::with_capacity(graph.nodes.len());
        let mut assets = vec![];
        for (path, container) in std::iter::zip(paths, graph.nodes.iter()) {
            let container_record_id = self
                .store
                .insert_container(project_id.clone(), path)
                .await
                .unwrap();

            containers.push(container_record_id.clone());

            let properties_record_id = if let project_watcher::state::DataResource::Ok(properties) =
                container.properties()
            {
                let rid = container.rid().unwrap();
                let record_id = self
                    .store
                    .insert_container_properties(
                        project_id.clone(),
                        container_record_id.clone(),
                        rid.clone(),
                        properties.clone(),
                    )
                    .await
                    .unwrap();

                Some(record_id)
            } else {
                None
            };

            let settings_record_id =
                if let project_watcher::state::DataResource::Ok(settings) = container.settings() {
                    let record_id = self
                        .store
                        .insert_container_settings(
                            project_id.clone(),
                            container_record_id.clone(),
                            settings.clone(),
                        )
                        .await
                        .unwrap();

                    Some(record_id)
                } else {
                    None
                };

            let _update: Option<container::Record> = self
                .store
                .db
                .update(&container_record_id)
                .merge(ContainerRecordLinks {
                    properties: properties_record_id,
                    settings: settings_record_id,
                })
                .await
                .unwrap();
            assert!(_update.is_some());

            let mut asset_record_ids = Vec::with_capacity(assets.len());
            if let project_watcher::state::DataResource::Ok(assets) = container.assets() {
                for asset in assets {
                    let record_id = self
                        .store
                        .insert_asset(
                            project_id.clone(),
                            container_record_id.clone(),
                            asset.clone(),
                        )
                        .await
                        .unwrap();

                    asset_record_ids.push((asset.path.clone(), record_id));
                }
            }

            if let project_watcher::state::DataResource::Ok(flags) = container.flags() {
                let root_dir = PathBuf::from("/");
                for (path, resource_flags) in flags {
                    let resource = if *path == root_dir {
                        Some(container_record_id.clone())
                    } else {
                        asset_record_ids.iter().find_map(|(asset_path, record_id)| {
                            (asset_path == path).then_some(record_id.clone())
                        })
                    };

                    for flag in resource_flags {
                        let record_id = self
                            .store
                            .insert_flag(project_id.clone(), resource.clone(), path.clone(), flag)
                            .await
                            .unwrap();
                    }
                }
            }
        }

        Ok((containers, assets))
    }
}

fn paths_from_graph_data(graph: &project_watcher::state::Graph) -> Vec<PathBuf> {
    fn inner(
        graph: &project_watcher::state::Graph,
        paths: &mut Vec<PathBuf>,
        idx: usize,
        root: &PathBuf,
    ) {
        let node = graph.nodes.get(idx).unwrap();
        let root = root.join(node.name());
        paths[idx] = root.clone();
        for child in graph.children[idx].iter() {
            inner(graph, paths, *child, &root);
        }
    }

    let mut paths = vec![PathBuf::new(); graph.nodes.len()];
    inner(graph, &mut paths, 0, &PathBuf::new());
    paths
}

pub mod error {
    #[derive(Debug)]
    pub enum ProjectInit {
        PropertiesCorrupt,
    }

    pub enum ProjectResourcesInit {
        ProjectNotFound,
    }
}

impl Database {
    async fn handle_project_events(&self, events: Vec<project_watcher::Update>) {}
}

struct Store {
    db: Surreal<Db>,
}

impl Store {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    pub async fn handle_query(&self, tx: Tx<surrealdb::Response>, query: String) {
        Self::send_response(tx, self.db.query(query).await);
    }

    pub async fn handle_search(&self, tx: Tx<Vec<ResourceId>>, query: String) {
        #[allow(dead_code)]
        #[derive(serde::Deserialize, Debug)]
        struct Record {
            id: surrealdb::RecordIdKey,
            score: f64,
        }

        let query = escape_string(query);
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
            .map(|record| ResourceId::from_str(&record.id.to_string()).unwrap())
            .collect();

        Self::send_response(tx, Ok(results));
    }

    fn send_response<T>(tx: Tx<T>, value: surrealdb::Result<T>) {
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
        id: ResourceId,
    ) -> surrealdb::Result<Option<ResourceId>> {
        let table = match kind {
            ResourceKind::Container => "container",
            ResourceKind::Asset => "asset",
        };

        let mut result = self
            .db
            // .query("SELECT in FROM has_resource WHERE out = type::thing($table, $id)")
            // .bind(("table", table))
            // .bind(("id", id))
            .query("SELECT in AS id FROM has_resource WHERE out = $id")
            .bind(("id", (table, id.into_surreal_id())))
            .await?;

        let result = result.take::<Vec<IdRecord>>(0)?;
        if result.is_empty() {
            return Ok(None);
        };

        assert_eq!(result.len(), 1);
        let rid = result[0].id.to_string();
        let Ok(rid) = ResourceId::from_str(&rid) else {
            return Err(surrealdb::Error::Db(surrealdb::error::Db::IdInvalid {
                value: rid.to_string(),
            }));
        };

        Ok(Some(rid))
    }
}

mod project {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use syre_core as core;

    impl Store {
        /// # Panics
        /// If the insertion or subsequent query to obtain the id fails.
        pub async fn insert_project(
            &self,
            path: PathBuf,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = self
                .db
                .create::<Option<IdRecord>>("project")
                .content(Record { path })
                .await?
                .unwrap();

            Ok(record.id)
        }

        /// Inserts project properties keyed by the project's resource id.
        ///
        /// # Panics
        /// If the insertion or subsequent query to obtain the id fails.
        pub async fn insert_project_properties(
            &self,
            project_id: surrealdb::RecordId,
            project: core::project::Project,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let id = project.rid().clone();
            let core::project::Project {
                name,
                description,
                data_root,
                analysis_root,
                ..
            } = project;

            let record = PropertiesRecord {
                _project: project_id,
                name,
                description,
                data_root,
                analysis_root,
            };

            let record = self
                .db
                .create::<Option<IdRecord>>(("project_properties", id.to_string()))
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }

        /// Inserts project settings.
        ///
        /// # Panics
        /// If the insertion or subsequent query to obtain the id fails.
        pub async fn insert_project_settings(
            &self,
            project_id: surrealdb::RecordId,
            creator: Option<core::types::UserId>,
            created: DateTime<Utc>,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = self
                .db
                .create::<Option<IdRecord>>("project_settings")
                .content(SettingsRecord {
                    _project: project_id,
                    creator,
                    created,
                })
                .await?
                .unwrap();

            Ok(record.id)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Record {
        path: PathBuf,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PropertiesRecord {
        _project: surrealdb::RecordId,
        name: String,
        description: Option<String>,
        data_root: PathBuf,
        analysis_root: Option<PathBuf>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SettingsRecord {
        _project: surrealdb::RecordId,
        creator: Option<core::types::UserId>,

        #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
        created: DateTime<Utc>,
    }
}

mod container {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
    use syre_local as local;

    impl Store {
        pub async fn insert_container(
            &self,
            project_id: surrealdb::RecordId,
            path: PathBuf,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = self
                .db
                .create::<Option<IdRecord>>("container")
                .content(Record {
                    _project: project_id,
                    path,
                })
                .await?
                .unwrap();

            Ok(record.id)
        }

        pub async fn insert_container_properties(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            id: core::types::ResourceId,
            properties: core::project::ContainerProperties,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let core::project::ContainerProperties {
                name,
                kind,
                description,
                tags,
                metadata,
            } = properties;

            let record = PropertiesRecord {
                _project: project_id,
                _container: container_id,
                name,
                kind,
                description,
                tags,
                metadata,
            };

            let record = self
                .db
                .create::<Option<IdRecord>>(("container_properties", id.to_string()))
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }

        pub async fn insert_container_settings(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            settings: local::project::container::Settings,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let local::project::container::Settings {
                creator, created, ..
            } = settings;

            let record = SettingsRecord {
                _project: project_id,
                _container: container_id,
                creator,
                created,
            };

            let record = self
                .db
                .create::<Option<IdRecord>>("container_settings")
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Record {
        _project: surrealdb::RecordId,
        path: PathBuf,
    }

    #[derive(Serialize)]
    struct PropertiesRecord {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
        name: String,
        kind: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        metadata: HashMap<String, core::types::Value>,
    }

    #[derive(Serialize)]
    struct SettingsRecord {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
        creator: Option<core::types::UserId>,

        #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
        created: DateTime<Utc>,
    }
}

mod asset {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::Serialize;
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
    use syre_local as local;
    use syre_project_watcher as project_watcher;

    impl Store {
        pub async fn insert_asset(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            asset: project_watcher::state::Asset,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let fs_resource_present = asset.is_present();
            let rid = asset.rid().clone();
            let created = asset.properties.created().clone();
            let core::project::Asset {
                properties:
                    core::project::AssetProperties {
                        creator,
                        name,
                        kind,
                        description,
                        tags,
                        metadata,
                        ..
                    },
                path,
                ..
            } = asset.into_inner();

            let record = Record {
                _project: project_id,
                _container: container_id,
                name,
                kind,
                description,
                tags,
                metadata,
                path,
                fs_resource_present,
                creator,
                created,
            };

            let record = self
                .db
                .create::<Option<IdRecord>>(("asset", rid))
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }
    }

    #[derive(Serialize)]
    struct Record {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
        name: Option<String>,
        kind: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        metadata: HashMap<String, core::types::Value>,
        path: PathBuf,
        fs_resource_present: bool,

        creator: core::types::Creator,

        #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
        created: DateTime<Utc>,
    }
}

mod flag {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::Serialize;
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
    use syre_local as local;
    use syre_project_watcher as project_watcher;

    impl Store {
        pub async fn insert_flag(
            &self,
            project_id: surrealdb::RecordId,
            resource: Option<surrealdb::RecordId>,
            path: PathBuf,
            flag: &local::project::Flag,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = Record {
                _project: project_id,
                resource,
                path,
                severity: flag.severity(),
                message: flag.message().clone(),
            };

            let record = self
                .db
                .create::<Option<IdRecord>>(("flag", flag.id().to_string()))
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }
    }

    #[derive(Serialize)]
    struct Record {
        _project: surrealdb::RecordId,
        resource: Option<surrealdb::RecordId>,
        path: PathBuf,
        severity: local::project::flag::Severity,
        message: String,
    }
}

/// Escapes a string.
///
/// # Characters
/// + `'`
fn escape_string(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let input = input.replace("'", "\\'");
    input
}

mod cast {
    use serde::{Serialize, Serializer};

    pub fn chrono_as_sql_datetime<S>(
        t: &chrono::DateTime<chrono::Utc>,
        s: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Into::<surrealdb::sql::Datetime>::into(*t).serialize(s)
    }
}

// pub mod project {
//     use super::super::command::project::Command;
//     use super::{Result, Store};
//     use chrono::{DateTime, Utc};
//     use serde::{Deserialize, Serialize};
//     use std::path::PathBuf;
//     use syre_core::{project::Project as CoreProject, types::ResourceId};
//     use syre_local::project::{config::Settings, Project as LocalProject};

//     impl Store {
//         pub async fn handle_command_project(&self, cmd: Command) {
//             match cmd {
//                 Command::Create { id, project, tx } => {
//                     let resp = self.project_create(id, project).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::Update { id, project, tx } => {
//                     let resp = self.project_update(id, project).await;
//                     Self::send_response(tx, resp);
//                 }
//             }
//         }

//         async fn project_create(&self, id: ResourceId, project: Record) -> Result {
//             self.db
//                 .create::<Option<Record>>(("project", id.to_string()))
//                 .content(project)
//                 .await?;

//             Ok(())
//         }

//         async fn project_update(&self, id: ResourceId, project: Record) -> Result {
//             self.db
//                 .update::<Option<Record>>(("project", id.to_string()))
//                 .content(project)
//                 .await?;

//             Ok(())
//         }
//     }

//     #[allow(dead_code)]
//     #[derive(Debug, Serialize, Deserialize)]
//     pub struct Record {
//         name: String,
//         description: Option<String>,
//         data_root: PathBuf,
//         analysis_root: Option<PathBuf>,

//         creator: Option<syre_core::types::UserId>,
//         created: DateTime<Utc>,

//         base_path: PathBuf,
//     }

//     impl Record {
//         pub fn new(
//             base_path: impl Into<PathBuf>,
//             name: impl Into<String>,
//             data_root: impl Into<PathBuf>,
//             created: DateTime<Utc>,
//         ) -> Self {
//             Self {
//                 name: name.into(),
//                 description: None,
//                 data_root: data_root.into(),
//                 analysis_root: None,
//                 creator: None,
//                 created,
//                 base_path: base_path.into(),
//             }
//         }

//         pub fn set_description(&mut self, description: impl Into<String>) {
//             let _ = self.description.insert(description.into());
//         }

//         pub fn set_analysis_root(&mut self, analysis_root: impl Into<PathBuf>) {
//             let _ = self.analysis_root.insert(analysis_root.into());
//         }
//     }

//     impl From<LocalProject> for Record {
//         fn from(value: LocalProject) -> Self {
//             let (properties, settings, base_path) = value.into_parts();
//             let CoreProject {
//                 name,
//                 description,
//                 data_root,
//                 analysis_root,
//                 meta_level: _,
//                 ..
//             } = properties;

//             let Settings {
//                 local_format_version: _,
//                 created,
//                 creator,
//                 permissions: _,
//             } = settings;

//             Self {
//                 name,
//                 description,
//                 data_root,
//                 analysis_root,
//                 creator,
//                 created,
//                 base_path,
//             }
//         }
//     }
// }

// pub mod graph {
//     use super::{
//         super::command::graph::{Command, ContainerTree},
//         asset::Record as AssetRecord,
//         container::Record as ContainerRecord,
//         error, Error, Result, Store,
//     };
//     use futures::future::{BoxFuture, FutureExt};
//     use std::{collections::HashMap, str::FromStr};
//     use syre_core::{
//         graph::ResourceNode,
//         project::{Asset, Container},
//         types::ResourceId,
//     };

//     type Nodes = HashMap<ResourceId, ResourceNode<Container>>;

//     impl Store {
//         pub async fn handle_command_graph(&self, cmd: Command) {
//             match cmd {
//                 Command::Create { tx, graph, project } => {
//                     let resp = self.graph_create(graph, project).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::CreateSubgraph { tx, graph, parent } => {
//                     let resp = self.graph_create_subgraph(graph, parent).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::Remove { tx, root } => {
//                     let resp = self.graph_remove(root).await;
//                     Self::send_response(tx, resp);
//                 }
//             }
//         }

//         async fn graph_create(&self, graph: ContainerTree, project: ResourceId) -> Result {
//             let (nodes, edges) = graph.into_components();
//             let Resources { containers, assets } = nodes_to_records(nodes);
//             for container in containers {
//                 let ContainerInfo { id, record } = container;
//                 self.db
//                     .create::<Option<ContainerRecord>>(("container", id.to_string()))
//                     .content(record)
//                     .await?;

//                 self.db
//                     .query("RELATE $project -> has_resource -> $id")
//                     .bind(("project", ("project", project.clone().into_surreal_id())))
//                     .bind(("id", ("container", id.into_surreal_id())))
//                     .await?;
//             }

//             for (parent, children) in edges {
//                 for child in children {
//                     self.db
//                         .query("RELATE $parent -> has_child -> $child")
//                         .bind(("parent", ("container", parent.clone().into_surreal_id())))
//                         .bind(("child", ("container", child.clone().into_surreal_id())))
//                         .await?;
//                 }
//             }

//             for asset in assets {
//                 let AssetInfo {
//                     id,
//                     record,
//                     container,
//                 } = asset;
//                 self.db
//                     .create::<Option<AssetRecord>>(("asset", id.to_string()))
//                     .content(record)
//                     .await?;

//                 self.db
//                     .query("RELATE $container -> has_asset -> $id")
//                     .bind((
//                         "container",
//                         ("container", container.clone().into_surreal_id()),
//                     ))
//                     .bind(("id", ("asset", id.clone().into_surreal_id())))
//                     .await?;

//                 self.db
//                     .query("RELATE $project -> has_resource -> $id")
//                     .bind(("project", ("project", project.clone().into_surreal_id())))
//                     .bind(("id", ("asset", id.into_surreal_id())))
//                     .await?;
//             }

//             Ok(())
//         }

//         async fn graph_create_subgraph(&self, graph: ContainerTree, parent: ResourceId) -> Result {
//             let Some(project) = self
//                 .project_from_resource_id(super::ResourceKind::Container, parent.clone())
//                 .await?
//             else {
//                 return Err(Error::Db(error::Db::NoRecordFound));
//             };

//             let root = graph.root().clone();
//             self.graph_create(graph, project).await?;

//             self.db
//                 .query("RELATE $parent -> has_child -> $child")
//                 .bind(("parent", ("container", parent.clone().into_surreal_id())))
//                 .bind(("child", ("container", root.into_surreal_id())))
//                 .await?;

//             Ok(())
//         }

//         async fn graph_remove(&self, root: ResourceId) -> Result {
//             let containers = self.descendants(root).await?;
//             for container in containers {
//                 self.db
//                     .query("DELETE asset WHERE <-(has_asset WHERE in == $container)")
//                     .bind((
//                         "container",
//                         ("container", container.clone().into_surreal_id()),
//                     ))
//                     .await?;

//                 self.db
//                     .delete::<Option<super::container::Record>>(("container", container))
//                     .await?;
//             }

//             Ok(())
//         }

//         async fn children(&self, parent: ResourceId) -> Result<Vec<ResourceId>> {
//             #[derive(serde::Deserialize, Debug)]
//             struct Record {
//                 out: surrealdb::RecordIdKey,
//             }

//             let mut results = self
//                 .db
//                 .query("SELECT out FROM has_child WHERE in == $parent")
//                 .bind(("parent", ("container", parent.into_surreal_id())))
//                 .await?;

//             let results = results.take::<Vec<Record>>(0)?;
//             let ids = results
//                 .into_iter()
//                 .map(|record| ResourceId::from_str(&record.out.to_string()).unwrap())
//                 .collect();

//             Ok(ids)
//         }

//         // See https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
//         /// Get all descendant Containers.
//         /// Include root.
//         fn descendants(&self, root: ResourceId) -> BoxFuture<'_, Result<Vec<ResourceId>>> {
//             let mut descendants = vec![root.clone()];
//             async move {
//                 for child in self.children(root).await? {
//                     descendants.extend(self.descendants(child).await?);
//                 }

//                 Ok(descendants)
//             }
//             .boxed()
//         }
//     }

//     fn nodes_to_records(nodes: Nodes) -> Resources {
//         let mut container_info = Vec::with_capacity(nodes.len());
//         let mut asset_info = Vec::new();
//         for container in nodes.into_values() {
//             todo!();
//             // let Container {
//             //     rid: cid,
//             //     properties,
//             //     assets,
//             //     analyses: _,
//             // } = container.into_data();

//             // let record = ContainerRecord::;

//             // container_info.push(ContainerInfo {
//             //     id: cid.clone(),
//             //     record,
//             // });

//             // for asset in assets.into_values() {
//             //     let Asset {
//             //         rid: aid,
//             //         properties,
//             //         path,
//             //     } = asset;

//             //     asset_info.push(AssetInfo {
//             //         id: aid,
//             //         record: AssetRecord::from_properties(properties, path),
//             //         container: cid.clone(),
//             //     });
//             // }
//         }

//         Resources {
//             containers: container_info,
//             assets: asset_info,
//         }
//     }

//     struct Resources {
//         pub containers: Vec<ContainerInfo>,
//         pub assets: Vec<AssetInfo>,
//     }

//     struct ContainerInfo {
//         id: ResourceId,
//         record: ContainerRecord,
//     }

//     struct AssetInfo {
//         id: ResourceId,
//         record: AssetRecord,
//         container: ResourceId,
//     }
// }

// pub mod container {
//     use super::{super::command::container::Command, ResourceKind, Result, Store};
//     use chrono::{DateTime, Utc};
//     use serde::{Deserialize, Serialize};
//     use std::path::PathBuf;
//     use surrealdb::{error, Error};
//     use syre_core::{
//         project::{ContainerProperties, Metadata},
//         types::{ResourceId, UserId},
//     };
//     use syre_local::project::{container::Settings as ContainerSettings, Container};

//     impl Store {
//         pub async fn handle_command_container(&self, cmd: Command) {
//             match cmd {
//                 Command::Create {
//                     tx,
//                     id,
//                     container,
//                     parent,
//                 } => {
//                     let resp = self.container_create(id, container, parent).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::Update { tx, id, container } => {
//                     let resp = self.container_update(id, container).await;
//                     Self::send_response(tx, resp);
//                 }
//             }
//         }

//         async fn container_create(
//             &self,
//             id: ResourceId,
//             container: Record,
//             parent: ResourceId,
//         ) -> Result {
//             let Some(project) = self
//                 .project_from_resource_id(ResourceKind::Container, parent.clone())
//                 .await?
//             else {
//                 return Err(Error::Db(error::Db::NoRecordFound));
//             };

//             self.db
//                 .create::<Option<Record>>(("container", id.to_string()))
//                 .content(container)
//                 .await?;

//             self.db
//                 .query("RELATE $parent -> has_child -> $id")
//                 .bind(("parent", ("container", parent.into_surreal_id())))
//                 .bind(("id", ("container", id.clone().into_surreal_id())))
//                 .await?;

//             self.db
//                 .query("RELATE $project -> has_resource -> $id")
//                 .bind(("project", ("project", project.into_surreal_id())))
//                 .bind(("id", ("container", id.into_surreal_id())))
//                 .await?;

//             Ok(())
//         }

//         async fn container_update(&self, id: ResourceId, container: Record) -> Result {
//             self.db
//                 .update::<Option<Record>>(("container", id.to_string()))
//                 .content(container)
//                 .await?;

//             Ok(())
//         }
//     }

//     #[allow(dead_code)]
//     #[derive(Debug, Serialize, Deserialize)]
//     pub struct Record {
//         name: String,
//         kind: Option<String>,
//         description: Option<String>,
//         tags: Vec<String>,
//         metadata: Metadata,
//         creator: Option<UserId>,
//         created: DateTime<Utc>,
//         base_path: PathBuf,
//     }

//     impl From<Container> for Record {
//         fn from(value: Container) -> Self {
//             let (container, settings, base_path) = value.into_parts();
//             let ContainerProperties {
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 ..
//             } = container.properties;

//             let ContainerSettings {
//                 creator, created, ..
//             } = settings;

//             Self {
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 creator,
//                 created,
//                 base_path,
//             }
//         }
//     }
// }

// pub mod asset {
//     use super::{super::command::asset::Command, ResourceKind, Result, Store};
//     use chrono::{DateTime, Utc};
//     use serde::{Deserialize, Serialize};
//     use std::path::PathBuf;
//     use surrealdb::{error, Error};
//     use syre_core::{
//         project::{Asset, AssetProperties, Metadata},
//         types::ResourceId,
//     };

//     impl Store {
//         pub async fn handle_command_asset(&self, cmd: Command) {
//             match cmd {
//                 Command::Create {
//                     tx,
//                     id,
//                     asset,
//                     container,
//                 } => {
//                     let resp = self.asset_create(id, asset, container).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::Update { tx, id, asset } => {
//                     let resp = self.asset_update(id, asset).await;
//                     Self::send_response(tx, resp);
//                 }

//                 Command::Remove { tx, id } => {
//                     let resp = self.asset_remove(id).await;
//                     Self::send_response(tx, resp);
//                 }
//             }
//         }

//         async fn asset_create(
//             &self,
//             id: ResourceId,
//             asset: Record,
//             container: ResourceId,
//         ) -> Result {
//             let Some(project) = self
//                 .project_from_resource_id(ResourceKind::Container, container.clone())
//                 .await?
//             else {
//                 return Err(Error::Db(error::Db::NoRecordFound));
//             };

//             self.db
//                 .create::<Option<Record>>(("asset", id.to_string()))
//                 .content(asset)
//                 .await?;

//             self.db
//                 .query("RELATE $container -> has_asset -> $id")
//                 .bind(("container", ("container", container.into_surreal_id())))
//                 .bind(("id", ("asset", id.clone().into_surreal_id())))
//                 .await?;

//             self.db
//                 .query("RELATE $project -> has_resource -> $id")
//                 .bind(("project", ("project", project.into_surreal_id())))
//                 .bind(("id", ("asset", id.into_surreal_id())))
//                 .await?;

//             Ok(())
//         }

//         async fn asset_update(&self, id: ResourceId, asset: Record) -> Result {
//             self.db
//                 .update::<Option<Record>>(("asset", id.to_string()))
//                 .content(asset)
//                 .await?;

//             Ok(())
//         }

//         async fn asset_remove(&self, id: ResourceId) -> Result {
//             self.db
//                 .delete::<Option<Record>>(("asset", id.into_surreal_id()))
//                 .await?;

//             Ok(())
//         }
//     }

//     #[allow(dead_code)]
//     #[derive(Debug, Serialize, Deserialize)]
//     pub struct Record {
//         name: Option<String>,
//         kind: Option<String>,
//         description: Option<String>,
//         tags: Vec<String>,
//         metadata: Metadata,
//         path: PathBuf,
//         creator_kind: types::CreatorKind,
//         creator: Option<types::CreatorId>,
//         created: DateTime<Utc>,
//     }

//     impl Record {
//         pub fn new(
//             path: PathBuf,
//             creator: syre_core::types::Creator,
//             created: DateTime<Utc>,
//         ) -> Self {
//             let (creator_kind, creator) = types::creator_to_parts(creator);
//             Self {
//                 path,
//                 created,
//                 name: None,
//                 kind: None,
//                 description: None,
//                 tags: vec![],
//                 metadata: Metadata::new(),
//                 creator,
//                 creator_kind,
//             }
//         }

//         pub fn from_properties(properties: AssetProperties, path: PathBuf) -> Self {
//             let created = properties.created().clone();
//             let AssetProperties {
//                 creator,
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 ..
//             } = properties;
//             let (creator_kind, creator) = types::creator_to_parts(creator);
//             Self {
//                 path,
//                 created,
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 creator,
//                 creator_kind,
//             }
//         }
//     }

//     impl From<Asset> for Record {
//         fn from(value: Asset) -> Self {
//             let Asset {
//                 properties, path, ..
//             } = value;

//             let created = properties.created().clone();
//             let AssetProperties {
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 creator,
//                 ..
//             } = properties;
//             let (creator_kind, creator) = types::creator_to_parts(creator);

//             Self {
//                 name,
//                 kind,
//                 description,
//                 tags,
//                 metadata,
//                 path,
//                 created,
//                 creator,
//                 creator_kind,
//             }
//         }
//     }

//     mod types {
//         use serde::{Deserialize, Serialize};
//         use syre_core::types::{Creator, ResourceId, UserId};

//         #[derive(Debug, Serialize, Deserialize)]
//         pub enum CreatorId {
//             Id(ResourceId),
//             Email(String),
//         }

//         #[derive(Debug, Serialize, Deserialize)]
//         pub enum CreatorKind {
//             User,
//             Script,
//         }

//         /// Converts a [syre_core::Creator](syre_core::types::Creator) into its
//         /// corresponding components.
//         pub fn creator_to_parts(
//             creator: syre_core::types::Creator,
//         ) -> (CreatorKind, Option<CreatorId>) {
//             match creator {
//                 Creator::User(None) => (CreatorKind::User, None),
//                 Creator::User(Some(UserId::Id(id))) => (CreatorKind::User, Some(CreatorId::Id(id))),
//                 Creator::User(Some(UserId::Email(email))) => {
//                     (CreatorKind::User, Some(CreatorId::Email(email)))
//                 }
//                 Creator::Script(id) => (CreatorKind::Script, Some(CreatorId::Id(id))),
//             }
//         }
//     }
// }

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
