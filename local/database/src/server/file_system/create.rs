//! Handle file system events.
use crate::error::Result;
use crate::events::{Container as ContainerUpdate, Project as ProjectUpdate, Update};
use crate::server::store::ContainerTree;
use crate::server::Database;
use notify::{self, event::CreateKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourceId;
#[cfg(target_os = "windows")]
use thot_local::constants::WINDOWS_UNC_PREFIX;
use thot_local::project::resources::Container;

impl Database {
    /// Handle [`notify::event::CreateKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_create(&mut self, event: DebouncedEvent) -> Result {
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
                    panic!("unknown path resource");
                }
            }

            CreateKind::Other => {
                tracing::debug!("other {:?}", event);
                todo!();
            }
        }
    }

    fn handle_create_folder(&mut self, path: PathBuf) -> Result {
        let ParentChild {
            parent,
            child: container,
        } = self.init_container(path)?;

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
        let asset = self.init_asset(path)?;
        todo!();
    }

    /// Initialize a path as a `Container` and add it into the graph;
    ///
    /// # Returns
    /// `ResourceId` of the initialize `Container`.
    #[tracing::instrument(skip(self))]
    fn init_container(&mut self, path: PathBuf) -> Result<ParentChild> {
        let Some(parent) = self
            .store
            .get_path_container_canonical(path.parent().unwrap())
            .cloned()
        else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Container` does not exist",
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

    fn init_asset(&mut self, path: PathBuf) -> Result {
        todo!();
    }
}

struct ParentChild {
    parent: ResourceId,
    child: ResourceId,
}
