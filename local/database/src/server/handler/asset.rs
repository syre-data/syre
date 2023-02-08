//! Handle `Asset` related functionality.
use super::super::Database;
use crate::command::AssetCommand;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::LocalSettings;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Asset as CoreAsset, StandardProperties};
use thot_core::types::ResourceId;

impl Database {
    pub fn handle_command_asset(&mut self, cmd: AssetCommand) -> JsValue {
        match cmd {
            AssetCommand::GetAsset(rid) => {
                let asset: Option<CoreAsset> = {
                    if let Some(container) = self.store.get_asset_container(&rid) {
                        let container = container.lock().expect("could not lock `Container`");
                        container.assets.get(&rid).cloned().into()
                    } else {
                        None
                    }
                };

                serde_json::to_value(asset).expect("could not convert `Asset` to JSON")
            }

            AssetCommand::GetAssets(rids) => {
                let assets = rids
                    .iter()
                    .filter_map(|rid| {
                        let Some(container) = self.store.get_asset_container(&rid) else {
                            return None;
                        };

                        let container = container.lock().expect("could not lock `Container`");
                        let Some(asset) = container.assets.get(&rid) else {
                            return None;
                        };

                        Some(asset.clone())
                    })
                    .collect::<Vec<CoreAsset>>();

                serde_json::to_value(assets).expect("could not convert `Vec<Asset>` to JSON")
            }

            AssetCommand::UpdateAssetProperties(rid, properties) => {
                let res = self.update_asset_properties(&rid, properties);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }
        }
    }

    fn update_asset_properties(
        &mut self,
        rid: &ResourceId,
        properties: StandardProperties,
    ) -> Result {
        let Some(container) = self.store.get_asset_container(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Asset` does not exist".to_string())).into());
        };

        let mut container = container.lock().expect("could not lock `Container`");
        let Some(asset) = container.assets.get_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Asset` does not exist".to_string())).into());
        };

        asset.properties = properties;
        container.save()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
