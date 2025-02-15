use std::{path::PathBuf, str::FromStr};
use surrealdb::{engine::local::Db, Surreal};
use syre_core::types::ResourceId;
use tokio::sync::oneshot;

type Tx<T> = oneshot::Sender<surrealdb::Result<T>>;

#[derive(Debug, serde::Deserialize)]
struct IdRecord {
    pub id: surrealdb::RecordId,
}

#[derive(derive_more::Deref)]
pub struct Store {
    db: Surreal<Db>,
}

impl Store {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
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
        project: Option<PathBuf>,
        query: String,
    ) {
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
            FROM container_properties
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

    #[derive(Serialize, Debug)]
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
