//! Handle `Asset` related functionality.
use super::super::Database;
use crate::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use crate::command::AssetCommand;
use crate::error::Result;
use serde_json::Value as JsValue;
use std::path::PathBuf;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Asset, AssetProperties, Container as CoreContainer};
use thot_core::types::ResourceId;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_asset(&mut self, cmd: AssetCommand) -> JsValue {
        match cmd {
            AssetCommand::Get(rid) => {
                let asset: Option<Asset> = {
                    if let Some(container) = self.store.get_asset_container(&rid) {
                        container.assets.get(&rid).cloned().into()
                    } else {
                        None
                    }
                };

                serde_json::to_value(asset).unwrap()
            }

            AssetCommand::GetMany(rids) => {
                let assets = rids
                    .iter()
                    .filter_map(|rid| {
                        let Some(container) = self.store.get_asset_container(&rid) else {
                            return None;
                        };

                        let Some(asset) = container.assets.get(&rid) else {
                            return None;
                        };

                        Some(asset.clone())
                    })
                    .collect::<Vec<Asset>>();

                serde_json::to_value(assets).expect("could not convert `Vec<Asset>` to JSON")
            }

            AssetCommand::Path(asset) => {
                let Some(container) = self.store.get_asset_container(&asset) else {
                    let res: Option<PathBuf> = None;
                    return serde_json::to_value(res).unwrap();
                };

                let asset = container.assets.get(&asset).unwrap();
                let path = container.base_path().join(asset.path.as_path());
                serde_json::to_value(Some(path)).unwrap()
            }

            AssetCommand::Parent(asset) => {
                // TODO Convert to result for homogeneity with `ContainerCommand::Parent`.
                let container: Option<CoreContainer> = self
                    .store
                    .get_asset_container(&asset)
                    .map(|container| (*container).clone().into());

                serde_json::to_value(container).unwrap()
            }

            AssetCommand::Add { asset, container } => {
                let res = self.store.add_asset(asset, container);
                serde_json::to_value(res).unwrap()
            }

            AssetCommand::Remove(asset) => {
                let res = self.store.remove_asset(&asset);
                serde_json::to_value(res).unwrap()
            }

            AssetCommand::UpdateProperties { asset, properties } => {
                let res = self.update_asset_properties(&asset, properties);
                serde_json::to_value(res).unwrap()
            }

            AssetCommand::Find { root, filter } => {
                let assets = self.store.find_assets(&root, filter);
                serde_json::to_value(assets).unwrap()
            }

            AssetCommand::FindWithMetadata { root, filter } => {
                let assets = self.store.find_assets_with_metadata(&root, filter);
                serde_json::to_value(assets).unwrap()
            }

            AssetCommand::BulkUpdateProperties(BulkUpdatePropertiesArgs { rids, update }) => {
                let res = self.bulk_update_asset_properties(&rids, &update);
                serde_json::to_value(res).unwrap()
            }
        }
    }

    fn update_asset_properties(&mut self, rid: &ResourceId, properties: AssetProperties) -> Result {
        let Some(container) = self.store.get_asset_container_id(&rid).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        let Some(container) = self.store.get_container_mut(&container) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        let Some(asset) = container.assets.get_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        asset.properties = properties;
        container.save()?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn remove_asset(&mut self, rid: &ResourceId) -> Result {
        let Some((_asset, path)) = self.store.remove_asset(rid)? else {
            return Ok(());
        };

        trash::delete(&path)?;
        Ok(())
    }

    /// Bulk update `Asset` properties.
    #[tracing::instrument(skip(self))]
    fn bulk_update_asset_properties(
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
        let Some(container) = self.store.get_asset_container_id(&rid).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Asset` does not exist",
            ))
            .into());
        };

        let Some(container) = self.store.get_container_mut(&container) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        let Some(asset) = container.assets.get_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
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

        container.save()?;
        Ok(())
    }
}
