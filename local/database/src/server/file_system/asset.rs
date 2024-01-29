//! Handle [`syre::Asset`](AssetEvent) events.
use super::event::app::Asset as AssetEvent;
use crate::event::{Asset as AssetUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use syre_local::project::asset as local_asset;

impl Database {
    pub fn handle_app_event_asset(&mut self, event: &AssetEvent) -> Result {
        match event {
            AssetEvent::Moved { asset, path } => {
                let asset_container = self.store.get_asset_container_id(&asset).unwrap().clone();
                let asset_project = self
                    .store
                    .get_container_project(&asset_container)
                    .unwrap()
                    .clone();

                let Ok(path_container) = local_asset::container_from_path_ancestor(&path) else {
                    // asset file moved out of any container
                    self.store.remove_asset(&asset)?;
                    self.publish_update(&Update::Project {
                        project: asset_project,
                        update: AssetUpdate::Removed(asset.clone()).into(),
                    })?;

                    return Ok(());
                };

                let path_container = self
                    .store
                    .get_path_container(&path_container)
                    .unwrap()
                    .clone();

                let path_project = self
                    .store
                    .get_container_project(&path_container)
                    .unwrap()
                    .clone();

                let container = self.store.get_container(&path_container).unwrap();
                let container_path = fs::canonicalize(container.base_path()).unwrap();
                let asset_path = path.strip_prefix(container_path).unwrap();

                if asset_container == path_container {
                    // path updated
                    self.store.update_asset_path(&asset, asset_path).unwrap();

                    let container = self.store.get_asset_container(&asset).unwrap();
                    let asset_path = container.assets.get(&asset).unwrap().path.clone();
                    self.publish_update(&Update::Project {
                        project: asset_project,
                        update: AssetUpdate::PathChanged {
                            asset: asset.clone(),
                            path: asset_path,
                        }
                        .into(),
                    })?;

                    return Ok(());
                }

                // asset moved containers
                self.store.move_asset(&asset, &path_container)?;
                self.store.update_asset_path(&asset, asset_path)?;

                if asset_project == path_project {
                    let container = self.store.get_asset_container(&asset).unwrap();
                    let asset_path = container.assets.get(&asset).unwrap().path.clone();
                    self.publish_update(&Update::Project {
                        project: asset_project,
                        update: AssetUpdate::Moved {
                            asset: asset.clone(),
                            container: path_container.clone(),
                            path: asset_path,
                        }
                        .into(),
                    })?;
                } else {
                    self.publish_update(&Update::Project {
                        project: asset_project,
                        update: AssetUpdate::Removed(asset.clone()).into(),
                    })?;

                    let asset = self
                        .store
                        .get_container(&path_container)
                        .unwrap()
                        .assets
                        .get(&asset)
                        .unwrap()
                        .clone();

                    self.publish_update(&Update::Project {
                        project: path_project,
                        update: AssetUpdate::Created {
                            container: path_container.clone(),
                            asset,
                        }
                        .into(),
                    })?;
                }

                Ok(())
            }

            AssetEvent::Removed(asset) => {
                let container = self.store.get_asset_container_id(&asset).unwrap();
                let project = self.store.get_container_project(container).unwrap().clone();
                self.store.remove_asset(&asset)?;
                self.publish_update(&Update::Project {
                    project,
                    update: AssetUpdate::Removed(asset.clone()).into(),
                })?;

                Ok(())
            }

            AssetEvent::Renamed { asset, name } => {
                let container = self.store.get_asset_container(&asset).unwrap();
                let cid = container.rid.clone();
                let container_path = fs::canonicalize(container.base_path()).unwrap().clone();
                let name = name.strip_prefix(container_path).unwrap();
                self.store.update_asset_path(&asset, name)?;

                let project = self.store.get_container_project(&cid).unwrap().clone();
                let container = self.store.get_asset_container(&asset).unwrap();
                let path = container.assets.get(&asset).unwrap().path.clone();
                self.publish_update(&Update::Project {
                    project,
                    update: AssetUpdate::PathChanged {
                        asset: asset.clone(),
                        path,
                    }
                    .into(),
                })?;

                Ok(())
            }

            AssetEvent::FileCreated(asset) => {
                let container = self.store.get_asset_container(&asset).unwrap();
                let asset = container.assets.get(&asset).unwrap();
                let project = self
                    .store
                    .get_container_project(&container.rid)
                    .unwrap()
                    .clone();

                self.publish_update(&Update::Project {
                    project,
                    update: AssetUpdate::Created {
                        container: container.rid.clone(),
                        asset: asset.clone(),
                    }
                    .into(),
                })?;

                Ok(())
            }
        }
    }
}
