//! File system event handler.
pub mod actor;
pub mod asset;
pub mod container;
pub mod event;
pub mod file;
pub mod file_system_event_processor;
pub mod folder;
pub mod graph;
pub mod project;
pub mod rectify_event_paths;
pub mod script;
pub mod thot_event_processor;

use crate::server::Database;
use crate::Result;
use event::app::Event;
use thot_core::types::ResourceId;

struct ParentChild {
    parent: ResourceId,
    child: ResourceId,
}
