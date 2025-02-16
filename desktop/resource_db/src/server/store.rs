use std::{path::PathBuf, str::FromStr};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use syre_core::types::ResourceId;
use tokio::sync::oneshot;

type Tx<T> = oneshot::Sender<surrealdb::Result<T>>;

#[derive(Debug, serde::Deserialize)]
pub struct IdRecord {
    pub id: surrealdb::RecordId,
}

pub const NAMESPACE: &str = "syre";
pub const DATABASE: &str = "resource_db";

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
DEFINE FIELD name       ON TABLE container TYPE string;
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
DEFINE FIELD metadata       ON TABLE container_properties FLEXIBLE TYPE object;
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
DEFINE FIELD metadata       ON TABLE asset FLEXIBLE TYPE object;

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
DEFINE FIELD _container     ON TABLE flag TYPE record<container>;
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

#[derive(derive_more::Deref)]
pub struct Store {
    db: Surreal<Db>,
}

impl Store {
    pub async fn new() -> surrealdb::Result<Self> {
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

        Ok(Self { db })
    }

    /// Remove all records from all tables.
    pub async fn clear_all(&self) {
        let tables = vec![
            "project",
            "project_properties",
            "project_settings",
            "container",
            "container_properties",
            "container_settings",
            "asset",
            "flag",
        ];

        for table in tables {
            self.db.delete::<Vec<IdRecord>>(table).await.unwrap();
        }
    }

    pub async fn handle_query(&self, tx: Tx<surrealdb::Response>, query: String) {
        Self::send_response(tx, self.db.query(query).await);
    }

    pub async fn handle_search(
        &self,
        tx: Tx<Vec<ResourceId>>,
        query: String,
        project: Option<PathBuf>,
    ) {
        #[derive(serde::Deserialize, Debug)]
        struct Record {
            id: surrealdb::RecordId,
            score: f64,
        }

        let query = escape_string(query);
        let (project_id, project_where) = if let Some(project) = project {
            let project_id = match self.project_record_id_from_path(project).await {
                Ok(project_id) => project_id,
                Err(err) => {
                    tracing::error!(?err);
                    Self::send_response(tx, Err(err));
                    return;
                }
            };

            if project_id.is_none() {
                Self::send_response(tx, Ok(vec![]));
                return;
            }

            (project_id, "_project=type::thing($project) AND")
        } else {
            (None, "")
        };

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
            FROM container_properties
            WHERE
            {project_where}
            (
                name @0@ '{query}'
                OR kind @1@ '{query}'
                OR description @2@ '{query}'
                OR tags @3@ '{query}'
                OR metadata @4@ '{query}'
            )
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
            WHERE 
            {project_where}
            (
                name @0@ '{query}'
                OR kind @1@ '{query}'
                OR description @2@ '{query}'
                OR tags @3@ '{query}'
                OR metadata @4@ '{query}'
                OR path @5@ '{query}'
            )
            ORDER BY score DESC"
        );

        let container_results = if let Some(project_id) = &project_id {
            self.db
                .query(container_query)
                .bind(("project", project_id.clone()))
                .await
        } else {
            self.db.query(container_query).await
        };

