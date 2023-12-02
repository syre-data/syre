//! Asset realated commands.
use super::types::{MetadataAction, TagsAction};
use serde::{Deserialize, Serialize};
use thot_core::db::StandardSearchFilter;
use thot_core::project::{Asset, AssetProperties};
use thot_core::types::ResourceId;

/// Asset realated commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum AssetCommand {
    /// Retrieves an [`Asset`] by [`ResourceId`].
    Get(ResourceId),

    /// Retrieves [`Asset`]s by [`ResourceId`].
    ///
    /// # Returns
    /// `Vec<Asset>` where [`Asset`]s that were not found
    /// are not included.
    GetMany(Vec<ResourceId>),

    /// Return the absolute path to the `Asset`'s file.
    Path(ResourceId),

    /// Gets an `Asset`'s `Container`.
    ///
    /// # Fields
    /// 1. `Asset`'s `ResourceId`.
    Parent(ResourceId),

    /// Insert's an [`Asset`] into a [`Container`](thot_core::project::Container).
    Add { asset: Asset, container: ResourceId },

    /// Updates an [`Asset`].
    UpdateProperties {
        asset: ResourceId,
        properties: AssetProperties,
    },

    /// Retrieves [`Asset`]s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find {
        root: ResourceId,
        filter: StandardSearchFilter,
    },

    /// Retrieves [`Asset`]s based on a filter.
    /// Lineage is compiled.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    FindWithMetadata {
        root: ResourceId,
        filter: StandardSearchFilter,
    },

    /// Update multiple [`Asset`]s' properties.
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
