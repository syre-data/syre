//! File system watcher.
// NB: `notify_debouncer_full` does a pretty good job of eliminating intermediate events.
// e.g. If a folder was created then moved, `notify_debouncer_full` will only emit
// a folder created event at the final path.
// However, there is still the chance for a race condition between the events being recieved
// and what is on disk.
// It is currenlty assumed that they are in sync.
#[path = "fs_processor.rs"]
mod fs_processor;

#[path = "notify_processor.rs"]
mod notify_processor;

use super::{actor::FileSystemActor, path_watcher};
use crate::{command::WatcherCommand, event::EventResult, Command, Error, Event, EventKind};
use crossbeam::channel::{Receiver, Sender};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, FileIdCache, FileIdMap};
use std::{
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::{Arc, Mutex},
    thread,
};

pub use config::Config;

pub struct Builder {
    /// Sends events to the client.
    event_tx: Sender<EventResult>,

    // Recieve commands from the client.
    command_rx: Receiver<Command>,

    app_config: Config,

    /// Initial paths to watch.
    paths: Vec<PathBuf>,
}

impl Builder {
    /// # Arguments
    /// 1. `command_rx`: Channel to recieve commands over.
    /// 2. `event_tx`: Channel to send events over.
    /// 3. `app_config`
    pub fn new(
        command_rx: Receiver<Command>,
        event_tx: Sender<EventResult>,
        app_config: config::Config,
    ) -> Self {
        Self {
            event_tx,
            command_rx,
            app_config,
            paths: vec![],
        }
    }

    pub fn add_path(&mut self, path: impl Into<PathBuf>) {
        self.paths.push(path.into());
    }

    pub fn add_paths(&mut self, paths: Vec<PathBuf>) {
        self.paths.extend(paths);
    }

    /// Run the file system watcher.
    ///
    /// # Notes
    /// + Sends an initial event representing the initial state of the watched paths.
    /// If any errors occur with the initial paths they are sent,
    /// otherwise an empty `Ok` is sent.
    pub fn run(self) -> Result<(), crossbeam::channel::RecvError> {
        let (fs_tx, fs_rx) = crossbeam::channel::unbounded();
        let (fs_command_tx, fs_command_rx) = crossbeam::channel::unbounded();
        let mut file_system_actor = FileSystemActor::new(fs_tx, fs_command_rx);
        thread::Builder::new()
            .name("syre local file system watcher actor".to_string())
            .spawn(move || file_system_actor.run())
            .unwrap();

        let (path_watcher_tx, path_watcher_rx) = crossbeam::channel::unbounded();
        let (path_watcher_command_tx, path_watcher_command_rx) = crossbeam::channel::unbounded();
        let mut path_watcher = path_watcher::Watcher::new(path_watcher_tx, path_watcher_command_rx);
        thread::Builder::new()
            .name("syre local file system watcher path watcher".to_string())
            .spawn(move || path_watcher.run())
            .unwrap();

        let mut errors = vec![];
        for path in std::iter::once(self.app_config.user_manifest())
            .chain(std::iter::once(self.app_config.project_manifest()))
            .chain(std::iter::once(self.app_config.local_config()))
            .chain(self.paths.iter())
        {
            let (tx, rx) = crossbeam::channel::bounded(1);
            fs_command_tx
                .send(WatcherCommand::Watch {
                    path: path.clone(),
                    tx,
                })
                .unwrap();

            if let Err(err) = rx.recv()? {
                let err = match &err.kind {
                    notify::ErrorKind::Io(io_err)
                        if io_err.kind() == std::io::ErrorKind::NotFound =>
                    {
                        path_watcher_command_tx
                            .send(path_watcher::Command::Watch(path.clone()))
                            .unwrap();

                        err.add_path(path.clone())
                    }
                    _ => err,
                };

                errors.push(err);
            }
        }

        if errors.len() > 0 {
            self.event_tx
                .send(Err(errors.into_iter().map(|err| err.into()).collect()))
                .unwrap();
        } else {
            self.event_tx.send(Ok(vec![])).unwrap();
        }

        let watcher = FsWatcher {
            event_tx: self.event_tx,
            command_rx: self.command_rx,
            command_tx: fs_command_tx,
            event_rx: fs_rx,
            path_watcher_rx,
            path_watcher_command_tx,
            file_ids: Arc::new(Mutex::new(FileIdMap::new())),
            roots: Mutex::new(vec![]),
            app_config: self.app_config,
            shutdown: Mutex::new(false),
        };

        watcher.run()
    }
}

/// Listens for events on the file system.
pub struct FsWatcher {
    /// Sends events to the client.
    event_tx: Sender<EventResult>,

    // Recieve commands from the client.
    command_rx: Receiver<Command>,

    /// Send commands to the file system watcher.
    command_tx: Sender<WatcherCommand>,

