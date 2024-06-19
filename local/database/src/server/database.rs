//! Database for storing resources.syre_local::system::collections::
#[path = "query/mod.rs"]
pub(super) mod query;

#[path = "file_system/mod.rs"]
mod file_system;

use super::{
    state::{self, State},
    store::data_store,
};
use crate::{common, constants, event::Update};
use crossbeam::channel::{select, Receiver};
use query::Query;
use serde_json::Value as JsValue;
use std::{collections::HashMap, path::PathBuf, thread};
use syre_local::{
    project::resources::{project::LoadError, Analyses, Project},
    system::collections::{ProjectManifest, UserManifest},
    TryReducible,
};

pub use config::Config;

pub struct Builder {
    config: Config,
    paths: Vec<PathBuf>,
}

impl Builder {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            paths: vec![],
        }
    }
}

impl Builder {
    pub fn run(self) -> Result<(), zmq::Error> {
        let zmq_context = zmq::Context::new();
        let update_tx = zmq_context.socket(zmq::PUB)?;
        update_tx.bind(&common::localhost_with_port(self.config.update_port()))?;

        let (query_tx, query_rx) = crossbeam::channel::unbounded();
        let (fs_event_tx, fs_event_rx) = crossbeam::channel::unbounded();
        let (fs_command_tx, fs_command_rx) = crossbeam::channel::unbounded();
        let query_actor = query::Actor::new(query_tx.clone());

        let fs_command_client = syre_fs_watcher::Client::new(fs_command_tx);
        let mut fs_watcher = syre_fs_watcher::server::Builder::new(
            fs_command_rx,
            fs_event_tx,
            syre_fs_watcher::server::Config::new(
                self.config.user_manifest().clone(),
                self.config.project_manifest().clone(),
            ),
        );
        fs_watcher.add_paths(self.paths);

        let (store_tx, store_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut datastore = data_store::Datastore::new(store_rx);
        let data_store = data_store::Client::new(store_tx);

        thread::Builder::new()
            .name("syre local database file system watcher".to_string())
            .spawn(move || fs_watcher.run())
            .unwrap();

        thread::Builder::new()
            .name("syre local database query actor".to_string())
            .spawn(move || {
                if let Err(err) = query_actor.run() {
                    tracing::error!(?err);
                }
            })
            .unwrap();

        thread::Builder::new()
            .name("syre local database data store".to_string())
            .spawn(move || {
                if let Err(err) = datastore.run() {
                    tracing::error!(?err);
                }
            })
            .unwrap();

        let mut user_manifest_state = Ok(vec![]);
        let mut project_manifest_state = Ok(vec![]);
        if let Err(errors) = fs_event_rx.recv().unwrap() {
            for err in errors {
                match err {
                    syre_fs_watcher::Error::Watch(err) => {
                        if let [path] = &err.paths[..] {
                            if path == self.config.user_manifest() {
                                let err = match err.kind {
                                    notify::ErrorKind::Io(err) => err,
                                    notify::ErrorKind::MaxFilesWatch
                                    | notify::ErrorKind::PathNotFound
                                    | notify::ErrorKind::Generic(_) => todo!(),
                                    notify::ErrorKind::WatchNotFound
                                    | notify::ErrorKind::InvalidConfig(_) => unreachable!(),
                                };

                                user_manifest_state = Err(err.into());
                            } else if path == self.config.project_manifest() {
                                let err = match err.kind {
                                    notify::ErrorKind::Io(err) => err,
                                    notify::ErrorKind::MaxFilesWatch
                                    | notify::ErrorKind::PathNotFound
                                    | notify::ErrorKind::Generic(_) => todo!(),
                                    notify::ErrorKind::WatchNotFound
                                    | notify::ErrorKind::InvalidConfig(_) => unreachable!(),
                                };

                                project_manifest_state = Err(err.into());
                            } else {
                                tracing::error!(?err);
                            }
                        }
                    }
                    syre_fs_watcher::Error::Processing { events, kind } => {
                        tracing::error!(?events, ?kind);
                        todo!()
                    }
                }
            }
        }

        if let Ok(manifest_state) = user_manifest_state.as_mut() {
            match UserManifest::load_from(self.config.user_manifest()) {
                Ok(manifest) => {
                    manifest_state.extend(manifest.to_vec());
                }
                Err(err) => {
                    user_manifest_state = Err(err);
                }
            }
        }

        if let Ok(manifest_state) = project_manifest_state.as_mut() {
            match ProjectManifest::load_from(self.config.project_manifest()) {
                Ok(manifest) => {
                    manifest_state.extend(manifest.to_vec());
                }
                Err(err) => {
                    project_manifest_state = Err(err);
                }
            }
        }

        let mut state = State::new(user_manifest_state, project_manifest_state);
        if let Ok(manifest) = state.app().project_manifest().as_ref() {
            for path in manifest.clone() {
                state
                    .try_reduce(state::Action::InsertProject(
                        state::project::State::load_from(path),
                    ))
                    .unwrap();
            }
        }

        tracing::debug!(?state);
        let mut db = Database {
            config: self.config,
            state,
            data_store,
            query_rx,
            fs_event_rx,
            fs_command_client,
            update_tx,
        };

        db.start();
        Ok(())
    }

    pub fn add_path(self, path: impl Into<PathBuf>) -> Self {
        let Self {
            mut paths,
            config: app_config,
        } = self;

        paths.push(path.into());
        Self {
            paths,
            config: app_config,
        }
    }

    pub fn add_paths(self, paths: Vec<PathBuf>) -> Self {
        let Self {
            paths: mut paths_stored,
            config: app_config,
        } = self;

        paths_stored.extend(paths);
        Self {
            paths: paths_stored,
            config: app_config,
        }
    }
}

/// Database.
pub struct Database {
    config: Config,
    state: State,
    data_store: data_store::Client,
    query_rx: Receiver<Query>,
    fs_event_rx: Receiver<syre_fs_watcher::EventResult>,
    fs_command_client: syre_fs_watcher::Client,

