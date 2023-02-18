//! File actions for an [`Asset`](thot_core::project::Asset).
use crate::Result;
use serde::{Deserialize, Serialize};
use thot_core::Error as CoreError;

// *************************
// *** Asset File Action ***
// *************************

/// How an Asset's file should be handled when created.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AssetFileAction {
    /// Move original asset file to new location.
    Move,

    /// Copy asset file into location, leaving original in place.
    Copy,

    /// Reference original asset file, do not move.
    Reference,
}

impl Default for AssetFileAction {
    fn default() -> Self {
        Self::Copy
    }
}

impl Into<String> for AssetFileAction {
    fn into(self) -> String {
        match self {
            AssetFileAction::Move => "Move".to_string(),
            AssetFileAction::Copy => "Copy".to_string(),
            AssetFileAction::Reference => "Reference".to_string(),
        }
    }
}

impl AssetFileAction {
    pub fn from_string(action: String) -> Result<Self> {
        let action = action.to_lowercase();

        match action.as_str() {
            "move" => Ok(AssetFileAction::Move),
            "copy" => Ok(AssetFileAction::Copy),
            "reference" => Ok(AssetFileAction::Reference),
            _ => Err(CoreError::ValueError("invalid `AssetFileAction` string".to_string()).into()),
        }
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
