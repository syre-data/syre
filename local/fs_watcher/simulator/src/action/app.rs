use std::path::PathBuf;
use syre_core::types::ResourceId;

#[derive(Debug, derive_more::From)]
pub enum Action {
    Project(Project),
}

#[derive(Debug)]
pub enum Project {
    Create { id: ResourceId, path: PathBuf },
    Move { id: ResourceId, to: PathBuf },
}