    /// Publication socket to broadcast updates.
    update_tx: zmq::Socket,
}

impl Database {
    /// Begin responding to events.
    pub fn start(&mut self) {
        self.listen_for_events();
    }

    /// Listen for events coming from child actors.
    fn listen_for_events(&mut self) {
        loop {
            select! {
                recv(self.query_rx) -> query => match query {
                    Ok(query::Query{query, tx}) => {
                        let response = self.handle_query(query);
                        if let Err(err) = tx.send(response) {
                            tracing::error!(?err);
                        }
                    }
                    Err(err) => panic!("{err:?}")
                },

                recv(self.fs_event_rx) -> events => match events {
                    Ok(events) => self.handle_file_system_events(events).unwrap(),
                    Err(err) => panic!("{err:?}"),
                }
            }
        }
    }

    /// Add a path to watch for file system changes.
    fn watch_path(&mut self, path: impl Into<PathBuf>) {
        let path: PathBuf = path.into();
        assert!(path.is_absolute());
        self.fs_command_client.watch(path).unwrap();
    }

    /// Remove a path from watching file system changes.
    fn unwatch_path(&mut self, path: impl Into<PathBuf>) {
        let path: PathBuf = path.into();
        assert!(path.is_absolute());
        self.fs_command_client.unwatch(path).unwrap();
    }

    /// Gets the final path of a file from the file system watcher.
    fn get_final_path(
        &self,
        path: impl Into<PathBuf>,
    ) -> Result<Option<PathBuf>, file_path_from_id::Error> {
        let path: PathBuf = path.into();
        assert!(path.is_absolute());
        self.fs_command_client
            .final_path(path)
            .map_err(|err| match err {
                syre_fs_watcher::client::error::FinalPath::InvalidPath => unreachable!(),
                syre_fs_watcher::client::error::FinalPath::Retrieval(err) => err,
            })
    }

