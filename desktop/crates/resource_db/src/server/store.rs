use crate::command::{AssetSearchResult, SearchResult};
use std::{path::PathBuf, str::FromStr};
use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
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

const DEFINE_TABLE_CONTAINER_PROPERTIES: &str = r#"
DEFINE TABLE container_properties SCHEMAFULL;

DEFINE FIELD _project       ON TABLE container_properties TYPE record<project>;
DEFINE FIELD _container     ON TABLE container_properties TYPE record<container>;
DEFINE FIELD name           ON TABLE container_properties TYPE string;
DEFINE FIELD kind           ON TABLE container_properties TYPE option<string>;
DEFINE FIELD description    ON TABLE container_properties TYPE option<string>;
DEFINE FIELD tags           ON TABLE container_properties TYPE set<string>;
DEFINE FIELD metadata       ON TABLE container_properties FLEXIBLE TYPE object;

DEFINE FIELD metadata_search                    ON TABLE container_properties TYPE string;
DEFINE EVENT container_update_metadata_search   ON TABLE container_properties
    WHEN ($event = "CREATE" || $event = "UPDATE") && $after.metadata != $before.metadata
    THEN {
        LET $search = $value.metadata.entries().fold(
            "",
            |$search, $field| string::concat(
                $search,
                string::concat($field[0], ":", $field[1]),
                " "
            )
        );
        UPDATE $value.id SET metadata_search = $search; 
    }
"#;

const DEFINE_TABLE_CONTAINER_SETTINGS: &str = "
DEFINE TABLE container_settings SCHEMAFULL;

DEFINE FIELD _project   ON TABLE container_settings TYPE record<project>;
DEFINE FIELD _container ON TABLE container_settings TYPE record<container>;
DEFINE FIELD creator    ON TABLE container_settings TYPE option<{ Email: string } | { Id: bytes }>;
DEFINE FIELD created    ON TABLE container_settings TYPE datetime;
";

const DEFINE_TABLE_ASSET: &str = r#"
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

DEFINE FIELD metadata_search                ON TABLE asset TYPE string;
DEFINE EVENT asset_update_metadata_search   ON TABLE asset
    WHEN ($event = "CREATE" || $event = "UPDATE") && $after.metadata != $before.metadata
    THEN {
        LET $search = $value.metadata.entries().fold(
            "", 
            |$search, $field| string::concat(
                $search,
                string::concat($field[0], ":", $field[1]),
                " "
            )
        );
        UPDATE $value.id SET metadata_search = $search; 
    }
"#;

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

DEFINE INDEX container_name         ON container_properties COLUMNS name FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_kind         ON container_properties COLUMNS kind FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_description  ON container_properties COLUMNS description FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_tags         ON container_properties COLUMNS tags FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX container_metadata     ON container_properties COLUMNS metadata_search FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);

DEFINE INDEX asset_name         ON asset COLUMNS name FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_kind         ON asset COLUMNS kind FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_description  ON asset COLUMNS description FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_tags         ON asset COLUMNS tags FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_metadata     ON asset COLUMNS metadata_search FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
DEFINE INDEX asset_path         ON asset COLUMNS path FULLTEXT ANALYZER properties_analyzer BM25(1.2, 0.75);
";

