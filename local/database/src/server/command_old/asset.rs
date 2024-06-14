//! Handle `Asset` related functionality.
use super::super::Database;
use crate::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use crate::command::AssetCommand;
use crate::error::Result;
use crate::server::store::data_store::asset::Record as AssetRecord;
use serde_json::Value as JsValue;
use std::path::PathBuf;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::{Asset, AssetProperties, Container as CoreContainer};
use syre_core::types::ResourceId;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_asset(&mut self, cmd: AssetCommand) -> JsValue {
        match cmd {
            AssetCommand::Get(asset) => serde_json::to_value(self.handle_asset_get(asset)).unwrap(),

            AssetCommand::GetMany(assets) => {
                serde_json::to_value(self.handle_asset_get_many(assets)).unwrap()
            }

            AssetCommand::Path(asset) => {
                serde_json::to_value(self.handle_asset_path(asset)).unwrap()
            }

            AssetCommand::Parent(asset) => {
                serde_json::to_value(self.handle_asset_parent(asset)).unwrap()
            }

            AssetCommand::Add { asset, container } => {
                serde_json::to_value(self.handle_asset_add(asset, container)).unwrap()
            }

            AssetCommand::Remove(asset) => {
                serde_json::to_value(self.handle_asset_remove(asset)).unwrap()
            }

            AssetCommand::UpdateProperties { asset, properties } => {
                let res = self.handle_asset_update_properties(&asset, properties);
                serde_json::to_value(res).unwrap()
            }

            AssetCommand::Find { root, filter } => {
                let assets = self.object_store.find_assets(&root, filter);
                serde_json::to_value(assets).unwrap()
            }

            AssetCommand::FindWithMetadata { root, filter } => {
                let assets = self.object_store.find_assets_with_metadata(&root, filter);
                serde_json::to_value(assets).unwrap()
            }

            AssetCommand::BulkUpdateProperties(BulkUpdatePropertiesArgs { rids, update }) => {
                let res = self.handle_asset_bulk_update_properties(&rids, &update);
                serde_json::to_value(res).unwrap()
            }
        }
    }

    /// Get a single Asset by id.
    fn handle_asset_get(&self, asset: ResourceId) -> Option<Asset> {
        if let Some(container) = self.object_store.get_asset_container(&asset) {
            container.assets.get(&asset).cloned().into()
        } else {
            None
        }
    }

    /// Gets many Assets by id.
    /// If an Asset is not found it is filtered out.
    fn handle_asset_get_many(&self, assets: Vec<ResourceId>) -> Vec<Asset> {
        assets
            .iter()
            .filter_map(|aid| {
                let Some(container) = self.object_store.get_asset_container(&aid) else {
                    return None;
                };

                let Some(asset) = container.assets.get(&aid) else {
                    return None;
                };

                Some(asset.clone())
            })
            .collect()
    }

    /// # Returns
    /// The absolute path to the Asset.
    /// + `None` if the Asset is not found.
    fn handle_asset_path(&self, asset: ResourceId) -> Option<PathBuf> {
        let Some(container) = self.object_store.get_asset_container(&asset) else {
            return None;
        };

        let Some(asset) = container.assets.get(&asset) else {
            // TODO: Update object store.
            tracing::error!("asset found in object store but not container");
            return None;
        };

        Some(container.base_path().join(asset.path.as_path()))
    }

    /// # Returns
    /// The Asset's Container.
    /// + `None` if the Asset is not found.
    fn handle_asset_parent(&self, asset: ResourceId) -> Option<CoreContainer> {
        self.object_store
            .get_asset_container(&asset)
            .map(|container| (*container).clone().into())
    }

    /// Adds an Asset to a Container.
    fn handle_asset_add(&mut self, asset: Asset, container: ResourceId) -> Result {
        self.object_store
            .add_asset(asset.clone(), container.clone())?;

        if let Err(err) = self
            .data_store
            .asset()
            .create(asset.rid.clone(), asset.into(), container)
        {
            tracing::error!(?err);
        }

        Ok(())
    }

    fn handle_asset_update_properties(
        &mut self,
        asset: &ResourceId,
        properties: AssetProperties,
    ) -> Result {
        let Some(container) = self.object_store.get_asset_container_id(asset).cloned() else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        let Some(container) = self.object_store.get_container_mut(&container) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        let Some(asset) = container.assets.get_mut(asset) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        let aid = asset.rid.clone();
        let update = AssetRecord::new(properties.clone(), asset.path.clone());

        asset.properties = properties;
        container.save()?;

        if let Err(err) = self.data_store.asset().update(aid, update) {
            tracing::error!(?err);
        }

        Ok(())
    }

    fn handle_asset_remove(&mut self, asset: ResourceId) -> Result<Option<(Asset, PathBuf)>> {
        let _ = self.data_store.asset().remove(asset.clone());
        let asset_info = self.object_store.remove_asset(&asset)?;
        Ok(asset_info)
    }

    /// Bulk update `Asset` properties.
    #[tracing::instrument(skip(self))]
    fn handle_asset_bulk_update_properties(
        &mut self,
        assets: &Vec<ResourceId>,
        update: &PropertiesUpdate,
    ) -> Result {
        for asset in assets {
            self.update_asset_properties_from_update(asset, update)?;
        }

        Ok(())
    }

    /// Update a `Asset`'s properties.
    #[tracing::instrument(skip(self))]
    fn update_asset_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &PropertiesUpdate,
    ) -> Result {
        let Some(container) = self.object_store.get_asset_container_id(&rid).cloned() else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        let Some(container) = self.object_store.get_container_mut(&container) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        let Some(asset) = container.assets.get_mut(&rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        // basic properties
        if let Some(name) = update.name.as_ref() {
            asset.properties.name = name.clone();
        }

        if let Some(kind) = update.kind.as_ref() {
            asset.properties.kind = kind.clone();
        }

        if let Some(description) = update.description.as_ref() {
            asset.properties.description = description.clone();
        }

        // tags
        asset
            .properties
            .tags
            .append(&mut update.tags.insert.clone());

        asset.properties.tags.sort();
        asset.properties.tags.dedup();
        asset
            .properties
            .tags
            .retain(|tag| !update.tags.remove.contains(tag));

        // metadata
        asset
            .properties
            .metadata
            .extend(update.metadata.insert.clone());

        for key in update.metadata.remove.iter() {
            asset.properties.metadata.remove(key);
        }

        let update = asset.clone();
        container.save()?;

        if let Err(err) = self
            .data_store
            .asset()
            .update(update.rid.clone(), update.into())
        {
            tracing::error!(?err);
        }

        Ok(())
    }
}
