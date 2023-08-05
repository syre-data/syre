//! Script commands.
use serde::Serialize;
use std::path::PathBuf;
use thot_core::types::ResourceId;

#[derive(Serialize)]
pub struct AddScriptArgs {
    pub project: ResourceId,
    pub path: PathBuf,
}

#[derive(Serialize)]
pub struct RemoveScriptArgs {
    pub project: ResourceId,
    pub script: ResourceId,
}
