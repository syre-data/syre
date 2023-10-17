//! Database for storing resources.
#[path = "./command/mod.rs"]
mod command;

#[path = "./file_system/mod.rs"]
mod file_system;

use self::command::CommandActor;
use self::file_system::actor::{FileSystemActor, FileSystemActorCommand};
use super::store::Datastore;
use super::Event;
use crate::command::Command;
use crate::events::Update;
use crate::{common, constants, Result};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent};
use serde_json::Value as JsValue;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Database.
pub struct Database {
    store: Datastore,
    event_rx: mpsc::Receiver<Event>,
    file_system_tx: mpsc::Sender<FileSystemActorCommand>,

    /// Publication socket to broadcast updates.
    update_tx: zmq::Socket,
}

impl Database {
    /// Creates a new Database.
    /// The database immediately begins listening for ZMQ and file system events.
    pub fn new() -> Self {
        let zmq_context = zmq::Context::new();
        let update_tx = zmq_context.socket(zmq::PUB).unwrap();
        update_tx.bind(&common::zmq_url(zmq::PUB).unwrap()).unwrap();

        let (event_tx, event_rx) = mpsc::channel();
        let (file_system_tx, file_system_rx) = mpsc::channel();
        let command_actor = CommandActor::new(event_tx.clone());
        let mut file_system_actor = FileSystemActor::new(event_tx, file_system_rx);

        thread::spawn(move || command_actor.run());
        thread::spawn(move || file_system_actor.run());

        Database {
            store: Datastore::new(),
            event_rx,
            file_system_tx,
            update_tx,
        }
    }

    /// Begin responding to events.
    pub fn start(&mut self) {
        self.listen_for_events();
    }

    /// Listen for events coming from child actors.
    fn listen_for_events(&mut self) {
        loop {
            match self.event_rx.recv().unwrap() {
                Event::Command { cmd, tx } => tx.send(self.handle_command(cmd)).unwrap(),
                Event::FileSystem(events) => self.handle_file_system_events(events).unwrap(),
            }
        }
    }

    /// Add a path to watch for file system changes.
    fn watch_path(&mut self, path: impl Into<PathBuf>) {
        self.file_system_tx
            .send(FileSystemActorCommand::Watch(path.into()))
            .unwrap();
    }

    /// Remove a path from watching file system changes.
    fn unwatch_path(&mut self, path: impl Into<PathBuf>) {
        self.file_system_tx
            .send(FileSystemActorCommand::Unwatch(path.into()))
            .unwrap();
    }

    /// Publish an update to subscribers.
    /// Triggered by file system events.
    fn publish_update(&self, update: &Update) -> zmq::Result<()> {
        let mut topic = constants::PUB_SUB_TOPIC.to_string();
        match update {
            Update::Project { project, update } => {
                topic.push_str(&format!("/project/{project}"));
            }
        };

        self.update_tx.send(&topic, zmq::SNDMORE)?;
        self.update_tx
            .send(&serde_json::to_string(update).unwrap(), 0)
    }

    // TODO Handle errors.
    /// Handles a given command, returning the correct data.
    fn handle_command(&mut self, command: Command) -> JsValue {
        tracing::debug!(?command);
        match command {
            Command::AssetCommand(cmd) => self.handle_command_asset(cmd),
            Command::ContainerCommand(cmd) => self.handle_command_container(cmd),
            Command::DatabaseCommand(cmd) => self.handle_command_database(cmd),
            Command::ProjectCommand(cmd) => self.handle_command_project(cmd),
            Command::GraphCommand(cmd) => self.handle_command_graph(cmd),
            Command::ScriptCommand(cmd) => self.handle_command_script(cmd),
            Command::UserCommand(cmd) => self.handle_command_user(cmd),
        }
    }

    /// Handle file system events.
    /// To be used with [`notify::Watcher`]s.
    #[tracing::instrument(skip(self))]
    fn handle_file_system_events(&mut self, events: DebounceEventResult) -> Result {
        let events = match events {
            Ok(events) => events,
            Err(errs) => {
                tracing::debug!("watch error: {errs:?}");
                return Err(crate::Error::DatabaseError(format!("{errs:?}")));
            }
        };
        tracing::debug!(?events);

        if let Some(res) = self.try_handle_events_group(&events) {
            return res;
        }

        for event in events.into_iter() {
            match event.event.kind {
                notify::EventKind::Modify(_) => self.handle_file_system_event_modify(event)?,
                notify::EventKind::Create(_) => self.handle_file_system_event_create(event)?,
                notify::EventKind::Remove(_) => self.handle_file_system_event_remove(event)?,
                _ => todo!(),
            }
        }

        Ok(())
    }

    /// Try to handle events as a group.
    ///
    /// # Returns
    /// `Some` with the `Result` if the events were handled as a group.
    /// `None` if the events were not handled as a group.
    fn try_handle_events_group(&mut self, events: &Vec<DebouncedEvent>) -> Option<Result> {
        let rename_from_to_events = events
            .iter()
            .map(|event| {
                event.kind
                    == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                        notify::event::RenameMode::From,
                    ))
                    || event.kind
                        == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                            notify::event::RenameMode::To,
                        ))
            })
            .collect::<Vec<_>>();

        let num_rename_from_to_events = rename_from_to_events
            .iter()
            .filter(|&&event| event)
            .collect::<Vec<_>>()
            .len();

        if num_rename_from_to_events == 2 {
            let p1 = rename_from_to_events.iter().position(|&e| e).unwrap();
            let p2 = rename_from_to_events.iter().rposition(|&e| e).unwrap();
            let (from, to) = if events[p1].kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::From,
                )) {
                (&events[p1], &events[p2])
            } else {
                (&events[p2], &events[p1])
            };

            return Some(self.handle_from_to_event(&from.paths[0], &to.paths[0]));
        }

        None
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