        let mut container_results = match container_results {
            Ok(results) => results,
            Err(err) => {
                tracing::error!(?err);
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let asset_results = if let Some(project_id) = &project_id {
            self.db
                .query(asset_query)
                .bind(("project", project_id.clone()))
                .await
        } else {
            self.db.query(asset_query).await
        };

        let mut asset_results = match asset_results {
            Ok(results) => results,
            Err(err) => {
                tracing::error!(?err);
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let container_results = match container_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                tracing::error!(?err);
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut asset_results = match asset_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                tracing::error!(?err);
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut results = container_results;
        results.append(&mut asset_results);
        results.sort_by(|ra, rb| ra.score.partial_cmp(&rb.score).unwrap());

        let results = results
            .into_iter()
            .map(|record| {
                let key = record.id.key();
                let key_str = key.to_string();
                let mut rid = key_str.chars();
                rid.next(); // strip key delimeters
                rid.next_back();
                let rid = rid.as_str();

                ResourceId::from_str(rid).unwrap()
            })
            .collect();

        Self::send_response(tx, Ok(results));
    }

    fn send_response<T>(tx: Tx<T>, value: surrealdb::Result<T>) {
        match tx.send(value) {
            Ok(_) => {}
            Err(_) => tracing::error!("could not send response"),
        }
    }
}

pub mod project {
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

        pub async fn project_record_id_from_path(
            &self,
            path: PathBuf,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .query("SELECT id FROM project WHERE path=type::string($path)")
                .bind(("path", path))
                .await?;

            let record: Option<IdRecord> = response.take(0)?;
            Ok(record.map(|record| record.id))
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

        /// Remove a project and all its resources.
        pub async fn remove_project(&self, project: PathBuf) -> surrealdb::Result<()> {
            let mut project_id = self
                .db
                .query("SELECT id FROM project WHERE path=type::string($path)")
                .bind(("path", project))
                .await?;

            let Some(project_id) = project_id.take::<Option<IdRecord>>(0)? else {
                todo!();
            };

            self.db
                .query(
                    "DELETE
                        project_properties, \
                        project_settings, \
                        container, \
                        container_properties, \
                        container_settings, \
                        asset, \
                        flag \
                    WHERE _project=type::thing($project)",
                )
                .bind(("project", project_id.id.clone()))
                .await?;

            self.db.delete::<Option<IdRecord>>(project_id.id).await?;

            Ok(())
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

pub mod container {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, ffi::OsString, path::PathBuf};
    use syre_core as core;
    use syre_local as local;

    impl Store {
        pub async fn insert_container(
            &self,
            project_id: surrealdb::RecordId,
            name: OsString,
            path: PathBuf,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = self
                .db
                .create::<Option<IdRecord>>("container")
                .content(Record {
                    _project: project_id,
                    name: name.to_string_lossy().to_string(),
                    path,
                })
                .await?
                .unwrap();

            Ok(record.id)
        }

        pub async fn container_record_id_from_path(
            &self,
            project: surrealdb::RecordId,
            path: PathBuf,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .query(
                    "
                    SELECT id FROM container \
                    WHERE _project=type::thing($project) AND path=type::string($path)",
                )
                .bind(("project", project))
                .bind(("path", path))
                .await?;

            let record: Option<IdRecord> = response.take(0)?;
            Ok(record.map(|record| record.id))
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

        pub async fn remove_container_properties_by_path(
            &self,
            project: PathBuf,
            container: PathBuf,
        ) -> surrealdb::Result<()> {
            let project_id = self
                .project_record_id_from_path(project)
                .await
                .unwrap()
                .unwrap();

            let mut record_id = self
                .query(
                    "SELECT properties as id FROM container \
                    WHERE _project=type::thing($project) AND path=type::string($path)",
                )
                .bind(("project", project_id))
                .bind(("path", container))
                .await?;

            let record_id: Option<IdRecord> = record_id.take(0)?;
            let record_id = record_id.unwrap();
            self.delete::<Option<IdRecord>>(record_id.id).await?;

            Ok(())
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

        pub async fn remove_container_settings_by_path(
            &self,
            project: PathBuf,
            container: PathBuf,
        ) -> surrealdb::Result<()> {
            let project_id = self
                .project_record_id_from_path(project)
                .await
                .unwrap()
                .unwrap();

            let mut record_id = self
                .query(
                    "SELECT settings as id FROM container \
                    WHERE _project=type::thing($project) AND path=type::string($path)",
                )
                .bind(("project", project_id))
                .bind(("path", container))
                .await?;

            let record_id: Option<IdRecord> = record_id.take(0)?;
            let record_id = record_id.unwrap();
            self.delete::<Option<IdRecord>>(record_id.id).await?;

            Ok(())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Record {
        _project: surrealdb::RecordId,
        name: String,
        path: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PropertiesRecord {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
        name: String,
        kind: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
        metadata: HashMap<String, core::types::Value>,
    }

    #[derive(Serialize)]
    pub struct SettingsRecord {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
        creator: Option<core::types::UserId>,

        #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
        created: DateTime<Utc>,
    }
}

pub mod asset {
    use super::{cast, IdRecord, Store};
    use chrono::{DateTime, Utc};
    use serde::Serialize;
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
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

        pub async fn asset_record_id_from_path(
            &self,
            container: surrealdb::RecordId,
            path: PathBuf,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .query(
                    "
                    SELECT id FROM asset \
                    WHERE _container=type::thing($container) AND path=type::string($path)",
                )
                .bind(("container", container))
                .bind(("path", path))
                .await?;

            let record: Option<IdRecord> = response.take(0)?;
            Ok(record.map(|record| record.id))
        }

        /// Remove all assets associated with a container.
        ///
        /// # Returns
        /// Number of records removed.
        pub async fn remove_container_assets_by_path(
            &self,
            project: PathBuf,
            container: PathBuf,
        ) -> surrealdb::Result<usize> {
            let project_id = self.project_record_id_from_path(project).await?.unwrap();
            let container_id = self
                .container_record_id_from_path(project_id, container)
                .await?
                .unwrap();

            let mut result = self
                .query("count(DELETE asset WHERE _container=type::thing($container) RETURN BEFORE)")
                .bind(("container", container_id))
                .await?;

            let removed = result.take::<Option<usize>>(0)?;
            Ok(removed.unwrap())
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

pub mod flag {
    use super::{IdRecord, Store};
    use serde::Serialize;
    use std::path::PathBuf;
    use syre_local as local;

    impl Store {
        pub async fn insert_flag(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            resource: Option<surrealdb::RecordId>,
            path: PathBuf,
            flag: &local::project::Flag,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            let record = Record {
                _project: project_id,
                _container: container_id,
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

        /// Remove all flags of a container.
        ///
        /// # Returns
        /// Number of records removed.
        pub async fn remove_container_flags_by_path(
            &self,
            project: PathBuf,
            container: PathBuf,
        ) -> surrealdb::Result<usize> {
            let project_id = self.project_record_id_from_path(project).await?.unwrap();
            let container_id = self
                .container_record_id_from_path(project_id, container)
                .await?
                .unwrap();

            let mut result = self
                .query("count(DELETE flag WHERE _container=type::thing($container) RETURN BEFORE)")
                .bind(("container", container_id))
                .await?;

            let removed = result.take::<Option<usize>>(0)?;
            Ok(removed.unwrap())
        }
    }

    #[derive(Serialize)]
    struct Record {
        _project: surrealdb::RecordId,
        _container: surrealdb::RecordId,
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

pub mod cast {
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
