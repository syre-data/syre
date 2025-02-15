use super::{store, Store};
use crate::Command;
use std::{path::PathBuf, thread};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use syre_core::types::ResourceId;
use syre_project_watcher as project_watcher;
use tokio::sync::mpsc;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum ResourceKind {
    Container,
    Asset,
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
    pw_client: project_watcher::Client,
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
            pw_client: project_watcher::Client::new(),
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
                        tracing::debug!("`project_event` channel closed");
                        break;
                    };
                    tracing::debug!(?events);

                    self.handle_update_events(events).await;
                }
                cmd = self.query_rx.recv() => {
                    let Some(cmd) = cmd else {
                        tracing::debug!("`query` channel closed");
                        break;
                    };
                    tracing::debug!(?cmd);

                    match cmd {
                        Command::Query { query, tx } => self.store.handle_query(tx, query).await,
                        Command::Search { tx, project, query } => self.store.handle_search(tx, project, query).await,
                    }
                },
            }
        }

        tracing::trace!("shutting down");
    }

    async fn init_state(&self) -> surrealdb::Result<()> {
        let projects = self
            .pw_client
            .state()
            .projects()
            .expect("could not get project states");

        for project in projects.iter() {
            if let Err(err) = self.init_project(project).await {
                tracing::error!("could not load project {:?}: {err:?}", project.path());
            }
        }

        Ok(())
    }

    async fn init_project(
        &self,
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

        let _update: Option<store::project::Record> = self
            .store
            .update(project_record_id.clone())
            .merge(ProjectRecordLinks {
                properties: Some(properties_record_id.clone()),
                settings: settings_record_id,
            })
            .await
            .unwrap();
        assert!(_update.is_some());

        let (containers, assets) = match self
            .init_project_resources(project_record_id.clone(), properties.rid().clone())
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
        project_id: surrealdb::RecordId,
        project: ResourceId,
    ) -> Result<(Vec<surrealdb::RecordId>, Vec<surrealdb::RecordId>), error::ProjectResourcesInit>
    {
        #[derive(serde::Serialize)]
        struct ContainerRecordLinks {
            properties: Option<surrealdb::RecordId>,
            settings: Option<surrealdb::RecordId>,
        }

        let Some((_, _, graph)) = self.pw_client.project().resources(project).unwrap() else {
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

            let _update: Option<store::container::Record> = self
                .store
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

mod project_events {
    use super::Database;
    use syre_project_watcher::{event, Update};

    impl Database {
        pub(super) async fn handle_update_events(&self, events: Vec<Update>) {
            for event in events {
                tracing::trace!(?event);
                match event.kind() {
                    event::UpdateKind::App(_) => self.handle_event_update_app(event).await,
                    event::UpdateKind::Project { .. } => {
                        self.handle_event_update_project(event).await
                    }
                }
            }
        }

        async fn handle_event_update_app(&self, event: Update) {
            let event::UpdateKind::App(kind) = event.kind() else {
                panic!("invalid event kind");
            };

            match kind {
                event::App::UserManifest(_) => {
                    self.handle_event_update_app_user_manifest(event).await
                }
                event::App::ProjectManifest(_) => {
                    self.handle_event_update_app_project_manifest(event).await
                }
                event::App::LocalConfig(_) => {
                    self.handle_event_update_app_local_config(event).await
                }
            }
        }

        async fn handle_event_update_app_user_manifest(&self, event: Update) {
            let event::UpdateKind::App(event::App::UserManifest(kind)) = event.kind() else {
                panic!("invalid event kind");
            };

            match kind {
                event::UserManifest::Ok(_) => {}
                event::UserManifest::Error => {}
                event::UserManifest::Added(_) => {}
                event::UserManifest::Removed(_) => {}
                event::UserManifest::Updated(_) => {}
            }
        }

        async fn handle_event_update_app_project_manifest(&self, event: Update) {
            let event::UpdateKind::App(event::App::ProjectManifest(kind)) = event.kind() else {
                panic!("invalid event kind");
            };

            match kind {
                event::ProjectManifest::Added(_) => {
                    self.handle_event_update_app_project_manifest_added(event)
                        .await
                }
                event::ProjectManifest::Removed(_) => {
                    self.handle_event_update_app_project_manifest_removed(event)
                        .await
                }
                event::ProjectManifest::Repaired => {
                    self.handle_event_update_app_project_manifest_repaired(event)
                        .await
                }
                event::ProjectManifest::Corrupted => {
                    self.handle_event_update_app_project_manifest_corrupted(event)
                        .await
                }
            }
        }

        async fn handle_event_update_app_project_manifest_added(&self, event: Update) {
            let event::UpdateKind::App(event::App::ProjectManifest(event::ProjectManifest::Added(
                projects,
            ))) = event.kind()
            else {
                panic!("invalid event kind");
            };

            let projects = self
                .pw_client
                .project()
                .get_many(projects.clone())
                .expect("could not get project states");

            for project in projects.iter() {
                if let Err(err) = self.init_project(project).await {
                    tracing::error!("could not load project {:?}: {err:?}", project.path());
                }
            }
        }

        async fn handle_event_update_app_project_manifest_removed(&self, event: Update) {
            let event::UpdateKind::App(event::App::ProjectManifest(
                event::ProjectManifest::Removed(projects),
            )) = event.kind()
            else {
                panic!("invalid event kind");
            };

            for project in projects {
                if let Err(err) = self.store.remove_project(project.clone()).await {
                    tracing::debug!("could not remove project {project:?}: {err:?}");
                };
            }
        }

        async fn handle_event_update_app_project_manifest_corrupted(&self, event: Update) {
            let event::UpdateKind::App(event::App::ProjectManifest(
                event::ProjectManifest::Corrupted,
            )) = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store.clear_all().await;
        }

        async fn handle_event_update_app_project_manifest_repaired(&self, event: Update) {
            let event::UpdateKind::App(event::App::ProjectManifest(
                event::ProjectManifest::Repaired,
            )) = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.init_state().await.unwrap();
        }

        async fn handle_event_update_project(&self, event: Update) {
            let event::UpdateKind::Project { update, .. } = event.kind() else {
                panic!("invalid event kind");
            };

            match update {
                event::Project::FolderRemoved => todo!(),
                event::Project::Moved(_) => todo!(),
                event::Project::Properties(_) => todo!(),
                event::Project::Settings(_) => todo!(),
                event::Project::Analyses(_) => todo!(),
                event::Project::Graph(..) => todo!(),
                event::Project::Container { .. } => {
                    self.handle_event_update_project_container(event).await
                }
                event::Project::Asset { .. } => todo!(),
                event::Project::AssetFile(_) => todo!(),
                event::Project::AnalysisFile(_) => todo!(),
            }
        }

        async fn handle_event_update_project_container(&self, event: Update) {
            let event::UpdateKind::Project {
                update: event::Project::Container { update, .. },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::Container::Properties(data_resource) => {
                    self.handle_event_update_project_container_properties(event)
                        .await
                }
                event::Container::Settings(data_resource) => todo!(),
                event::Container::Assets(data_resource) => todo!(),
                event::Container::Flags(data_resource) => todo!(),
            }
        }

        async fn handle_event_update_project_container_properties(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Properties(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::DataResource::Created(_) => todo!(),
                event::DataResource::Removed => todo!(),
                event::DataResource::Corrupted(_) => todo!(),
                event::DataResource::Repaired(_) => todo!(),
                event::DataResource::Modified(_) => todo!(),
            }
        }

        async fn handle_event_update_app_local_config(&self, event: Update) {
            let event::UpdateKind::App(event::App::LocalConfig(kind)) = event.kind() else {
                panic!("invalid event kind");
            };

            match kind {
                event::LocalConfig::Ok(_) => {}
                event::LocalConfig::Error => {}
                event::LocalConfig::Updated => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
