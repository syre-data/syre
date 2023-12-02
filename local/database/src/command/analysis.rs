//! Analysis commands.
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

#[derive(Serialize, Deserialize, Debug)]
pub enum AnalysisCommand {
    Flag {
        resource: ResourceId,
        message: String,
    },
}
