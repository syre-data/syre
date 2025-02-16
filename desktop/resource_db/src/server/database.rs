use super::{store, Store};
use crate::Command;
use std::{path::PathBuf, thread};
use syre_core::types::ResourceId;
use syre_project_watcher as project_watcher;
use tokio::sync::mpsc;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum ResourceKind {
    Container,
    Asset,
}

pub struct Builder {
    query_rx: mpsc::UnboundedReceiver<Command>,
}

impl Builder {
    pub fn new(query_rx: mpsc::UnboundedReceiver<Command>) -> Self {
        Self { query_rx }
    }

    #[tokio::main]
    pub async fn run(self) -> surrealdb::Result<()> {
        let (project_event_tx, project_event_rx) = mpsc::unbounded_channel();
        let project_actor = super::project_watcher_actor::Builder::new(project_event_tx);
        thread::Builder::new()
            .name("syre desktop resource db project watcher actor".to_string())
            .spawn(move || {
                project_actor.run();
            })
            .expect("could not launch project watcher actor");

        let mut db = Database::new(self.query_rx, project_event_rx).await?;
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
    async fn new(
        query_rx: mpsc::UnboundedReceiver<Command>,
        project_event_rx: mpsc::UnboundedReceiver<Vec<project_watcher::Update>>,
    ) -> surrealdb::Result<Self> {
        Ok(Self {
            pw_client: project_watcher::Client::new(),
            store: Store::new().await?,
            query_rx,
            project_event_rx,
        })
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
                        Command::Search { tx,  query } => self.store.handle_search(tx, query, None).await,
                        Command::SearchProject { tx, project, query } => self.store.handle_search(tx, query, Some(project)).await,
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
                .insert_container(project_id.clone(), container.name().clone(), path)
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
                            .insert_flag(
                                project_id.clone(),
                                container_record_id.clone(),
                                resource.clone(),
                                path.clone(),
                                flag,
                            )
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
        root: PathBuf,
    ) {
        let root = if idx == 0 {
            root
        } else {
            let node = graph.nodes.get(idx).unwrap();
            root.join(node.name())
        };

        paths[idx] = root.clone();
        for child in graph.children[idx].iter() {
            inner(graph, paths, *child, root.clone());
        }
    }

    let mut paths = vec![PathBuf::new(); graph.nodes.len()];
    inner(
        graph,
        &mut paths,
        0,
        PathBuf::from_iter(std::iter::once(std::path::Component::RootDir)),
    );
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
    use super::{store, Database};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, path::PathBuf};
    use syre_core as core;
    use syre_local as local;
    use syre_project_watcher::{event, state, Update};

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
                event::Project::Asset { .. } => self.handle_event_update_project_asset(event).await,
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
                event::Container::Properties(_) => {
                    self.handle_event_update_project_container_properties(event)
                        .await
                }
                event::Container::Settings(_) => {
                    self.handle_event_update_project_container_settings(event)
                        .await
                }
                event::Container::Assets(_) => {
                    self.handle_event_update_project_container_assets(event)
                        .await
                }
                event::Container::Flags(_) => {
                    self.handle_event_update_project_container_flags(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_container_properties(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Container {
                        update: event::Container::Properties(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::DataResource::Created(_) => {
                    self.handle_event_update_project_container_properties_created(event)
                        .await
                }
                event::DataResource::Removed => {
                    self.handle_event_update_project_container_properties_removed(event)
                        .await
                }
                event::DataResource::Corrupted(_) => {
                    self.handle_event_update_project_container_properties_corrupted(event)
                        .await
                }
                event::DataResource::Repaired(_) => {
                    self.handle_event_update_project_container_properties_repaired(event)
                        .await
                }
                event::DataResource::Modified(_) => {
                    self.handle_event_update_project_container_properties_modified(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_container_properties_created(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Properties(event::DataResource::Created(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let state::DataResource::Ok(local::project::container::StoredProperties {
                rid,
                properties,
                ..
            }) = update
            else {
                return;
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            self.store
                .insert_container_properties(
                    project_id,
                    container_id,
                    rid.clone(),
                    properties.clone(),
                )
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_properties_removed(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Properties(event::DataResource::Removed),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_properties_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_properties_corrupted(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Properties(event::DataResource::Corrupted(_)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_properties_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_properties_repaired(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Properties(event::DataResource::Repaired(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let local::project::container::StoredProperties {
                rid, properties, ..
            } = update;

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            self.store
                .insert_container_properties(
                    project_id,
                    container_id,
                    rid.clone(),
                    properties.clone(),
                )
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_properties_modified(&self, event: Update) {
            #[derive(Serialize)]
            struct Update {
                name: String,
                kind: Option<String>,
                description: Option<String>,
                tags: Vec<String>,
                metadata: HashMap<String, core::types::Value>,
            }

            let event::UpdateKind::Project {
                update:
                    event::Project::Container {
                        update: event::Container::Properties(event::DataResource::Modified(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let local::project::container::StoredProperties {
                rid, properties, ..
            } = update;

            let core::project::ContainerProperties {
                name,
                kind,
                description,
                tags,
                metadata,
            } = properties.clone();

            self.store
                .update::<Option<store::IdRecord>>(("container_properties", rid.to_string()))
                .merge(Update {
                    name,
                    kind,
                    description,
                    tags,
                    metadata,
                })
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_settings(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Container {
                        update: event::Container::Settings(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::DataResource::Created(_) => {
                    self.handle_event_update_project_container_settings_created(event)
                        .await
                }
                event::DataResource::Removed => {
                    self.handle_event_update_project_container_settings_removed(event)
                        .await
                }
                event::DataResource::Corrupted(io_serde) => {
                    self.handle_event_update_project_container_settings_corrupted(event)
                        .await
                }
                event::DataResource::Repaired(_) => {
                    self.handle_event_update_project_container_settings_repaired(event)
                        .await
                }
                event::DataResource::Modified(_) => {
                    self.handle_event_update_project_container_settings_modified(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_container_settings_created(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Settings(event::DataResource::Created(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let state::DataResource::Ok(update) = update else {
                return;
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            self.store
                .insert_container_settings(project_id, container_id, update.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_settings_removed(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Settings(event::DataResource::Removed),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_settings_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_settings_corrupted(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Settings(event::DataResource::Corrupted(_)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_settings_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_settings_repaired(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Settings(event::DataResource::Repaired(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            self.store
                .insert_container_settings(project_id, container_id, update.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_settings_modified(&self, event: Update) {
            #[derive(Serialize)]
            struct Update {
                creator: Option<core::types::UserId>,

                #[serde(serialize_with = "store::cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Settings(event::DataResource::Modified(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let mut settings_id = self
                .store
                .query(
                    "SELECT settings as id FROM container \
                    WHERE _project=type::thing($project) AND path=type::string($path)",
                )
                .bind(("project", project_id))
                .bind(("path", container_path.clone()))
                .await
                .unwrap();

            let settings_id: Option<store::IdRecord> = settings_id.take(0).unwrap();
            let settings_id = settings_id.unwrap();

            let update = Update {
                creator: update.creator.clone(),
                created: update.created.clone(),
            };

            self.store
                .update::<Option<store::IdRecord>>(settings_id.id)
                .merge(update)
                .await
                .unwrap()
                .unwrap();
        }

        async fn handle_event_update_project_container_assets(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Container {
                        update: event::Container::Assets(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::DataResource::Created(_) => {
                    self.handle_event_update_project_container_assets_created(event)
                        .await
                }
                event::DataResource::Removed => {
                    self.handle_event_update_project_container_assets_removed(event)
                        .await
                }
                event::DataResource::Corrupted(_) => {
                    self.handle_event_update_project_container_assets_corrupted(event)
                        .await
                }
                event::DataResource::Repaired(_) => {
                    self.handle_event_update_project_container_assets_repaired(event)
                        .await
                }
                event::DataResource::Modified(_) => {
                    self.handle_event_update_project_container_assets_modified(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_container_assets_created(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Assets(event::DataResource::Created(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let state::DataResource::Ok(update) = update else {
                return;
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            for asset in update {
                self.store
                    .insert_asset(project_id.clone(), container_id.clone(), asset.clone())
                    .await
                    .unwrap();
            }
        }

        async fn handle_event_update_project_container_assets_removed(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Assets(event::DataResource::Removed),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_assets_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_assets_corrupted(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Assets(event::DataResource::Corrupted(_)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_assets_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_assets_repaired(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Assets(event::DataResource::Repaired(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            for asset in update {
                self.store
                    .insert_asset(project_id.clone(), container_id.clone(), asset.clone())
                    .await
                    .unwrap();
            }
        }

        async fn handle_event_update_project_container_assets_modified(&self, event: Update) {
            #[derive(Deserialize)]
            struct AssetRecord {
                pub id: surrealdb::RecordId,
                pub path: PathBuf,
            }

            #[derive(Serialize)]
            struct UpdateRecord {
                name: Option<String>,
                kind: Option<String>,
                description: Option<String>,
                tags: Vec<String>,
                metadata: HashMap<String, core::types::Value>,
                path: PathBuf,
                fs_resource_present: bool,

                creator: core::types::Creator,

                #[serde(serialize_with = "store::cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Assets(event::DataResource::Modified(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            let mut assets = self
                .store
                .query("SELECT id, path FROM asset WHERE _container=type::thing($container)")
                .bind(("container", container_id.clone()))
                .await
                .unwrap();

            let assets = assets.take::<Vec<AssetRecord>>(0).unwrap();
            let asset_paths = assets.iter().map(|asset| &asset.path).collect::<Vec<_>>();
            let update_paths = update.iter().map(|asset| &asset.path).collect::<Vec<_>>();
            for asset in assets.iter() {
                if !update_paths.contains(&&asset.path) {
                    self.store
                        .delete::<Option<store::IdRecord>>(asset.id.clone())
                        .await
                        .unwrap();
                }
            }

            for asset_update in update {
                if let Some(idx) = asset_paths
                    .iter()
                    .position(|path| &asset_update.path == *path)
                {
                    let update_record = UpdateRecord {
                        name: asset_update.properties.name.clone(),
                        kind: asset_update.properties.kind.clone(),
                        description: asset_update.properties.description.clone(),
                        tags: asset_update.properties.tags.clone(),
                        metadata: asset_update.properties.metadata.clone(),
                        path: asset_update.path.clone(),
                        fs_resource_present: asset_update.is_present(),
                        creator: asset_update.properties.creator.clone(),
                        created: asset_update.properties.created().clone(),
                    };

                    self.store
                        .update::<Option<store::IdRecord>>(assets[idx].id.clone())
                        .merge(update_record)
                        .await
                        .unwrap();
                } else {
                    self.store
                        .insert_asset(
                            project_id.clone(),
                            container_id.clone(),
                            asset_update.clone(),
                        )
                        .await
                        .unwrap();
                }
            }
        }

        async fn handle_event_update_project_container_flags(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Container {
                        update: event::Container::Flags(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::DataResource::Created(_) => {
                    self.handle_event_update_project_container_flags_created(event)
                        .await
                }
                event::DataResource::Removed => {
                    self.handle_event_update_project_container_flags_removed(event)
                        .await
                }
                event::DataResource::Corrupted(_) => {
                    self.handle_event_update_project_container_flags_corrupted(event)
                        .await
                }
                event::DataResource::Repaired(_) => {
                    self.handle_event_update_project_container_flags_repaired(event)
                        .await
                }
                event::DataResource::Modified(_) => {
                    self.handle_event_update_project_container_flags_modified(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_container_flags_created(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Flags(event::DataResource::Created(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let state::DataResource::Ok(update) = update else {
                return;
            };
            let root_dir = PathBuf::from("/");

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            for (path, flags) in update {
                let resource = if *path == root_dir {
                    Some(container_id.clone())
                } else {
                    self.store
                        .asset_record_id_from_path(container_id.clone(), path.clone())
                        .await
                        .unwrap()
                };

                for flag in flags {
                    self.store
                        .insert_flag(
                            project_id.clone(),
                            container_id.clone(),
                            resource.clone(),
                            path.clone(),
                            flag,
                        )
                        .await
                        .unwrap();
                }
            }
        }

        async fn handle_event_update_project_container_flags_removed(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Flags(event::DataResource::Created(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_flags_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_flags_corrupted(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Flags(event::DataResource::Corrupted(_)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .remove_container_flags_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
        }

        async fn handle_event_update_project_container_flags_repaired(&self, event: Update) {
            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Flags(event::DataResource::Repaired(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };
            let root_dir = PathBuf::from("/");

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            for (path, flags) in update {
                let resource = if *path == root_dir {
                    Some(container_id.clone())
                } else {
                    self.store
                        .asset_record_id_from_path(container_id.clone(), path.clone())
                        .await
                        .unwrap()
                };

                for flag in flags {
                    self.store
                        .insert_flag(
                            project_id.clone(),
                            container_id.clone(),
                            resource.clone(),
                            path.clone(),
                            flag,
                        )
                        .await
                        .unwrap();
                }
            }
        }

        async fn handle_event_update_project_container_flags_modified(&self, event: Update) {
            let root_dir = PathBuf::from("/");

            #[derive(Deserialize)]
            struct FlagRecord {
                pub id: surrealdb::RecordId,
                pub path: PathBuf,
            }

            #[derive(Serialize)]
            struct UpdateRecord {
                severity: local::project::flag::Severity,
                message: String,
            }

            let event::UpdateKind::Project {
                path: project_path,
                update:
                    event::Project::Container {
                        path: container_path,
                        update: event::Container::Flags(event::DataResource::Modified(update)),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let project_id = self
                .store
                .project_record_id_from_path(project_path.clone())
                .await
                .unwrap()
                .unwrap();

            let container_id = self
                .store
                .container_record_id_from_path(project_id.clone(), container_path.clone())
                .await
                .unwrap()
                .unwrap();

            self.store
                .remove_container_flags_by_path(project_path.clone(), container_path.clone())
                .await
                .unwrap();
            for (path, flags) in update {
                let resource = if *path == root_dir {
                    Some(container_id.clone())
                } else {
                    self.store
                        .asset_record_id_from_path(container_id.clone(), path.clone())
                        .await
                        .unwrap()
                };

                for flag in flags {
                    self.store
                        .insert_flag(
                            project_id.clone(),
                            container_id.clone(),
                            resource.clone(),
                            path.clone(),
                            flag,
                        )
                        .await
                        .unwrap();
                }
            }
        }

        async fn handle_event_update_project_asset(&self, event: Update) {
            let event::UpdateKind::Project {
                update: event::Project::Asset { update, .. },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            match update {
                event::Asset::FileCreated => {
                    self.handle_event_update_project_asset_file_created(event)
                        .await
                }
                event::Asset::FileRemoved => {
                    self.handle_event_update_project_asset_file_removed(event)
                        .await
                }
                event::Asset::Properties(_) => {
                    self.handle_event_update_project_asset_properties(event)
                        .await
                }
            }
        }

        async fn handle_event_update_project_asset_file_created(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Asset {
                        asset,
                        update: event::Asset::FileCreated,
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .query("UPDATE type::thing($asset) SET fs_resource_present = TRUE")
                .bind(("asset", asset.clone()))
                .await
                .unwrap();
        }

        async fn handle_event_update_project_asset_file_removed(&self, event: Update) {
            let event::UpdateKind::Project {
                update:
                    event::Project::Asset {
                        asset,
                        update: event::Asset::FileRemoved,
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            self.store
                .query("UPDATE type::thing($asset) SET fs_resource_present = FALSE")
                .bind(("asset", asset.clone()))
                .await
                .unwrap();
        }

        async fn handle_event_update_project_asset_properties(&self, event: Update) {
            #[derive(Serialize)]
            struct UpdateRecord {
                name: Option<String>,
                kind: Option<String>,
                description: Option<String>,
                tags: Vec<String>,
                metadata: HashMap<String, core::types::Value>,
                path: PathBuf,
                fs_resource_present: bool,

                creator: core::types::Creator,

                #[serde(serialize_with = "store::cast::chrono_as_sql_datetime")]
                created: DateTime<Utc>,
            }

            let event::UpdateKind::Project {
                update:
                    event::Project::Asset {
                        asset,
                        update: event::Asset::Properties(update),
                        ..
                    },
                ..
            } = event.kind()
            else {
                panic!("invalid event kind");
            };

            let update_record = UpdateRecord {
                name: update.properties.name.clone(),
                kind: update.properties.kind.clone(),
                description: update.properties.description.clone(),
                tags: update.properties.tags.clone(),
                metadata: update.properties.metadata.clone(),
                path: update.path.clone(),
                fs_resource_present: update.is_present(),
                creator: update.properties.creator.clone(),
                created: update.properties.created().clone(),
            };

            self.store
                .update::<Option<store::IdRecord>>(("asset", asset.clone()))
                .merge(update_record)
                .await
                .unwrap();
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
