//! Handle file system events.
use crate::error::{Error, Result};
use crate::events::{
    Asset as AssetUpdate, Container as ContainerUpdate, Project as ProjectUpdate, Update,
};
use crate::server::store::ContainerTree;
use crate::server::Database;
use notify::{self, event::CreateKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::fs;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::error::{Error as LocalError, ProjectError};
use thot_local::project::project;
use thot_local::project::resources::{Asset, Container};

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

        // ignore if container
        if self
            .store
            .get_path_container_canonical(&path)
            .unwrap()
            .is_some()
        {
            return Ok(());
        }

        // handle new
        let ParentChild {
            parent,
            child: container,
        } = self.file_system_init_container(path)?;

        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        let container = self.store.get_container(&container).unwrap();
        self.publish_update(&Update::Project {
            project,
            update: ProjectUpdate::Container(ContainerUpdate::ChildCreated {
                parent,
                container: (*container).clone(),
            }),
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
            if path == analysis_root {
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
        } = match self.file_system_init_asset(path) {
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
            update: ProjectUpdate::Asset(AssetUpdate::Created {
                container: container.rid.clone(),
                asset,
            }),
        })?;

        Ok(())
    }

    /// Initialize a path as a `Container` and add it into the graph.
    ///
    /// # Returns
    /// `ResourceId` of the initialize `Container`.
    #[tracing::instrument(skip(self))]
    fn file_system_init_container(&mut self, path: PathBuf) -> Result<ParentChild> {
        let Some(parent) = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .unwrap()
            .cloned()
        else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "parent `Container` does not exist",
            ))
            .into());
        };

        // init container
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let mut container = Container::new(path);
        container.properties.name = name;
        container.save()?;

        // insert into graph
        let child = container.rid.clone();
        self.store
            .insert_subgraph(&parent, ContainerTree::new(container))?;

        Ok(ParentChild { parent, child })
    }

    fn file_system_init_asset(&mut self, path: PathBuf) -> Result<ParentChild> {
        let container_path = thot_local::project::asset::container_from_path_ancestor(&path)?;
        let container = self
            .store
            .get_path_container_canonical(&container_path)
            .unwrap()
            .cloned()
            .unwrap();

        if let Some(asset) = self.store.get_path_asset_id_canonical(&path).unwrap() {
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
