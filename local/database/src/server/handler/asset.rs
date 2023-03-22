//! Handle `Asset` related functionality.
use super::super::Database;
use crate::command::AssetCommand;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::LocalSettings;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer, StandardProperties};
use thot_core::types::ResourceId;

impl Database {
    pub fn handle_command_asset(&mut self, cmd: AssetCommand) -> JsValue {
        match cmd {
            AssetCommand::Get(rid) => {
                let asset: Option<CoreAsset> = {
                    if let Some(container) = self.store.get_asset_container(&rid) {
                        container.assets.get(&rid).cloned().into()
                    } else {
                        None
                    }
                };

                serde_json::to_value(asset).expect("could not convert `Asset` to JSON")
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
                    .collect::<Vec<CoreAsset>>();

                serde_json::to_value(assets).expect("could not convert `Vec<Asset>` to JSON")
            }

            AssetCommand::Parent(asset) => {
                // @todo: Convert to result for homogeneity with `ContainerCommand::Parent`.
                let container: Option<CoreContainer> = self
                    .store
                    .get_asset_container(&asset)
                    .cloned()
                    .map(|container| container.into());

                serde_json::to_value(container).expect("could not convert `Container` to JSON")
            }

            AssetCommand::Add(asset, container) => {
                let res = self.store.add_asset(asset, container);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            AssetCommand::Remove(rid) => {
                let res = self.remove_asset(&rid);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            AssetCommand::UpdateProperties(rid, properties) => {
                let res = self.update_asset_properties(&rid, properties);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            AssetCommand::Find(root, filter) => {
                let assets = self.store.find_assets(&root, filter);
                serde_json::to_value(assets).expect("could not convert result to JSON")
            }

            AssetCommand::FindWithMetadata(root, filter) => {
                let assets = self.store.find_assets_with_metadata(&root, filter);
                serde_json::to_value(assets).expect("could not convert result to JSON")
            }
        }
    }

    fn update_asset_properties(
        &mut self,
        rid: &ResourceId,
        properties: StandardProperties,
    ) -> Result {
        let Some(container) = self.store.get_asset_container_id(&rid).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Asset` does not exist")).into());
        };

        let Some(container) = self.store.get_container_mut(&container) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist")).into());
        };

        let Some(asset) = container.assets.get_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Asset` does not exist")).into());
        };

        asset.properties = properties;
        container.save()?;
        Ok(())
    }

    fn remove_asset(&mut self, rid: &ResourceId) -> Result {
        self.store.remove_asset(rid)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
