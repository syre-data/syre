//! Handle [`thot::Folder`](FolderEvent) events.
use super::event::thot::Folder as FolderEvent;
use super::ParentChild;
use crate::event::{Graph as GraphUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use std::path::{Component, PathBuf};
use thot_local::graph::{ContainerTreeLoader, ContainerTreeTransformer};
use thot_local::project::{asset, container, project};

impl Database {
    pub fn handle_thot_event_folder(&mut self, event: FolderEvent) -> Result {
        match event {
            FolderEvent::Created(path) => {
                // ignore analysis folder
                let path_project = project::project_root_path(&path).unwrap();
                let path_project = self
                    .store
                    .get_path_project_canonical(&path_project)
                    .unwrap()
                    .unwrap();

                let path_project = self.store.get_project(path_project).unwrap();
                let analysis_path = fs::canonicalize(
                    path_project
                        .base_path()
                        .join(path_project.analysis_root.clone().unwrap()),
                )
                .unwrap();

                if path.strip_prefix(analysis_path).is_ok() {
                    return Ok(());
                }

                // ignore if bucket
                let path_container = asset::container_from_path_ancestor(&path)?;
                let path_container = self.store.get_path_container(&path_container).unwrap();
                let path_container = self.store.get_container(path_container).unwrap();
                let bucket_path = path
                    .strip_prefix(path_container.base_path())
                    .unwrap()
                    .to_path_buf();

                let Component::Normal(bucket_path_root) = bucket_path.components().next().unwrap()
                else {
                    panic!("invalid path type")
                };

                if path_container
                    .buckets()
                    .iter()
                    .any(|bucket| bucket.starts_with(&bucket_path_root))
                {
                    return Ok(());
                }

                // init subgraph
                // NOTE When transferring large amounts of data
                // some folder creation events are missed by `notify`.
                // We account for this here by taking the highest possible folder as the root
                // of the subgraph to be initialized.
                let mut root_path = path_container.base_path().to_path_buf();
                root_path.push(bucket_path.components().next().unwrap());

                let ParentChild {
                    parent,
                    child: container,
                } = self.init_subgraph(root_path)?;

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
        }
    }

    /// Initialize a path as a  Container tree and insert it into the graph.
    ///
    /// # Returns
    /// `ResourceId` of the graph's root `Container`.
    #[tracing::instrument(skip(self))]
    fn init_subgraph(&mut self, path: PathBuf) -> Result<ParentChild> {
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
}
