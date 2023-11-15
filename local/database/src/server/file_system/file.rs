//! Handle [`thot::File`](FileEvent) events.
use super::event::thot::File as FileEvent;
use crate::event::{Asset as AssetUpdate, Graph as GraphUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::path::PathBuf;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::graph::{ContainerTreeLoader, ContainerTreeTransformer};
use thot_local::project::container;
use thot_local::project::resources::Asset;

impl Database {
    pub fn handle_thot_event_file(&mut self, event: FileEvent) -> Result {
        match event {
            FileEvent::Created(path) => {
                let container_path =
                    thot_local::project::asset::container_from_path_ancestor(&path)?;

                let path_container = self
                    .store
                    .get_path_container_canonical(&container_path)
                    .unwrap()
                    .cloned()
                    .unwrap();

                let path_container = self.store.get_container(&path_container).unwrap();
                if self
                    .store
                    .get_path_asset_id_canonical(&path)
                    .unwrap()
                    .is_some()
                {
                    return Ok(());
                }

                // NOTE When transferring large amounts of data
                // some folder creation events are missed by `notify`.
                // We account for this here by checking if the file is placed in a bucket.
                // If not then we ensure it is placed in a Container, otherwise we initialize the highest possible
                // folder as a subgraph.
                let asset_path = path
                    .strip_prefix(path_container.base_path())
                    .unwrap()
                    .to_path_buf();

                let mut as_asset = true;
                if asset_path.components().count() > 1 {
                    as_asset = path_container
                        .buckets()
                        .iter()
                        .any(|bucket| asset_path.strip_prefix(bucket).is_ok())
                }

                if as_asset {
                    self.handle_file_as_asset(asset_path, path_container.rid.clone())?;
                } else {
                    let root_path = path_container
                        .base_path()
                        .join(asset_path.components().next().unwrap());

                    self.init_subgraph_file(root_path)?;
                }

                Ok(())
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn handle_file_as_asset(&mut self, asset_path: PathBuf, container: ResourceId) -> Result {
        let asset = Asset::new(ResourcePath::new(asset_path)?)?;
        let aid = asset.rid.clone();
        self.store.add_asset(asset, container.clone())?;

        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        let container = self.store.get_container(&container).unwrap();
        let asset = container.assets.get(&aid).unwrap().clone();

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

    #[tracing::instrument(skip(self))]
    fn init_subgraph_file(&mut self, path: PathBuf) -> Result {
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
        builder.build(&path)?;

        // insert into graph
        let graph = ContainerTreeLoader::load(path)?;
        let root = graph.root().clone();
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
}
