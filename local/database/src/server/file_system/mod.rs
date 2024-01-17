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
pub mod script;
pub mod thot_event_processor;

use crate::server::Database;
use crate::Result;
use event::thot::Event;
use thot_core::types::ResourceId;

impl Database {
    pub fn handle_thot_events(&mut self, events: Vec<Event>) -> Result {
        let events = sort_events(events);
        tracing::debug!(?events);

        for event in events.into_iter() {
            match event {
                Event::Project(event) => self.handle_thot_event_project(event)?,
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

fn sort_events(events: Vec<Event>) -> Vec<Event> {
    let mut sorted_events = Vec::with_capacity(events.len());
    let mut project_events = Vec::with_capacity(events.len());
    let mut folder_events = Vec::with_capacity(events.len());
    let mut file_events = Vec::with_capacity(events.len());
    let mut other_events = Vec::with_capacity(events.len());
    for event in events {
        match event {
            Event::Project(_) => project_events.push(event),
            Event::Graph(_) => folder_events.push(event),
            Event::Container(_) => folder_events.push(event),
            Event::Asset(_) => file_events.push(event),
            Event::Script(_) => other_events.push(event),
            Event::Folder(_) => folder_events.push(event),
            Event::File(_) => file_events.push(event),
        }
    }

    sorted_events.append(&mut project_events);
    sorted_events.append(&mut folder_events);
    sorted_events.append(&mut file_events);
    sorted_events.append(&mut other_events);
    sorted_events
}
