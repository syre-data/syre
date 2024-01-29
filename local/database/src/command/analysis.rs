//! Analysis commands.
use serde::{Deserialize, Serialize};
use syre_core::types::ResourceId;

#[derive(Serialize, Deserialize, Debug)]
pub enum AnalysisCommand {
    Flag {
        resource: ResourceId,
        message: String,
    },
}
