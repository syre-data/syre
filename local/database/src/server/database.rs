//! Database for storing resources.
#[path = "./command/mod.rs"]
pub(super) mod command;

#[path = "./file_system/mod.rs"]
mod file_system;

use self::command::CommandActor;
use self::file_system::actor::{FileSystemActor, FileSystemActorCommand};
use self::file_system::file_system_event_processor::FileSystemEventProcessor;
use super::store::Datastore;
use super::Event;
use crate::command::Command;
use crate::event::Update;
use crate::{common, constants, Result};
use notify_debouncer_full::DebounceEventResult;
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
            Update::Project {
                project,
                update: _update,
            } => {
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
            Command::AnalysisCommand(cmd) => self.handle_command_analysis(cmd),
        }
    }

    /// Handle file system events.
    /// To be used with [`notify::Watcher`]s.
    #[tracing::instrument(skip(self))]
    fn handle_file_system_events(&mut self, events: DebounceEventResult) -> Result {
        let events = match events {
            Ok(events) => events,
            Err(errs) => {
                #[cfg(target_os = "macos")]
                {
                    if errs.iter().any(|err| match &err.clone().kind {
                        notify::ErrorKind::Io(err)
                            if err.kind() == std::io::ErrorKind::NotFound =>
                        {
                            false
                        }
                        notify::ErrorKind::Generic(msg)
                            if msg.contains("No such file or directory") =>
                        {
                            false
                        }
                        _ => true,
                    }) {
                        tracing::debug!("watch error: {errs:?}");
                        return Err(crate::Error::Database(format!("{errs:?}")));
                    }

                    Vec::new()
                }
                #[cfg(not(target_os = "macos"))]
                {
                    tracing::debug!("watch error: {errs:?}");
                    return Err(crate::Error::Database(format!("{errs:?}")));
                }
            }
        };

        let events = FileSystemEventProcessor::process(events);
        let events = self.process_file_system_events_to_thot_events(&events);
        self.handle_thot_events(events)?;
        Ok(())
    }
}
