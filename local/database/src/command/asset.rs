//! Asset realated commands.
use super::types::{MetadataAction, TagsAction};
use serde::{Deserialize, Serialize};
use thot_core::db::StandardSearchFilter;
use thot_core::project::{Asset as CoreAsset, AssetProperties};
use thot_core::types::ResourceId;

/// Asset realated commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum AssetCommand {
    /// Retrieves an [`Asset`](CoreAsset) by [`ResourceId`].
    Get(ResourceId),

    /// Retrieves [`Asset`](CoreAsset)s by [`ResourceId`].
    ///
    /// # Returns
    /// `Vec<[Asset](CoreAsset)>` where [`Asset`](thot_core::project::Asset)s that were not found
    /// are not included.
    GetMany(Vec<ResourceId>),

    /// Gets an `Asset`'s `Container`.
    ///
    /// # Fields
    /// 1. `Asset`'s `ResourceId`.
    Parent(ResourceId),

    /// Insert's an [`Asset`](CoreAsset) into a
    /// [`Container`](thot_core::project::Container).
    ///  
    /// # Fields
    /// 1. [`Asset`](CoreAsset).
    /// 2. `Container`.
    Add(CoreAsset, ResourceId),

    /// Removes an [`Asset`](CoreAsset).
    Remove(ResourceId),

    /// Updates an [`Asset`](CoreAsset).
    UpdateProperties(ResourceId, AssetProperties),

    /// Retrieves [`Asset`](CoreAsset)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),

    /// Retrieves [`Asset`](CoreAsset)s based on a filter.
    /// Lineage is compiled.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    FindWithMetadata(ResourceId, StandardSearchFilter),

    /// Update multiple [`Asset`](CoreAsset)s' properties.
    BulkUpdateProperties(BulkUpdateAssetPropertiesArgs),
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AssetPropertiesUpdate {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkUpdateAssetPropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: AssetPropertiesUpdate,
}