    /// Recieve events from the file system watcher.
    event_rx: Receiver<DebounceEventResult>,

    /// Recieve events from the poll watcher.
    path_watcher_rx: Receiver<Vec<PathBuf>>,

    /// Send commands to the path watcher.
    path_watcher_command_tx: Sender<path_watcher::Command>,

    // NB: Must use own file id cache because the one being used by the notify watcher
    // is automatically updated on events recieved before we have access.
    // This means we lose the ability to get the file's id on destructive events
    // such as when a file is removed or moved from a location.
    // This cache is in the CommandInner and EventInner structs.
    /// Cache to hold file ids.
    file_ids: Arc<Mutex<FileIdMap>>,

    /// Project roots being watched.
    roots: Mutex<Vec<PathBuf>>,

    /// Application configuration.
    app_config: config::Config,

    /// Flag to indicate the watcher should be set down.
    shutdown: Mutex<bool>,
}

impl FsWatcher {
    /// Begins responsiveness allowing events to be sent.
    pub fn run(&self) -> StdResult<(), crossbeam::channel::RecvError> {
        loop {
            let shutdown = self.shutdown.lock().unwrap();
            if *shutdown {
                tracing::debug!("shutting down");
                break;
            }

            crossbeam::select! {
                recv(self.command_rx) -> cmd => match cmd {
                    Ok(cmd) => self.handle_command(cmd),
                    Err(err) => {
                        tracing::error!("command rx channel closed, shutting down");
                        return Err(err);
                    }
                },

                recv(self.event_rx) -> events => match events {
                    Ok(events) => self.handle_events(events),
                    Err(err) => {
                        tracing::error!("event rx channel closed, shutting down");
                        return Err(err);
                    }
                },

                recv(self.path_watcher_rx) -> paths => match paths {
                    Ok(events) => self.handle_path_watcher_event(events),
                    Err(err) => {
                        tracing::error!("path watcher rx channel closed, shutting down");
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_command(&self, command: Command) {
        tracing::debug!(?command);
        match command {
            Command::Watch(path) => {
                self.handle_command_watch(path);
            }

            Command::Unwatch(path) => {
                self.handle_command_unwatch(path);
            }

            Command::ClearProjects => {
                let (tx, rx) = crossbeam::channel::bounded(1);
                let mut roots = self.roots.lock().unwrap();
                for path in roots.clone().iter() {
                    self.command_tx
                        .send(WatcherCommand::Unwatch {
                            path: path.clone(),
                            tx: tx.clone(),
                        })
                        .unwrap();

                    // Only way for unwatch to fail is if relative path is given
                    // but can not be canonicalized.
                    // Because only absolute paths are accepted, watch should not fail.
                    rx.recv().unwrap().unwrap();
                    roots.retain(|root| root != path);
                    let mut file_ids = self.file_ids.lock().unwrap();
                    file_ids.remove_root(&path);
                }

                assert!(roots.is_empty());
            }

            Command::FinalPath { path, tx } => {
                self.final_path(path, tx);
            }

            Command::Shutdown => {
                let mut shutdown = self.shutdown.lock().unwrap();
                *shutdown = true;
            }
        }
    }

    /// Gets the final path of a file.
    ///
    /// # Returns
    /// + `None` if the path is not in the watcher's cache.
    ///
    /// # Errors
    /// + If the final path could not be obtained.
    fn final_path(
        &self,
        path: impl AsRef<Path>,
        tx: Sender<StdResult<Option<PathBuf>, file_path_from_id::Error>>,
    ) {
        let path = path.as_ref();
        let id = {
            let file_ids = self.file_ids.lock().unwrap();
            let Some(id) = file_ids.cached_file_id(path).cloned() else {
                tx.send(Ok(None)).unwrap();
                return;
            };

            id
        };

        let path = file_path_from_id::path_from_id(&id).map(|path| Some(path));
        tx.send(path).unwrap();
    }

    fn handle_path_watcher_event(&self, paths: Vec<PathBuf>) {
        use super::event::{Event, File, Folder};
        use std::time::Instant;

        let events = paths
            .into_iter()
            .filter_map(|path| {
                let (tx, rx) = crossbeam::channel::bounded(1);
                self.command_tx
                    .send(WatcherCommand::Watch {
                        path: path.clone(),
                        tx,
                    })
                    .unwrap();

                if rx.recv().unwrap().is_err() {
                    return None;
                }

                if path.is_dir() {
                    self.path_watcher_command_tx
                        .send(path_watcher::Command::Unwatch(path.clone()))
                        .unwrap();

                    Some(Event::new(Folder::Created(path), Instant::now()))
                } else if path.is_file() {
                    self.path_watcher_command_tx
                        .send(path_watcher::Command::Unwatch(path.clone()))
                        .unwrap();

                    Some(Event::new(File::Created(path), Instant::now()))
                } else if path.is_symlink() {
                    todo!();
                } else if path.exists() {
                    todo!();
                } else {
                    None
                }
            })
            .collect();

        let (events, errors) = self.process_events_fs_to_app(events);
        if !events.is_empty() {
            self.event_tx.send(Ok(events)).unwrap();
        }

        if !errors.is_empty() {
            self.event_tx.send(Err(errors)).unwrap();
        }
    }

    fn handle_events(&self, events: DebounceEventResult) {
        let events = match events {
            Ok(events) => events,
            Err(errors) => {
                self.handle_event_errors(errors);
                return;
            }
        };

        if let Some(event) = events.iter().find(|event| event.need_rescan()) {
            let mut file_ids = self.file_ids.lock().unwrap();
            file_ids.rescan();
            Event::new_from_notify(EventKind::OutOfSync, event.attrs.tracker());
            self.event_tx.send(Ok(vec![])).unwrap();
        } else {
            let (events, errors) = self.process_events(events);
            if !events.is_empty() {
                self.event_tx.send(Ok(events)).unwrap();
            }

            if !errors.is_empty() {
                self.event_tx.send(Err(errors)).unwrap();
            }
        }
    }

    fn handle_event_errors(&self, errors: Vec<notify::Error>) {
        let errors = errors.into_iter().map(|err| Error::Watch(err)).collect();
        self.event_tx.send(Err(errors)).unwrap();
    }

    /// Process file system events into app events.
    ///
    /// # Returns
    /// Tuple of (events, errors).
    fn process_events(
        &self,
        events: Vec<notify_debouncer_full::DebouncedEvent>,
    ) -> (Vec<Event>, Vec<Error>) {
        #[cfg(target_os = "linux")]
        let events = self.handle_remove_events(events);

        tracing::debug!(?events);
        let (fs_events, mut errors) = self.process_events_notify_to_fs(&events);

        tracing::debug!(?fs_events, ?errors);
        let (mut app_events, app_errors) = self.process_events_fs_to_app(fs_events);
        app_events.sort_by_key(|event| event.time().clone());

        tracing::debug!(?app_events, ?app_errors);
        errors.extend(app_errors);

        (app_events, errors)
    }
}

impl FsWatcher {
    fn handle_command_watch(&self, path: PathBuf) {
        // Required due to match arm guard expression.
        // See https://github.com/rust-lang/rfcs/pull/3637
        fn watch_path(tx: &Sender<path_watcher::Command>, path: PathBuf) {
            tracing::debug!("watching {path:?} for creation");
            tx.send(path_watcher::Command::Watch(path)).unwrap();
        }

        assert!(path.is_absolute());
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.command_tx
            .send(WatcherCommand::Watch {
                path: path.clone(),
                tx,
            })
            .unwrap();

        if let Err(err) = rx.recv().unwrap() {
            match &err.kind {
                notify::ErrorKind::PathNotFound => {
                    watch_path(&self.path_watcher_command_tx, path.clone());
                }

                notify::ErrorKind::Io(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    watch_path(&self.path_watcher_command_tx, path.clone());
                }

                notify::ErrorKind::Generic(_)
                | notify::ErrorKind::Io(_)
                | notify::ErrorKind::MaxFilesWatch => todo!(),

                notify::ErrorKind::WatchNotFound | notify::ErrorKind::InvalidConfig(_) => {
                    tracing::error!(?err);
                    unreachable!()
                }
            }
        }

        let mut roots = self.roots.lock().unwrap();
        if !roots.contains(&path) {
            roots.push(path.clone());
        }

        let mut file_ids = self.file_ids.lock().unwrap();
        file_ids.add_root(path, notify::RecursiveMode::Recursive);
    }

    fn handle_command_unwatch(&self, path: PathBuf) {
        assert!(path.is_absolute());
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.command_tx
            .send(WatcherCommand::Unwatch {
                path: path.clone(),
                tx,
            })
            .unwrap();

        if let Err(err) = rx.recv().unwrap() {
            match &err.kind {
                notify::ErrorKind::WatchNotFound => {
                    self.path_watcher_command_tx
                        .send(path_watcher::Command::Unwatch(path.clone()))
                        .unwrap();
                }

                _ => {
                    tracing::error!(?err);
                    unreachable!();
                }
            }
        }

        let mut roots = self.roots.lock().unwrap();
        roots.retain(|root| root != &path);

        let mut file_ids = self.file_ids.lock().unwrap();
        file_ids.remove_root(&path);
    }
}

#[cfg(target_os = "linux")]
impl FsWatcher {
    /// Some text editors remove then replace a file when saving it.
    /// We must manually check it the file exists to determine if this occured.
    ///
    /// If a config file is removed, it is added to the path watcher.
    ///
    /// # Returns
    /// Events modified to account for false removals.
    ///
    /// # Notes
    /// See https://docs.rs/notify/latest/notify/#editor-behaviour.
    fn handle_remove_events(&self, mut events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        let mut remove_events = vec![];
        for (index, event) in events.iter().enumerate() {
            match &event.kind {
                notify::EventKind::Remove(_) => {
                    let [path] = &event.paths[..] else {
                        panic!("invalid paths");
                    };

                    if !remove_events.iter().any(|&index| {
                        let event: &DebouncedEvent = &events[index];
                        if let Some(p) = event.paths.get(0) {
                            p == path
                        } else {
                            false
                        }
                    }) {
                        remove_events.push(index);
                    }
                }
                notify::EventKind::Create(_) => {
                    let [path] = &event.paths[..] else {
                        panic!("invalid paths");
                    };

                    if let Some(index) = remove_events.iter().position(|&index| {
                        let event = &events[index];
                        &event.paths[0] == path
                    }) {
                        remove_events.swap_remove(index);
                    }
                }
                notify::EventKind::Any
                | notify::EventKind::Access(_)
                | notify::EventKind::Modify(_)
                | notify::EventKind::Other => {}
            }
        }

        for index in remove_events {
            let event = &events[index];
            let [path] = &event.paths[..] else {
                panic!("invalid paths");
            };

            if path == self.app_config.user_manifest()
                || path == self.app_config.project_manifest()
                || path == self.app_config.local_config()
            {
                if path.exists() {
                    let (tx, rx) = crossbeam::channel::bounded(1);
                    tracing::debug!("rewatching {path:?}");
                    self.command_tx
                        .send(WatcherCommand::Watch {
                            path: path.clone(),
                            tx,
                        })
                        .unwrap();

                    match rx.recv().unwrap() {
                        Ok(()) => {
                            let event = event.event.clone().set_kind(notify::EventKind::Modify(
                                notify::event::ModifyKind::Data(notify::event::DataChange::Any),
                            ));

                            *events[index] = event;
                        }
                        Err(err) => {
                            panic!("UNUSUAL SITUATION: watching manifest modified with {event:?} resulted in {err:?}");
                        }
                    }
                } else {
                    self.path_watcher_command_tx
                        .send(path_watcher::Command::Watch(path.clone()))
                        .unwrap();
                }
            } else {
                tracing::debug!("UNUSUAL REMOVE EVENT: {event:?}");
            }
        }

        events
    }
}

pub mod config {
    use std::{io, path::PathBuf};
    use syre_local::{
        common,
        error::IoSerde,
        file_resource::SystemResource,
        system::{
            collections::{ProjectManifest, UserManifest},
            config::Config as LocalConfig,
        },
    };

    #[derive(Clone, Debug)]
    pub struct Config {
        /// Path to the local user manifest file.
        /// Should be absolute.
        user_manifest: PathBuf,

        /// Path to the local project manifest file.
        /// Should be absolute.
        project_manifest: PathBuf,

        /// Path to the local config file.
        /// Should be absolute.
        local_config: PathBuf,
    }

    impl Config {
        /// # Notes
        /// + On Windows paths are converted to UNC.
        pub fn new(
            user_manifest: impl Into<PathBuf>,
            project_manifest: impl Into<PathBuf>,
            local_config: impl Into<PathBuf>,
        ) -> Self {
            if cfg!(target_os = "windows") {
                Self {
                    user_manifest: common::ensure_windows_unc(user_manifest),
                    project_manifest: common::ensure_windows_unc(project_manifest),
                    local_config: common::ensure_windows_unc(local_config),
                }
            } else {
                Self {
                    user_manifest: user_manifest.into(),
                    project_manifest: project_manifest.into(),
                    local_config: local_config.into(),
                }
            }
        }

        /// Creates an app config using the paths obtained from the system.
        pub fn try_default() -> Result<Self, io::Error> {
            Ok(Self {
                user_manifest: UserManifest::default_path()?,
                project_manifest: ProjectManifest::default_path()?,
                local_config: LocalConfig::default_path()?,
            })
        }

        pub fn user_manifest(&self) -> &PathBuf {
            &self.user_manifest
        }

        pub fn project_manifest(&self) -> &PathBuf {
            &self.project_manifest
        }

        pub fn local_config(&self) -> &PathBuf {
            &self.local_config
        }

        pub fn load_user_manifest(&self) -> Result<UserManifest, IoSerde> {
            UserManifest::load_from(self.user_manifest.clone())
        }

        pub fn load_project_manifest(&self) -> Result<ProjectManifest, IoSerde> {
            ProjectManifest::load_from(self.project_manifest.clone())
        }

        pub fn load_local_config(&self) -> Result<LocalConfig, IoSerde> {
            LocalConfig::load_from(self.local_config.clone())
        }
    }
}
