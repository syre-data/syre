//! Database for storing resources.
#[path = "./command/mod.rs"]
pub(super) mod command;

#[path = "./file_system/mod.rs"]
mod file_system;

use self::command::CommandActor;
use self::file_system::actor::{FileSystemActor, FileSystemActorCommand};
use self::file_system::file_system_event_processor::FileSystemEventProcessor;
use super::store::{data_store, Objectstore};
use super::Event;
use crate::command::Command;
use crate::event::Update;
use crate::{common, constants, Result};
use notify_debouncer_full::DebounceEventResult;
use serde_json::Value as JsValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::sync::mpsc;
use std::thread;

/// Database.
pub struct Database {
    object_store: Objectstore,
    data_store: data_store::Client,
    event_rx: mpsc::Receiver<Event>,
    file_system_tx: mpsc::Sender<FileSystemActorCommand>,

    /// Publication socket to broadcast updates.
    update_tx: zmq::Socket,
}

impl Database {
    /// Creates a new Database.
    /// The database immediately begins listening for ZMQ and file system events.
    pub fn new() -> StdResult<Self, zmq::Error> {
        let zmq_context = zmq::Context::new();
        let update_tx = zmq_context.socket(zmq::PUB)?;
        update_tx.bind(&common::zmq_url(zmq::PUB).unwrap())?;

        let (event_tx, event_rx) = mpsc::channel();
        let (file_system_tx, file_system_rx) = mpsc::channel();
        let command_actor = CommandActor::new(event_tx.clone());
        let mut file_system_actor = FileSystemActor::new(event_tx, file_system_rx);

        let (store_tx, store_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut datastore = data_store::Datastore::new(store_rx);
        let data_store = data_store::Client::new(store_tx);

        thread::spawn(move || file_system_actor.run());
        thread::spawn(move || {
            if let Err(err) = command_actor.run() {
                tracing::error!(?err);
            }
        });

        thread::spawn(move || {
            if let Err(err) = datastore.run() {
                tracing::error!(?err);
            }
        });

        Ok(Database {
            object_store: Objectstore::new(),
            data_store,
            event_rx,
            file_system_tx,
            update_tx,
        })
    }

    /// Begin responding to events.
    pub fn start(&mut self) {
        self.listen_for_events();
    }

    /// Listen for events coming from child actors.
    fn listen_for_events(&mut self) {
        loop {
            match self.event_rx.recv().unwrap() {
                Event::Command { cmd, tx } => {
                    let response = self.handle_command(cmd);
                    if let Err(err) = tx.send(response) {
                        tracing::error!(?err);
                    }
                }

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

    /// Gets the final path of a file from the file system watcher.
    fn get_final_path(
        &self,
        path: impl Into<PathBuf>,
    ) -> StdResult<Option<PathBuf>, file_path_from_id::Error> {
        let (tx, rx) = mpsc::channel();
        self.file_system_tx
            .send(FileSystemActorCommand::FinalPath {
                path: path.into(),
                tx,
            })
            .unwrap();

        rx.recv().unwrap()
    }

    /// Publish a updates to subscribers.
    /// Triggered by file system events.
    fn publish_updates(&self, updates: &Vec<Update>) -> zmq::Result<()> {
        let mut project_updates = HashMap::with_capacity(updates.len());
        for update in updates.iter() {
            match update {
                Update::Project {
                    event_id: _,
                    project,
                    update: _,
                } => {
                    let project = project_updates.entry(project).or_insert(vec![]);
                    project.push(update);
                }
            };
        }

        let topic = constants::PUB_SUB_TOPIC.to_string();
        for (project, updates) in project_updates {
            let project_topic = format!("{topic}/project/{project}");
            self.update_tx.send(&project_topic, zmq::SNDMORE)?;
            if let Err(err) = self
                .update_tx
                .send(&serde_json::to_string(&updates).unwrap(), 0)
            {
                tracing::error!(?err);
            }
        }

        Ok(())
    }

    // TODO Handle errors.
    /// Handles a given command, returning the correct data.
    fn handle_command(&mut self, command: Command) -> JsValue {
        tracing::debug!(?command);
        match command {
            Command::Asset(cmd) => self.handle_command_asset(cmd),
            Command::Container(cmd) => self.handle_command_container(cmd),
            Command::Database(cmd) => self.handle_command_database(cmd),
            Command::Project(cmd) => self.handle_command_project(cmd),
            Command::Graph(cmd) => self.handle_command_graph(cmd),
            Command::Analysis(cmd) => self.handle_command_analysis(cmd),
            Command::User(cmd) => self.handle_command_user(cmd),
            Command::Runner(cmd) => self.handle_command_runner(cmd),
            Command::Search(cmd) => self.handle_command_search(cmd),
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    impl Database {
        /// Handle file system events.
        /// To be used with [`notify::Watcher`]s.
        #[tracing::instrument(skip(self))]
        pub fn handle_file_system_events(&mut self, events: DebounceEventResult) -> Result {
            let events = match events {
                Ok(events) => events,
                Err(errs) => {
                    tracing::error!("watch error: {errs:?}");
                    return Err(crate::Error::Database(format!("{errs:?}")));
                }
            };

            let events = self.rectify_event_paths(events);
            let mut events = FileSystemEventProcessor::process(events);
            events.sort_by(|a, b| a.time.cmp(&b.time));
            let updates = self.process_file_system_events(events);
            if let Err(err) = self.publish_updates(&updates) {
                tracing::error!(?err);
            }

            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use notify_debouncer_full::DebouncedEvent;
    use std::path::{Component, Path};
    use std::time::Instant;

    const TRASH_PATH: &str = ".Trash";

    impl Database {
        /// Handle file system events.
        /// To be used with [`notify::Watcher`]s.
        #[tracing::instrument(skip(self))]
        pub fn handle_file_system_events(&mut self, events: DebounceEventResult) -> Result {
            let events = match events {
                Ok(events) => events,
                Err(errs) => self.handle_file_system_watcher_errors(errs)?,
            };

            let mut events = FileSystemEventProcessor::process(events);
            events.sort_by(|a, b| a.time.cmp(&b.time));
            let updates = self.process_file_system_events(events);
            if let Err(err) = self.publish_updates(&updates) {
                tracing::error!(?err);
            }

            Ok(())
        }

        fn handle_file_system_watcher_errors(
            &self,
            errors: Vec<notify::Error>,
        ) -> Result<Vec<DebouncedEvent>> {
            const WATCH_ROOT_MOVED_PATTERN: &str =
                r"IO error for operation on (.+): No such file or directory \(os error 2\)";

            let (root_moved_errors, unhandled_errors): (Vec<_>, Vec<_>) =
                errors.into_iter().partition(|err| match &err.kind {
                    notify::ErrorKind::Generic(msg)
                        if msg.contains("No such file or directory (os error 2)") =>
                    {
                        true
                    }

                    _ => false,
                });

            let root_moved_pattern = regex::Regex::new(WATCH_ROOT_MOVED_PATTERN).unwrap();
            let moved_roots = root_moved_errors
                .into_iter()
                .map(|err| {
                    let notify::ErrorKind::Generic(msg) = err.kind else {
                        panic!("failed to partition errors correctly");
                    };

                    match root_moved_pattern.captures(&msg) {
                        None => panic!("unknown error message"),
                        Some(captures) => {
                            let path = captures.get(1).unwrap().as_str().to_string();
                            PathBuf::from(path)
                        }
                    }
                })
                .collect::<Vec<_>>();

            if moved_roots.len() == 0 && unhandled_errors.len() > 0 {
                tracing::debug!("watch error: {unhandled_errors:?}");
                return Err(crate::Error::Database(format!("{unhandled_errors:?}")));
            }

            let mut events = Vec::with_capacity(moved_roots.len() * 2);
            for path in moved_roots {
                let final_path = match self.get_final_path(&path) {
                    Ok(Some(final_path)) => Some(final_path),

                    Ok(None) => {
                        tracing::debug!("could not get final path of {path:?}");
                        continue;
                    }

                    Err(file_path_from_id::Error::NoFileInfo) => {
                        // path deleted
                        None
                    }

                    Err(err) => {
                        tracing::debug!("error retrieving final path of {path:?}: {err:?}");
                        continue;
                    }
                };

                tracing::debug!(?final_path);

                events.push(DebouncedEvent::new(
                    notify::Event {
                        kind: notify::EventKind::Remove(notify::event::RemoveKind::Folder),
                        paths: vec![path],
                        attrs: notify::event::EventAttributes::new(),
                    },
                    Instant::now(),
                ));

                if let Some(final_path) = final_path {
                    if !path_in_trash(&final_path) {
                        events.push(DebouncedEvent::new(
                            notify::Event {
                                kind: notify::EventKind::Create(notify::event::CreateKind::Folder),
                                paths: vec![final_path],
                                attrs: notify::event::EventAttributes::new(),
                            },
                            Instant::now(),
                        ));
                    }
                }
            }
            tracing::debug!(?events);

            Ok(events)
        }
    }

    fn path_in_trash(path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        match std::env::var_os("HOME") {
            None => {
                for component in path.components() {
                    match component {
                        Component::Normal(component) => {
                            if component == TRASH_PATH {
                                return true;
                            }
                        }

                        _ => {}
                    }
                }

                return false;
            }
            Some(home) => {
                let trash_path = PathBuf::from(home).join(TRASH_PATH);
                return path.starts_with(trash_path);
            }
        }
    }
}
