//! Handle [`syre::Asset`](AssetEvent) events.
use super::event::app::Asset as AssetEvent;
use crate::event::{Asset as AssetUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use syre_local::project::asset as local_asset;
use uuid::Uuid;

impl Database {
    pub fn handle_app_event_asset(
        &mut self,
        event: &AssetEvent,
        event_id: &Uuid,
    ) -> Result<Vec<Update>> {
        match event {
            AssetEvent::Moved { asset, path } => {
                let asset_container = self
                    .object_store
                    .get_asset_container_id(&asset)
                    .unwrap()
                    .clone();
                let asset_project = self
                    .object_store
                    .get_container_project(&asset_container)
                    .unwrap()
                    .clone();

                let Ok(path_container) = local_asset::container_from_path_ancestor(&path) else {
                    // asset file moved out of any container
                    self.object_store.remove_asset(&asset)?;
                    return Ok(vec![Update::project(
                        asset_project,
                        AssetUpdate::Removed(asset.clone()).into(),
                        event_id.clone(),
                    )]);
                };

                let path_container = self
                    .object_store
                    .get_path_container(&path_container)
                    .unwrap()
                    .clone();

                let path_project = self
                    .object_store
                    .get_container_project(&path_container)
                    .unwrap()
                    .clone();

                let container = self.object_store.get_container(&path_container).unwrap();
                let container_path = fs::canonicalize(container.base_path()).unwrap();
                let asset_path = path.strip_prefix(container_path).unwrap();

                if asset_container == path_container {
                    // path updated
                    self.object_store
                        .update_asset_path(&asset, asset_path)
                        .unwrap();

                    let container = self.object_store.get_asset_container(&asset).unwrap();
                    let asset_path = container.assets.get(&asset).unwrap().path.clone();
                    return Ok(vec![Update::project(
                        asset_project,
                        AssetUpdate::PathChanged {
                            asset: asset.clone(),
                            path: asset_path,
                        }
                        .into(),
                        event_id.clone(),
                    )]);
                }

                // asset moved containers
                self.object_store.move_asset(&asset, &path_container)?;
                self.object_store.update_asset_path(&asset, asset_path)?;

                if asset_project == path_project {
                    let container = self.object_store.get_asset_container(&asset).unwrap();
                    let asset_path = container.assets.get(&asset).unwrap().path.clone();
                    return Ok(vec![Update::project(
                        asset_project,
                        AssetUpdate::Moved {
                            asset: asset.clone(),
                            container: path_container.clone(),
                            path: asset_path,
                        }
                        .into(),
                        event_id.clone(),
                    )]);
                } else {
                    let mut updates = vec![Update::project(
                        asset_project,
                        AssetUpdate::Removed(asset.clone()).into(),
                        event_id.clone(),
                    )];

                    let Some(container) = self.object_store.get_container(&path_container) else {
                        tracing::error!("Could not get container");
                        return Ok(updates);
                    };

                    let Some(asset) = container.assets.get(&asset) else {
                        tracing::error!("Could not get asset");

                        return Ok(updates);
                    };

                    updates.push(Update::project(
                        path_project,
                        AssetUpdate::Created {
                            container: path_container.clone(),
                            asset: asset.clone(),
                        }
                        .into(),
                        event_id.clone(),
                    ));

                    return Ok(updates);
                }
            }

            AssetEvent::Removed(asset) => {
                let container = self.object_store.get_asset_container_id(&asset).unwrap();
                let project = self
                    .object_store
                    .get_container_project(container)
                    .unwrap()
                    .clone();
                self.object_store.remove_asset(&asset)?;
                Ok(vec![Update::project(
                    project,
                    AssetUpdate::Removed(asset.clone()).into(),
                    event_id.clone(),
                )])
            }

            AssetEvent::Renamed { asset, name } => {
                let container = self.object_store.get_asset_container(&asset).unwrap();
                let cid = container.rid.clone();
                let container_path = fs::canonicalize(container.base_path()).unwrap().clone();
                let name = name.strip_prefix(container_path).unwrap();
                self.object_store.update_asset_path(&asset, name)?;

                let project = self
                    .object_store
                    .get_container_project(&cid)
                    .unwrap()
                    .clone();
                let container = self.object_store.get_asset_container(&asset).unwrap();
                let path = container.assets.get(&asset).unwrap().path.clone();
                Ok(vec![Update::project(
                    project,
                    AssetUpdate::PathChanged {
                        asset: asset.clone(),
                        path,
                    }
                    .into(),
                    event_id.clone(),
                )])
            }

            AssetEvent::FileCreated(asset) => {
                let container = self.object_store.get_asset_container(&asset).unwrap();
                let asset = container.assets.get(&asset).unwrap();
                let project = self
                    .object_store
                    .get_container_project(&container.rid)
                    .unwrap()
                    .clone();

                Ok(vec![Update::project(
                    project,
                    AssetUpdate::Created {
                        container: container.rid.clone(),
                        asset: asset.clone(),
                    }
                    .into(),
                    event_id.clone(),
                )])
            }
        }
    }
}
