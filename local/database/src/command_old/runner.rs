//! Analysis commands.
use serde::{Deserialize, Serialize};
use syre_core::types::ResourceId;

#[derive(Serialize, Deserialize, Debug)]
pub enum RunnerCommand {
    Flag {
        resource: ResourceId,
        message: String,
    },
}
