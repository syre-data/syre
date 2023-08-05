//! Resources for [`Asset`](CoreAsset) functionality.
use serde::Serialize;
use thot_core::project::Asset as CoreAsset;

#[derive(Serialize)]
pub struct AssetArgs {
    pub asset: CoreAsset,
}
