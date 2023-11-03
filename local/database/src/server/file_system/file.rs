//! Handle [`thot::File`](FileEvent) events.
use super::event::thot::File as FileEvent;
use super::ParentChild;
use crate::event::{Asset as AssetUpdate, Update};
use crate::server::Database;
use crate::{Error, Result};
use std::path::PathBuf;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourcePath;
use thot_local::project::resources::Asset;

impl Database {
    pub fn handle_thot_event_file(&mut self, event: FileEvent) -> Result {
        match event {
            FileEvent::Created(path) => {
                let ParentChild {
                    parent: container,
                    child: asset,
                } = match self.asset_init(path) {
                    Ok(container_asset) => container_asset,

                    Err(Error::CoreError(CoreError::ResourceError(
                        ResourceError::AlreadyExists(_msg),
                    ))) => return Ok(()),

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
        }
    }

    fn asset_init(&mut self, path: PathBuf) -> Result<ParentChild> {
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
