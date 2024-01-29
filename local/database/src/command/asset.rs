//! Asset realated commands.
use super::types::{MetadataAction, TagsAction};
use serde::{Deserialize, Serialize};
use syre_core::db::StandardSearchFilter;
use syre_core::project::{Asset, AssetProperties};
use syre_core::types::ResourceId;

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

    /// Insert's an [`Asset`] into a [`Container`](syre_core::project::Container).
    Add { asset: Asset, container: ResourceId },

    /// Removes an Asset from its Container.
    /// Does not interat with the file system. (e.g. Deleting the associated file.)
    Remove(ResourceId),

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
    BulkUpdateProperties(BulkUpdatePropertiesArgs),
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PropertiesUpdate {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: PropertiesUpdate,
}