#[derive(derive_more::Deref, Clone)]
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
        tx: Tx<SearchResult>,
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
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        "could not retrieve project record id from path, aborting search: {err:?}"
                    );
                    Self::send_response(tx, Err(err));
                    return;
                }
            };

            if project_id.is_none() {
                Self::send_response(tx, Ok(SearchResult::empty()));
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
                OR metadata_search @4@ '{query}'
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
                OR metadata_search @4@ '{query}'
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
                #[cfg(feature = "tracing")]
                tracing::warn!("container query error, aborting search: {err:?}");
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
                #[cfg(feature = "tracing")]
                tracing::warn!("asset query error, aborting search: {err:?}");
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut container_results = match container_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not take container results, aborting search: {err:?}");
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut asset_results = match asset_results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not take asset results, aborting search: {err:?}");
                Self::send_response(tx, Err(err));
                return;
            }
        };

        container_results.sort_by(|ra, rb| rb.score.partial_cmp(&ra.score).unwrap());
        asset_results.sort_by(|ra, rb| rb.score.partial_cmp(&ra.score).unwrap());

        let (containers, container_scores): (Vec<_>, Vec<_>) = container_results
            .into_iter()
            .map(|record| {
                let rid = record_id_key_to_resource_id(record.id.key()).unwrap();
                (rid, record.score)
            })
            .unzip();

        let (assets, asset_scores): (Vec<_>, Vec<_>) = asset_results
            .into_iter()
            .map(|record| {
                let rid = record_id_key_to_resource_id(record.id.key()).unwrap();
                (rid, record.score)
            })
            .unzip();

        let results = SearchResult::new(containers, assets, container_scores, asset_scores);

        Self::send_response(tx, Ok(results));
    }

    pub async fn handle_search_assets(
        &self,
        tx: Tx<AssetSearchResult>,
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
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        "could not obtain project record id from path, aborting search: {err:?}"
                    );
                    Self::send_response(tx, Err(err));
                    return;
                }
            };

            if project_id.is_none() {
                Self::send_response(tx, Ok(AssetSearchResult::empty()));
                return;
            }

            (project_id, "_project=type::thing($project) AND")
        } else {
            (None, "")
        };

        let query = format!(
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
                OR metadata_search @4@ '{query}'
                OR path @5@ '{query}'
            )
            ORDER BY score DESC"
        );

        let results = if let Some(project_id) = &project_id {
            self.db
                .query(query)
                .bind(("project", project_id.clone()))
                .await
        } else {
            self.db.query(query).await
        };

        let mut results = match results {
            Ok(results) => results,
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("query error, aborting search: {err:?}");
                Self::send_response(tx, Err(err));
                return;
            }
        };

        let mut results = match results.take::<Vec<Record>>(0) {
            Ok(results) => results,
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not take result, aborting search: {err:?}");
                Self::send_response(tx, Err(err));
                return;
            }
        };

        results.sort_by(|ra, rb| rb.score.partial_cmp(&ra.score).unwrap());

        let (assets, scores): (Vec<_>, Vec<_>) = results
            .into_iter()
            .map(|record| {
                let rid = record_id_key_to_resource_id(record.id.key()).unwrap();
                (rid, record.score)
            })
            .unzip();

        let results = AssetSearchResult::new(assets, scores);
        Self::send_response(tx, Ok(results));
    }

    fn send_response<T>(tx: Tx<T>, value: surrealdb::Result<T>) {
        match tx.send(value) {
            Ok(_) => {}
            Err(_) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send response")
            }
        }
    }
}

pub mod project {
    use super::{IdRecord, Store, cast};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use syre_core as core;
    use syre_local as local;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Record {
        path: PathBuf,
    }

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

        /// Set a project's path.
        ///
        /// # Arguments
        /// + `project`: Project's current path.
        /// + `path`: New path to set.
        ///
        /// # Returns
        /// `None` if the project could not be found.
        pub async fn project_set_path(
            &self,
            project: PathBuf,
            path: PathBuf,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .db
                .query(
                    "UPDATE project SET path=type::string($path) \
                WEHRE path=type::string($project)",
                )
                .bind(("project", project))
                .bind(("path", path))
                .await?;

            let record: Option<IdRecord> = response.take(0)?;
            Ok(record.map(|record| record.id))
        }