    /// Publish a updates to subscribers.
    /// Triggered by file system events.
    fn publish_updates(&self, updates: &Vec<Update>) -> zmq::Result<()> {
        use crate::event;

        let mut sorted_updates = HashMap::with_capacity(updates.len());
        for update in updates.iter() {
            match update.kind() {
                event::UpdateKind::App(event::App::UserManifest(_)) => {
                    let events = sorted_updates
                        .entry("app/user_manifest".to_string())
                        .or_insert(vec![]);
                    events.push(update);
                }
                event::UpdateKind::App(event::App::ProjectManifest(_)) => {
                    let events = sorted_updates
                        .entry("app/project_manifest".to_string())
                        .or_insert(vec![]);
                    events.push(update);
                }
                event::UpdateKind::Project { project, .. } => {
                    let key = match project {
                        None => format!("project/unknown"),
                        Some(id) => format!("project/{id}"),
                    };

                    let events = sorted_updates.entry(key).or_insert(vec![]);
                    events.push(update);
                }
            };
        }

        let base_topic = constants::PUB_SUB_TOPIC.to_string();
        for (event_topic, updates) in sorted_updates {
            let topic = format!("{base_topic}/{event_topic}");
            self.update_tx.send(&topic, zmq::SNDMORE)?;
            if let Err(err) = self
                .update_tx
                .send(&serde_json::to_string(&updates).unwrap(), 0)
            {
                tracing::error!(?err);
            }
        }

        Ok(())
    }

    fn handle_query(&self, query: crate::Query) -> JsValue {
        use crate::Query;

        tracing::debug!(?query);
        match query {
            Query::Config(query) => self.handle_query_config(query),
            Query::State(query) => self.handle_query_state(query),
            Query::User(query) => self.handle_query_user(query),
            Query::Project(query) => self.handle_query_project(query),
        }
    }
    // // TODO Handle errors.
    // /// Handles a given command, returning the correct data.
    // fn handle_command(&mut self, command: crate::Command) -> JsValue {
    //     use crate::Command;

    //     tracing::debug!(?command);
    //     match command {
    //         Command::Asset(cmd) => self.handle_command_asset(cmd),
    //         Command::Container(cmd) => self.handle_command_container(cmd),
    //         Command::Database(cmd) => self.handle_command_database(cmd),
    //         Command::Project(cmd) => self.handle_command_project(cmd),
    //         Command::Graph(cmd) => self.handle_command_graph(cmd),
    //         Command::Analysis(cmd) => self.handle_command_analysis(cmd),
    //         Command::User(cmd) => self.handle_command_user(cmd),
    //         Command::Runner(cmd) => self.handle_command_runner(cmd),
    //         Command::Search(cmd) => self.handle_command_search(cmd),
    //     }
    // }
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

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::path::Path;

    impl Database {
        /// Handle file system events.
        /// To be used with [`notify::Watcher`]s.
        #[tracing::instrument(skip(self))]
        pub fn handle_file_system_events(
            &mut self,
            events: syre_fs_watcher::EventResult,
        ) -> crate::Result {
            let events = match events {
                Ok(events) => events,
                Err(errs) => self.handle_file_system_watcher_errors(errs)?,
            };

            let updates = self.process_file_system_events(events);
            if let Err(err) = self.publish_updates(&updates) {
                tracing::error!(?err);
            }

            tracing::debug!(?self.state);
            Ok(())
        }

        fn handle_file_system_watcher_errors(
            &self,
            errors: Vec<syre_fs_watcher::Error>,
        ) -> crate::Result<Vec<syre_fs_watcher::Event>> {
            todo!();
        }
    }

    fn path_in_trash(path: impl AsRef<Path>) -> bool {
        todo!()
    }
}

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn process_file_system_events(
        &mut self,
        events: Vec<syre_fs_watcher::Event>,
    ) -> Vec<Update> {
        self.process_events(events)
    }
}

mod config {
    use crate::constants::PortNumber;
    use std::path::PathBuf;

    pub struct Config {
        user_manifest: PathBuf,
        project_manifest: PathBuf,
        update_port: PortNumber,
    }

    impl Config {
        pub fn new(
            user_manifest: impl Into<PathBuf>,
            project_manifest: impl Into<PathBuf>,
            update_port: impl Into<PortNumber>,
        ) -> Self {
            Self {
                user_manifest: user_manifest.into(),
                project_manifest: project_manifest.into(),
                update_port: update_port.into(),
            }
        }

        pub fn user_manifest(&self) -> &PathBuf {
            &self.user_manifest
        }

        pub fn project_manifest(&self) -> &PathBuf {
            &self.project_manifest
        }

        pub fn update_port(&self) -> PortNumber {
            self.update_port
        }
    }
}
