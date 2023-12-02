//! Resources for [`Asset`](CoreAsset) functionality.
use super::types::{MetadataAction, ResourcePropertiesUpdate, TagsAction};
use serde::Serialize;
use thot_core::project::{Asset, AssetProperties};
use thot_core::types::ResourceId;

#[derive(Serialize)]
pub struct AssetArgs {
    pub asset: Asset,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: AssetProperties, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                                     // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesStringArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: String, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                            // Unify with `UpdatePropertiesArgs` once resolved.
                            // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Bulk update resources.
#[derive(Clone, Serialize)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: AssetPropertiesUpdate,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct AssetPropertiesUpdate {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

impl From<ResourcePropertiesUpdate> for AssetPropertiesUpdate {
    fn from(update: ResourcePropertiesUpdate) -> Self {
        Self {
            name: update.name.map(|name| Some(name)),
            kind: update.kind,
            description: update.description,
            tags: update.tags,
            metadata: update.metadata,
        }
    }
}
