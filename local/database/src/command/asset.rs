//! Asset realated commands.
use serde::{Deserialize, Serialize};
use thot_core::db::StandardSearchFilter;
use thot_core::project::{Asset as CoreAsset, StandardProperties};
use thot_core::types::ResourceId;

/// Asset realated commands.
#[derive(Serialize, Deserialize)]
pub enum AssetCommand {
    /// Retrieves an [`Asset`](CoreAsset) by [`ResourceId`].
    Get(ResourceId),

    /// Retrieves [`Asset`](CoreAsset)s by [`ResourceId`].
    ///
    /// # Returns
    /// `Vec<[Asset](CoreAsset)>` where [`Asset`](thot_core::project::Asset)s that were not found
    /// are not included.
    GetMany(Vec<ResourceId>),

    /// Insert's an [`Asset`](CoreAsset) into a
    /// [`Container`](thot_core::project::Container).
    ///  
    /// # Fields
    /// 1. [`Asset`](CoreAsset).
    /// 2. `Container`.
    Add(CoreAsset, ResourceId),

    /// Updates an [`Asset`](CoreAsset).
    UpdateProperties(ResourceId, StandardProperties),

    /// Retrieves [`Asset`](CoreAsset)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