        /// Inserts project properties keyed by the project's resource id.
        ///
        /// # Panics
        /// If the insertion or subsequent query to obtain the id fails.
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_project_properties(
            &self,
            project_id: surrealdb::RecordId,
            project: core::project::Project,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            #[derive(Debug, Serialize, Deserialize)]
            pub struct Record {
                _project: surrealdb::RecordId,
                name: String,
                description: Option<String>,
                data_root: PathBuf,
                analysis_root: Option<PathBuf>,
            }

            let id = project.rid().clone();
            let core::project::Project {
                name,
                description,
                data_root,
                analysis_root,
                ..
            } = project;

            let record = Record {
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
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_project_settings(
            &self,
            project_id: surrealdb::RecordId,
            creator: Option<core::types::UserId>,
            created: DateTime<Utc>,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            #[derive(Debug, Serialize, Deserialize)]
            pub struct Record {
                _project: surrealdb::RecordId,
                creator: Option<core::types::UserId>,

                #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let record = self
                .db
                .create::<Option<IdRecord>>("project_settings")
                .content(Record {
                    _project: project_id,
                    creator,
                    created,
                })
                .await?
                .unwrap();

            Ok(record.id)
        }

        /// Remove a project and all its resources.
        pub async fn remove_project_by_path(&self, project: PathBuf) -> surrealdb::Result<()> {
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

        pub async fn project_properties_update(
            &self,
            project: surrealdb::RecordId,
            update: core::project::Project,
        ) -> surrealdb::Result<()> {
            #[derive(Debug, Serialize, Deserialize)]
            struct Update {
                name: String,
                description: Option<String>,
                data_root: PathBuf,
                analysis_root: Option<PathBuf>,
            }

            let mut record_id = self
                .db
                .query("SELECT id FROM project_properties WHERE _project=type::thing($project)")
                .bind(("project", project))
                .await?;

            let record_id: Option<IdRecord> = record_id.take(0)?;
            let record_id = record_id.unwrap();

            let core::project::Project {
                name,
                description,
                data_root,
                analysis_root,
                ..
            } = update;

            self.db
                .update::<Option<IdRecord>>(record_id.id)
                .merge(Update {
                    name,
                    description,
                    data_root,
                    analysis_root,
                })
                .await?;

            Ok(())
        }

        pub async fn project_properties_remove(
            &self,
            project: surrealdb::RecordId,
        ) -> surrealdb::Result<()> {
            self.db
                .query("DELETE project_properties WHERE _project=type::thing($project)")
                .bind(("project", project))
                .await?;

            Ok(())
        }

        pub async fn project_settings_update(
            &self,
            project: surrealdb::RecordId,
            update: local::project::config::Settings,
        ) -> surrealdb::Result<()> {
            #[derive(Debug, Serialize, Deserialize)]
            struct Update {
                creator: Option<core::types::UserId>,

                #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let mut record_id = self
                .db
                .query("SELECT id FROM project_settings WHERE _project=type::thing($project)")
                .bind(("project", project))
                .await?;

            let record_id: Option<IdRecord> = record_id.take(0)?;
            let record_id = record_id.unwrap();

            let local::project::config::Settings {
                created, creator, ..
            } = update;

            self.db
                .update::<Option<IdRecord>>(record_id.id)
                .merge(Update { creator, created })
                .await?;

            Ok(())
        }

        pub async fn project_settings_remove(
            &self,
            project: surrealdb::RecordId,
        ) -> surrealdb::Result<()> {
            self.db
                .query("DELETE project_settings WHERE _project=type::thing($project)")
                .bind(("project", project))
                .await?;

            Ok(())
        }
    }
}

pub mod container {
    use super::{IdRecord, Store, cast};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, ffi::OsString, path::PathBuf};
    use syre_core as core;
    use syre_local as local;

    #[derive(Serialize, Deserialize)]
    pub struct Record {
        _project: surrealdb::RecordId,
        name: String,
        path: PathBuf,
    }

    impl Store {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
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

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
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

        /// Sets the given container's name.
        ///
        /// # Returns
        /// `None` if a container with the given path and project is not found.
        ///
        /// # Notes
        /// + This does not change the container's path.
        ///     See `container_set_path_and_update_children`.
        pub async fn container_set_name(
            &self,
            project: surrealdb::RecordId,
            container: PathBuf,
            name: OsString,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .query(
                    "UPDATE container \
                    SET name=type::string($name)
                    WHERE _project=type::thing($project) AND path=type::string($container))",
                )
                .bind(("project", project))
                .bind(("container", container))
                .bind(("name", name))
                .await?;

            let record: Option<IdRecord> = response.take(0)?;
            Ok(record.map(|record| record.id))
        }

        /// Sets the given container's path and updated all children paths.
        ///
        /// # Arguments
        /// + `container`: Current path.
        /// + `path`: Update path.
        ///
        /// # Returns
        /// All updated records.
        /// If a container with the given path is not found an empty `Vec` is returned.
        ///
        /// # Notes
        /// + This does not change the container's name.
        ///     See `container_set_name`.
        pub async fn container_set_path_and_update_children(
            &self,
            project: surrealdb::RecordId,
            container: PathBuf,
            path: PathBuf,
        ) -> surrealdb::Result<Vec<surrealdb::RecordId>> {
            let container_len = container.as_os_str().to_string_lossy().chars().count();
            let mut response = self
            .query(
                "UPDATE container \
                SET path=string::concat( \
                    type::string($path), \
                    string::slice( \
                        path, \
                        type::int($container_len)
                    ) \
                )
                WHERE _project=type::thing($project) AND string::starts_with(path, type::string($container))",
            )
            .bind(("project", project))
            .bind(("container", container))
            .bind(("path", path))
            .bind(("container_len", container_len))
            .await?;

            let record: Vec<IdRecord> = response.take(0)?;
            Ok(record.into_iter().map(|record| record.id).collect())
        }

        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(level = "trace", skip(self, project_id, container_id, properties))
        )]
        pub async fn insert_container_properties(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            id: core::types::ResourceId,
            properties: core::project::ContainerProperties,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            #[derive(Serialize, Deserialize, Debug)]
            struct Record {
                _project: surrealdb::RecordId,
                _container: surrealdb::RecordId,
                name: String,
                kind: Option<String>,
                description: Option<String>,
                tags: Vec<String>,
                metadata: HashMap<String, core::types::Value>,
                metadata_search: String,
            }

            let core::project::ContainerProperties {
                name,
                kind,
                description,
                tags,
                metadata,
            } = properties;

            let record = Record {
                _project: project_id,
                _container: container_id,
                name,
                kind,
                description,
                tags,
                metadata,
                metadata_search: "".to_string(),
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

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_container_settings(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            settings: local::project::container::Settings,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            #[derive(Serialize)]
            struct Record {
                _project: surrealdb::RecordId,
                _container: surrealdb::RecordId,
                creator: Option<core::types::UserId>,

                #[serde(serialize_with = "cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let local::project::container::Settings {
                creator, created, ..
            } = settings;

            let record = Record {
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

        /// Remove a container, all its children, and all related resources.
        /// i.e. From `container`, `container_properties`, `container_settings`, and `asset`.
        ///
        /// # Not implemented
        /// + Remove associated flags.
        pub async fn remove_subgraph_by_path(
            &self,
            project: surrealdb::RecordId,
            root: PathBuf,
        ) -> surrealdb::Result<()> {
            // TODO: Remove all associated flags
            self.db.query(
                r#"
                $containers = SELECT id FROM container 
                    WHERE _project=type::thing($project) AND string::starts_with(path, type::string($root));

                BEGIN TRANSACTION;
                FOR $container in $containers {
                    DELETE $container;
                    DELETE container_properties WHERE _container=$container;
                    DELETE container_settings WHERE _container=$container;
                    DELETE asset WHERE _container=$container;
                };
                COMMIT TRANSACTION;
                "#
            )
            .bind(("project", project))
            .bind(("root", root))
            .await?;

            Ok(())
        }
    }
}

pub mod asset {
    use super::{IdRecord, Store, cast};
    use chrono::{DateTime, Utc};
    use serde::Serialize;
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
    use syre_project_daemon as project_daemon;

    impl Store {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_asset(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            asset: project_daemon::state::Asset,
        ) -> surrealdb::Result<surrealdb::RecordId> {
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

                metadata_search: String,
            }

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
                metadata_search: "".to_string(),
            };

            let record = self
                .db
                .create::<Option<IdRecord>>(("asset", rid.to_string()))
                .content(record)
                .await?
                .unwrap();

            Ok(record.id)
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_assets(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            assets: Vec<project_daemon::state::Asset>,
        ) -> surrealdb::Result<Vec<surrealdb::RecordId>> {
            #[derive(Serialize)]
            struct Record {
                id: surrealdb::RecordId,
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

                metadata_search: String,
            }

            let asset_to_record = move |asset: project_daemon::state::Asset| {
                let fs_resource_present = asset.is_present();
                let rid = asset.rid().clone().to_string();
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

                Record {
                    id: ("asset", rid).into(),
                    _project: project_id.clone(),
                    _container: container_id.clone(),
                    name,
                    kind,
                    description,
                    tags,
                    metadata,
                    path,
                    fs_resource_present,
                    creator,
                    created,
                    metadata_search: "".to_string(),
                }
            };

            let records = assets.into_iter().map(asset_to_record).collect::<Vec<_>>();
            let records = self
                .db
                .insert::<Vec<IdRecord>>("asset")
                .content(records)
                .await?
                .into_iter()
                .map(|record| record.id)
                .collect::<Vec<_>>();

            Ok(records)
        }

        pub async fn asset_record_id_from_path(
            &self,
            container: surrealdb::RecordId,
            path: PathBuf,
        ) -> surrealdb::Result<Option<surrealdb::RecordId>> {
            let mut response = self
                .query(
                    "SELECT id FROM asset \
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
}

pub mod flag {
    use super::{IdRecord, Store};
    use serde::Serialize;
    use std::path::PathBuf;
    use syre_local as local;

    impl Store {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub async fn insert_flag(
            &self,
            project_id: surrealdb::RecordId,
            container_id: surrealdb::RecordId,
            resource: Option<surrealdb::RecordId>,
            path: PathBuf,
            flag: &local::project::Flag,
        ) -> surrealdb::Result<surrealdb::RecordId> {
            #[derive(Serialize)]
            struct Record {
                _project: surrealdb::RecordId,
                _container: surrealdb::RecordId,
                resource: Option<surrealdb::RecordId>,
                path: PathBuf,
                severity: local::project::flag::Severity,
                message: String,
            }

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

fn record_id_key_to_resource_id(
    key: &surrealdb::RecordIdKey,
) -> Result<ResourceId, <ResourceId as FromStr>::Err> {
    let key_str = key.to_string();
    let mut rid = key_str.chars();
    rid.next(); // strip key delimeters
    rid.next_back();
    let rid = rid.as_str();

    ResourceId::from_str(rid)
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
        Into::<surrealdb::Datetime>::into(*t).serialize(s)
    }
}
