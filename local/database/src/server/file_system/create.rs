//! Handle file system events.
use crate::error::{Error, Result};
use crate::events::{Asset as AssetUpdate, Graph as GraphUpdate, Update};
use crate::server::Database;
use notify::{self, event::CreateKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::path::PathBuf;
use std::{fs, io};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::error::{Error as LocalError, ProjectError};
use thot_local::graph::{ContainerTreeDuplicator, ContainerTreeLoader, ContainerTreeTransformer};
use thot_local::project::resources::{Asset, Container};
use thot_local::project::{container, project};

impl Database {
    /// Handle [`notify::event::CreateKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_create(&mut self, event: DebouncedEvent) -> Result {
        tracing::debug!(?event);
        let EventKind::Create(kind) = event.event.kind else {
            panic!("invalid event kind");
        };

        let [path] = &event.event.paths[..] else {
            panic!("invalid paths");
        };

        if path.components().any(|seg| seg.as_os_str() == ".thot") {
            return Ok(());
        }

        let path = path.clone();
        match kind {
            CreateKind::File => self.handle_create_file(path),
            CreateKind::Folder => self.handle_create_folder(path),
            CreateKind::Any => {
                if path.is_file() {
                    self.handle_create_file(path)
                } else if path.is_dir() {
                    self.handle_create_folder(path)
                } else {
                    tracing::debug!("unknown path resource `{:?}`", path);
                    return Ok(());
                }
            }

            CreateKind::Other => {
                tracing::debug!("other {:?}", event);
                todo!();
            }
        }
    }

    fn handle_create_folder(&mut self, path: PathBuf) -> Result {
        // ignore graph root
        let project_path = project::project_root_path(&path)?;
        let Some(project) = self
            .store
            .get_path_project_canonical(&project_path)
            .unwrap()
        else {
            return Err(LocalError::ProjectError(ProjectError::PathNotInProject(path)).into());
        };

        let Some(project) = self.store.get_project(project) else {
            return Err(Error::DatabaseError("Project not loaded".into()));
        };

        if let Some(data_root) = project.data_root.as_ref() {
            let path = fs::canonicalize(&path).unwrap();
            let project_path = fs::canonicalize(&project_path).unwrap();
            let data_root = project_path.join(data_root);
            if path == data_root {
                return Ok(());
            }
        };

        // ignore if registered container
        // assume registration has already been handled
        if self
            .store
            .get_path_container_canonical(&path)
            .unwrap()
            .is_some()
        {
            return Ok(());
        }

        // handle if unregistered container
        match ContainerTreeLoader::load(&path) {
            Ok(graph) => {
                // existing container was copied
                // update resource ids and register
                let Some(loaded_container) = self.store.get_container(graph.root()) else {
                    // container not loaded
                    return self.file_system_handle_subtree_unknown(graph);
                };

                if loaded_container.base_path().exists() {
                    // container copied
                    return self.file_system_handle_subtree_copied(graph);
                }

                // container moved
                return self.file_system_handle_subtree_moved(project.rid.clone(), graph);
            }

            Err(LocalError::CoreError(CoreError::IoError(err)))
                if err.kind() == io::ErrorKind::NotFound => {}

            Err(err) => return Err(err.into()),
        }

        // handle new
        let ParentChild {
            parent,
            child: container,
        } = self.file_system_init_subgraph(path)?;

        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        let graph = self.store.get_container_graph(&container).unwrap();
        let graph = ContainerTreeTransformer::local_to_core(graph);
        self.publish_update(&Update::Project {
            project,
            update: GraphUpdate::Created { parent, graph }.into(),
        })?;

        Ok(())
    }

    fn handle_create_file(&mut self, path: PathBuf) -> Result {
        // ignore analysis root
        let project_path = project::project_root_path(&path)?;
        let Some(project) = self
            .store
            .get_path_project_canonical(&project_path)
            .unwrap()
        else {
            return Err(LocalError::ProjectError(ProjectError::PathNotInProject(path)).into());
        };

        let Some(project) = self.store.get_project(project) else {
            return Err(Error::DatabaseError("Project not loaded".into()));
        };

        if let Some(analysis_root) = project.analysis_root.as_ref() {
            let path = fs::canonicalize(&path).unwrap();
            let project_path = fs::canonicalize(&project_path).unwrap();
            let analysis_root = project_path.join(analysis_root);
            if path.starts_with(analysis_root) {
                return Ok(());
            }
        };

        // ignore if asset
        if self
            .store
            .get_path_asset_id_canonical(&path)
            .unwrap()
            .is_some()
        {
            return Ok(());
        }

        // handle new
        let ParentChild {
            parent: container,
            child: asset,
        } = match self.file_system_asset_init(path) {
            Ok(container_asset) => container_asset,

            Err(Error::CoreError(CoreError::ResourceError(ResourceError::AlreadyExists(_msg)))) => {
                return Ok(())
            }

            Err(err) => return Err(err),
        };

        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        let container = self.store.get_container(&container).unwrap();
        let asset = container.assets.get(&asset).unwrap().clone();

        self.publish_update(&Update::Project {
            project,
            update: AssetUpdate::Created {
                container: container.rid.clone(),
                asset,
            }
            .into(),
        })?;

        Ok(())
    }

    /// Moves an unloaded subtree into a new location.
    /// Registers resources.
    /// Published update.
    fn file_system_handle_subtree_unknown(&mut self, graph: ResourceTree<Container>) -> Result {
        let root = graph.root().clone();
        let path = graph.get(&root).unwrap().base_path();
        let parent = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .unwrap()
            .cloned()
            .unwrap();

        self.store.insert_subgraph(&parent, graph)?;
        let project = self.store.get_container_project(&root).unwrap().clone();
        let graph = self.store.get_container_graph(&root).unwrap();
        let graph = ContainerTreeTransformer::local_to_core(graph);
        self.publish_update(&Update::Project {
            project,
            update: GraphUpdate::Created { parent, graph }.into(),
        })?;

        Ok(())
    }

    /// Duplicates graph.
    /// Registers resources.
    /// Publishes update.
    fn file_system_handle_subtree_copied(&mut self, graph: ResourceTree<Container>) -> Result {
        let mut graph = ContainerTreeDuplicator::duplicate(&graph, graph.root())?;
        let root = graph.root().clone();
        let root_container = graph.get_mut(&root).unwrap();
        root_container.properties.name = root_container
            .base_path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        for container in graph.nodes().values() {
            container.save()?;
        }

        let path = graph.get(&root).unwrap().base_path();
        let parent = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .unwrap()
            .cloned()
            .unwrap();

        self.store.insert_subgraph(&parent, graph)?;

        let project = self.store.get_container_project(&root).unwrap().clone();
        let graph = self.store.get_container_graph(&root).unwrap();
        let graph = ContainerTreeTransformer::local_to_core(graph);
        self.publish_update(&Update::Project {
            project,
            update: GraphUpdate::Created { parent, graph }.into(),
        })?;

        Ok(())
    }

    /// Moves a subtree within or between `Project`s.
    /// Publishes update.
    fn file_system_handle_subtree_moved(
        &mut self,
        project: ResourceId,
        graph: ResourceTree<Container>,
    ) -> Result {
        let container_project = self
            .store
            .get_container_project(graph.root())
            .cloned()
            .unwrap();

        let root = graph.root().clone();
        let path = graph.get(&root).unwrap().base_path();
        let parent = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .unwrap()
            .cloned()
            .unwrap();

        if container_project == project {
            // moved within project
            let project_graph = self.store.get_project_graph_mut(&project).unwrap();
            project_graph.mv(graph.root(), &parent)?;

            self.publish_update(&Update::Project {
                project: project.clone(),
                update: GraphUpdate::Moved { parent, root }.into(),
            })?;
        } else {
            // moved from other project
            self.store.remove_subgraph(graph.root())?;
            self.store.insert_subgraph(&parent, graph)?;

            let graph = self.store.get_container_graph(&root).unwrap();
            let graph = ContainerTreeTransformer::local_to_core(graph);

            self.publish_update(&Update::Project {
                project: container_project.clone(),
                update: GraphUpdate::Removed(graph.clone()).into(),
            })?;

            self.publish_update(&Update::Project {
                project: project.clone(),
                update: GraphUpdate::Created { parent, graph }.into(),
            })?;
        }

        Ok(())
    }

    /// Initialize a path as a  Contaienr tree and insert it into the graph.
    ///
    /// # Returns
    /// `ResourceId` of the graph's root `Container`.
    #[tracing::instrument(skip(self))]
    fn file_system_init_subgraph(&mut self, path: PathBuf) -> Result<ParentChild> {
        let parent = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .unwrap()
            .cloned()
            .unwrap();

        // init graph
        let mut builder = container::InitOptions::init();
        builder.recurse(true);
        builder.with_assets();
        let child = builder.build(&path)?;

        // insert into graph
        let graph = ContainerTreeLoader::load(path)?;
        self.store.insert_subgraph(&parent, graph)?;

        Ok(ParentChild { parent, child })
    }

    fn file_system_asset_init(&mut self, path: PathBuf) -> Result<ParentChild> {
        let container_path = thot_local::project::asset::container_from_path_ancestor(&path)?;
        let container = self
            .store
            .get_path_container_canonical(&container_path)
            .unwrap()
            .cloned()
            .unwrap();

        if let Some(_asset) = self.store.get_path_asset_id_canonical(&path).unwrap() {
            return Err(CoreError::ResourceError(ResourceError::already_exists(
                "path is already an Asset",
            ))
            .into());
        }

        let asset_path = path
            .strip_prefix(container_path.clone())
            .unwrap()
            .to_path_buf();

        let asset = Asset::new(ResourcePath::new(asset_path)?)?;
        let aid = asset.rid.clone();
        self.store.add_asset(asset, container.clone())?;

        Ok(ParentChild {
            parent: container,
            child: aid,
        })
    }
}

struct ParentChild {
    parent: ResourceId,
    child: ResourceId,
}
