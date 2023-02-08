//! Asset realated commands.
use serde::{Deserialize, Serialize};
use thot_core::project::StandardProperties;
use thot_core::types::ResourceId;

/// Asset realated commands.
#[derive(Serialize, Deserialize)]
pub enum AssetCommand {
    /// Retrieves an [`Asset`](CoreAsset) by [`ResourceId`].
    GetAsset(ResourceId),

    /// Retrieves [`Asset`](CoreAsset)s by [`ResourceId`].
    ///
    /// # Returns
    /// `Vec<[Asset](CoreAsset)>` where [`Asset`](CoreAsset)s that were not found
    /// are not included.
    GetAssets(Vec<ResourceId>),

    /// Updates an [`Asset`](CoreAsset).
    UpdateAssetProperties(ResourceId, StandardProperties),
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
