//! Asset realated commands.
use serde::{Deserialize, Serialize};
use thot_core::db::resources::StandardSearchFilter;
use thot_core::project::StandardProperties;
use thot_core::types::ResourceId;

/// Asset realated commands.
#[derive(Serialize, Deserialize)]
pub enum AssetCommand {
    /// Retrieves an [`Asset`](thot_core::project::Asset) by [`ResourceId`].
    GetAsset(ResourceId),

    /// Retrieves [`Asset`](thot_core::project::Asset)s by [`ResourceId`].
    ///
    /// # Returns
    /// `Vec<[Asset](thot_core::project::Asset)>` where [`Asset`](thot_core::project::Asset)s that were not found
    /// are not included.
    GetAssets(Vec<ResourceId>),

    /// Updates an [`Asset`](thot_core::project::Asset).
    UpdateAssetProperties(ResourceId, StandardProperties),

    /// Retrieves [`Asset`](thot_core::project::Asset)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
