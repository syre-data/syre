//! File system event handler.
pub mod actor;
pub mod asset;
pub mod container;
pub mod event;
pub mod file;
pub mod file_system_event_processor;
pub mod folder;
pub mod graph;
pub mod script;
pub mod thot_event_processor;

use crate::server::Database;
use crate::Result;
use event::thot::Event;
use thot_core::types::ResourceId;

impl Database {
    pub fn handle_thot_events(&mut self, events: Vec<Event>) -> Result {
        tracing::debug!(?events);
        for event in events.into_iter() {
            match event {
                Event::Graph(event) => self.handle_thot_event_graph(event)?,
                Event::Container(event) => self.handle_thot_event_container(event)?,
                Event::Asset(event) => self.handle_thot_event_asset(event)?,
                Event::Script(event) => self.handle_thot_event_script(event)?,
                Event::Folder(event) => self.handle_thot_event_folder(event)?,
                Event::File(event) => self.handle_thot_event_file(event)?,
            }
        }

        Ok(())
    }
}

struct ParentChild {
    parent: ResourceId,
    child: ResourceId,
}
