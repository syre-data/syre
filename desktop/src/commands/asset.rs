//! Resources for [`Asset`](CoreAsset) functionality.
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
