//! File system watcher.
// NB: `notify_debouncer_full` does a pretty good job of eliminating intermediate events.
// e.g. If a folder was created then moved, `notify_debouncer_full` will only emit
// a folder created event at the final path.
// However, there is still the chance for a race condition between the events being recieved
// and what is on disk.
// It is currenlty assumed that they are in sync.
use super::{actor::FileSystemActor, event as fs_event};
use crate::{
    command::WatcherCommand,
    error,
    event::{self as app, EventResult},
    Command, Error, Event, EventKind,
};
use crossbeam::channel::{Receiver, Sender};
use notify::event::{CreateKind, EventKind as NotifyEventKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, FileIdCache, FileIdMap};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use syre_local::common as local_common;

pub struct Builder {
    /// Sends events to the client.
    event_tx: Sender<EventResult>,

    // Recieve commands from the client.
    command_rx: Receiver<Command>,

    app_config: config::AppConfig,

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
        app_config: config::AppConfig,
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

    pub fn run(self) -> Result<(), crossbeam::channel::RecvError> {
        let (fs_tx, fs_rx) = crossbeam::channel::unbounded();
        let (fs_command_tx, fs_command_rx) = crossbeam::channel::unbounded();
        let mut file_system_actor = FileSystemActor::new(fs_tx, fs_command_rx);
        thread::Builder::new()
            .name("syre file system watcher actor".to_string())
            .spawn(move || file_system_actor.run())
            .unwrap();

        for path in self.paths.iter() {
            let (tx, rx) = crossbeam::channel::bounded(1);
            fs_command_tx
                .send(WatcherCommand::Watch {
                    path: path.clone(),
                    tx,
                })
                .unwrap();
            if let Err(err) = rx.recv()? {
                panic!("{err:?}");
            }
        }

        let watcher = FsWatcher {
            event_tx: self.event_tx,
            command_rx: self.command_rx,
            command_tx: fs_command_tx,
            event_rx: fs_rx,
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

    // NB: Must use own file id cache because the one being used by the notify watcher
    // is automatically updated on events recieved before we have access.
    // This means we lose the ability to get the file's id on destructive events
    // such as when a file is removed or moved from a location.
    // This cache is in the CommandInner and EventInner structs.
    /// Cache to hold file ids.
    file_ids: Arc<Mutex<FileIdMap>>,

    /// Project roots being watched.
    roots: Mutex<Vec<PathBuf>>,

    app_config: config::AppConfig,

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
            }
        }

        Ok(())
    }

    fn handle_command(&self, command: Command) {
        tracing::debug!(?command);
        match command {
            Command::Watch(path) => {
                assert!(path.is_absolute());
                let (tx, rx) = crossbeam::channel::bounded(1);
                self.command_tx
                    .send(WatcherCommand::Watch {
                        path: path.clone(),
                        tx,
                    })
                    .unwrap();

                // Only way for watch to fail is if relative path is given
                // but can not be canonicalized.
                // Because only absolute paths are accepted, watch should not fail.
                rx.recv().unwrap().unwrap();
                let mut roots = self.roots.lock().unwrap();
                if !roots.contains(&path) {
                    roots.push(path.clone());
                }

                let mut file_ids = self.file_ids.lock().unwrap();
                file_ids.add_root(path, notify::RecursiveMode::Recursive);
            }

            Command::Unwatch(path) => {
                assert!(path.is_absolute());
                let (tx, rx) = crossbeam::channel::bounded(1);
                self.command_tx
                    .send(WatcherCommand::Unwatch {
                        path: path.clone(),
                        tx,
                    })
                    .unwrap();

                // Only way for unwatch to fail is if relative path is given
                // but can not be canonicalized.
                // Because only absolute paths are accepted, watch should not fail.
                rx.recv().unwrap().unwrap();
                let mut roots = self.roots.lock().unwrap();
                roots.retain(|root| root != &path);

                let mut file_ids = self.file_ids.lock().unwrap();
                file_ids.remove_root(&path);
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

    fn handle_events(&self, events: DebounceEventResult) {
        let events = match events {
            Ok(events) => events,
            Err(errors) => {
                self.handle_event_errors(errors);
                return;
            }
        };

        if events.iter().any(|event| event.need_rescan()) {
            let mut file_ids = self.file_ids.lock().unwrap();
            file_ids.rescan();
            self.event_tx
                .send(Ok(vec![Event::new(EventKind::OutOfSync)]))
                .unwrap();
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
        tracing::debug!(?events);
        let (fs_events, mut errors) = self.process_events_notify_to_fs(&events);

        tracing::debug!(?fs_events, ?errors);
        let (app_events, app_errors) = self.process_events_fs_to_app(fs_events);

        tracing::debug!(?app_events, ?app_errors);
        errors.extend(app_errors);
        (app_events, errors)
    }
}

impl FsWatcher {
    /// Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](fs_event::Event)s.
    ///
    /// # Notes
    /// + Events are assumed to have already been preprocessed with paths rectified.
    ///
    /// # Returns
    /// Tuple of (events, errors).
    fn process_events_notify_to_fs<'a>(
        &'a self,
        events: &'a Vec<DebouncedEvent>,
    ) -> (Vec<fs_event::Event>, Vec<Error>) {
        let events = events.iter().collect::<Vec<_>>();
        let filtered_events = Self::filter_events(events.clone());
        let (grouped, remaining) = self.group_events(filtered_events);
        let (mut converted, errors) = self.convert_events(remaining);
        converted.extend(grouped);

        self.update_file_ids(events);
        (converted, errors)
    }

    /// Filters out uninteresting events.
    fn filter_events(events: Vec<&DebouncedEvent>) -> Vec<&DebouncedEvent> {
        events
            .into_iter()
            .filter(|event| match event.kind {
                NotifyEventKind::Create(_)
                | NotifyEventKind::Remove(_)
                | NotifyEventKind::Modify(ModifyKind::Data(_))
                | NotifyEventKind::Modify(ModifyKind::Name(_))
                | NotifyEventKind::Modify(ModifyKind::Any) => true,

                _ => false,
            })
            .filter(|event| {
                if let [path] = &event.paths[..] {
                    if let Some(file_name) = path.file_name() {
                        return file_name != ".DS_Store";
                    }
                }

                true
            })
            .collect()
    }

    /// Tries to convert all events into a single one.
    ///
    /// # Returns
    /// Tuple of (<converted events>, <unconverted events>).
    fn group_events<'a>(
        &'a self,
        events: Vec<&'a DebouncedEvent>,
    ) -> (Vec<fs_event::Event>, Vec<&'a DebouncedEvent>) {
        let mut remaining = Vec::with_capacity(events.len());
        let mut grouped = HashMap::with_capacity(events.len());
        for event in events {
            match event.kind {
                NotifyEventKind::Modify(ModifyKind::Name(RenameMode::From))
                | NotifyEventKind::Remove(_) => {
                    let file_ids = self.file_ids.lock().unwrap();
                    let Some(id) = file_ids.cached_file_id(&event.paths[0]).cloned() else {
                        remaining.push(event);
                        continue;
                    };

                    let entry = grouped.entry(id).or_insert(vec![]);
                    entry.push(event);
                }

                NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To))
                | NotifyEventKind::Create(_) => {
                    let id = match self.file_id_from_watcher(event.paths[0].clone()) {
                        Ok(id) => id,
                        Err(_err) => {
                            tracing::error!("could not retrieve id from watcher");
                            remaining.push(event);
                            continue;
                        }
                    };

                    let Some(id) = id else {
                        remaining.push(event);
                        continue;
                    };

                    let entry = grouped.entry(id).or_insert(vec![]);
                    entry.push(event);
                }

                NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                    let [path] = &event.paths[..] else {
                        panic!("invalid paths");
                    };

                    if path.exists() {
                        let id = match self.file_id_from_watcher(event.paths[0].clone()) {
                            Ok(id) => id,
                            Err(_err) => {
                                tracing::error!("could not retrieve id from watcher");
                                remaining.push(event);
                                continue;
                            }
                        };

                        let Some(id) = id else {
                            remaining.push(event);
                            continue;
                        };

                        let entry = grouped.entry(id).or_insert(vec![]);
                        entry.push(event);
                    } else {
                        let file_ids = self.file_ids.lock().unwrap();
                        let Some(id) = file_ids.cached_file_id(&event.paths[0]).cloned() else {
                            remaining.push(event);
                            continue;
                        };

                        let entry = grouped.entry(id).or_insert(vec![]);
                        entry.push(event);
                    }
                }

                _ => {
                    remaining.push(event);
                }
            }
        }

        let mut converted = Vec::with_capacity(grouped.len() / 2);
        for mut events in grouped.into_values() {
            events.sort_unstable_by_key(|event| event.time);
            match &events[..] {
                [e] => remaining.push(e),

                [e1, e2] => match (e1.kind, e2.kind) {
                    (
                        NotifyEventKind::Modify(ModifyKind::Name(RenameMode::From)),
                        NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To)),
                    )
                    | (
                        NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)),
                        NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)),
                    ) => {
                        let path_from = normalize_path_root(e1.paths[0].clone());
                        let path_to = normalize_path_root(e2.paths[0].clone());
                        if path_from.parent() == path_to.parent() {
                            if path_to.is_file() {
                                converted.push(
                                    fs_event::Event::new(
                                        fs_event::File::Renamed {
                                            from: path_from,
                                            to: path_to,
                                        },
                                        e2.time,
                                    )
                                    .add_parent(e1)
                                    .add_parent(e2),
                                );
                            } else if path_to.is_dir() {
                                converted.push(
                                    fs_event::Event::new(
                                        fs_event::Folder::Renamed {
                                            from: path_from,
                                            to: path_to,
                                        },
                                        e2.time,
                                    )
                                    .add_parent(e1)
                                    .add_parent(e2),
                                )
                            } else {
                                remaining.push(e1);
                                remaining.push(e2);
                            }
                        } else {
                            if path_to.is_file() {
                                converted.push(
                                    fs_event::Event::new(
                                        fs_event::File::Moved {
                                            from: path_from,
                                            to: path_to,
                                        },
                                        e2.time,
                                    )
                                    .add_parent(e1)
                                    .add_parent(e2),
                                );
                            } else if path_to.is_dir() {
                                converted.push(
                                    fs_event::Event::new(
                                        fs_event::Folder::Moved {
                                            from: path_from,
                                            to: path_to,
                                        },
                                        e2.time,
                                    )
                                    .add_parent(e1)
                                    .add_parent(e2),
                                )
                            } else {
                                remaining.push(e1);
                                remaining.push(e2);
                            }
                        }
                    }
                    (
                        NotifyEventKind::Remove(RemoveKind::File),
                        NotifyEventKind::Create(CreateKind::File),
                    ) => {
                        let path_from = normalize_path_root(e1.paths[0].clone());
                        let path_to = normalize_path_root(e2.paths[0].clone());
                        if path_from.parent() == path_to.parent() {
                            converted.push(
                                fs_event::Event::new(
                                    fs_event::File::Renamed {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                )
                                .add_parent(e1)
                                .add_parent(e2),
                            );
                        } else {
                            converted.push(
                                fs_event::Event::new(
                                    fs_event::File::Moved {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                )
                                .add_parent(e1)
                                .add_parent(e2),
                            );
                        }
                    }
                    (
                        NotifyEventKind::Remove(RemoveKind::Folder),
                        NotifyEventKind::Create(CreateKind::Folder),
                    ) => {
                        let path_from = normalize_path_root(e1.paths[0].clone());
                        let path_to = normalize_path_root(e2.paths[0].clone());
                        if path_from.parent() == path_to.parent() {
                            converted.push(
                                fs_event::Event::new(
                                    fs_event::Folder::Renamed {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                )
                                .add_parent(e1)
                                .add_parent(e2),
                            );
                        } else {
                            converted.push(
                                fs_event::Event::new(
                                    fs_event::Folder::Moved {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                )
                                .add_parent(e1)
                                .add_parent(e2),
                            );
                        }
                    }

                    _ => {
                        remaining.extend(events);
                    }
                },

                _ => {
                    remaining.extend(events);
                }
            }
        }

        (converted, remaining)
    }

    fn convert_events<'a>(
        &'a self,
        events: Vec<&'a DebouncedEvent>,
    ) -> (Vec<fs_event::Event>, Vec<Error>) {
        enum ConversionResult<'a> {
            Ok(fs_event::Event<'a>),
            Err {
                event: &'a DebouncedEvent,
                kind: error::Process,
            },
        }

        let (converted, errors): (Vec<_>, Vec<_>) = events
            .into_iter()
            .filter_map(|event| match self.convert_event(&event) {
                Ok(event) => event.map(|event| ConversionResult::Ok(event)),
                Err(kind) => Some(ConversionResult::Err { event, kind }),
            })
            .partition(|event| match event {
                ConversionResult::Ok(_) => true,
                ConversionResult::Err { .. } => false,
            });

        let converted = converted
            .into_iter()
            .map(|event| match event {
                ConversionResult::Ok(event) => event,
                _ => unreachable!("events are partitioned"),
            })
            .collect();

        let errors = errors
            .into_iter()
            .map(|event| match event {
                ConversionResult::Err { event, kind } => Error::Processing {
                    events: vec![event.clone()],
                    kind,
                },
                _ => unreachable!("events are partitioned"),
            })
            .collect();

        (converted, errors)
    }

    fn convert_event(
        &self,
        event: &DebouncedEvent,
    ) -> Result<Option<fs_event::Event>, error::Process> {
        let time = event.time;
        let event = match event.kind {
            NotifyEventKind::Create(CreateKind::File) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                Some(fs_event::Event::new(fs_event::File::Created(path), time))
            }

            NotifyEventKind::Create(CreateKind::Folder) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                Some(fs_event::Event::new(fs_event::Folder::Created(path), time))
            }

            NotifyEventKind::Create(CreateKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                if path.is_file() {
                    Some(fs_event::Event::new(fs_event::File::Created(path), time))
                } else if path.is_dir() {
                    Some(fs_event::Event::new(fs_event::Folder::Created(path), time))
                } else {
                    return Err(error::Process::UnknownFileType);
                }
            }

            NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                let [from, to] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let to = fs::canonicalize(to).unwrap();
                let from = normalize_path_root(from);
                if to.is_file() {
                    if to.parent() == from.parent() {
                        Some(fs_event::Event::new(
                            fs_event::File::Renamed { from, to },
                            time,
                        ))
                    } else {
                        Some(fs_event::Event::new(
                            fs_event::File::Moved { from, to },
                            time,
                        ))
                    }
                } else if to.is_dir() {
                    if to.parent() == from.parent() {
                        Some(fs_event::Event::new(
                            fs_event::Folder::Renamed { from, to },
                            time,
                        ))
                    } else {
                        Some(fs_event::Event::new(
                            fs_event::Folder::Moved { from, to },
                            time,
                        ))
                    }
                } else {
                    return Err(error::Process::UnknownFileType);
                }
            }

            NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                #[cfg(not(target_os = "macos"))]
                todo!();

                match &event.paths[..] {
                    [path] => {
                        if path.exists() {
                            if path.is_file() {
                                let path = fs::canonicalize(path).unwrap();
                                Some(fs_event::Event::new(fs_event::File::Other(path), time))
                            } else if path.is_dir() {
                                let path = fs::canonicalize(path).unwrap();
                                Some(fs_event::Event::new(fs_event::Folder::Other(path), time))
                            } else {
                                return Err(error::Process::UnknownFileType);
                            }
                        } else {
                            // TODO Could check file ids to get if file or folder.
                            Some(fs_event::Event::new(
                                fs_event::Any::Removed(path.clone()),
                                time,
                            ))
                        }
                    }

                    paths => todo!("unknown paths pattern: {paths:?}"),
                }
            }

            NotifyEventKind::Modify(ModifyKind::Data(_)) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = match fs::canonicalize(path) {
                    Ok(path) => path,
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            return Err(error::Process::NotFound);
                        }
                        _ => {
                            return Err(error::Process::Canonicalize);
                        }
                    },
                };

                if path.is_file() {
                    Some(fs_event::Event::new(
                        fs_event::File::DataModified(path),
                        time,
                    ))
                } else {
                    None
                }
            }

            NotifyEventKind::Modify(ModifyKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = match fs::canonicalize(path) {
                    Ok(path) => path,
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            return Err(error::Process::NotFound);
                        }

                        _ => {
                            return Err(error::Process::Canonicalize);
                        }
                    },
                };

                if path.is_file() {
                    Some(fs_event::Event::new(fs_event::File::Other(path), time))
                } else if path.is_dir() {
                    Some(fs_event::Event::new(fs_event::Folder::Other(path), time))
                } else {
                    return Err(error::Process::UnknownFileType);
                }
            }

            NotifyEventKind::Remove(RemoveKind::File) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = normalize_path_root(path);
                Some(fs_event::Event::new(fs_event::File::Removed(path), time))
            }

            NotifyEventKind::Remove(RemoveKind::Folder) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = normalize_path_root(path);
                Some(fs_event::Event::new(fs_event::Folder::Removed(path), time))
            }

            NotifyEventKind::Remove(RemoveKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = normalize_path_root(path);
                Some(fs_event::Event::new(fs_event::Any::Removed(path), time))
            }

            event => unreachable!("unhandled event {event:?}"),
        };

        Ok(event)
    }

    /// Update file id cache based on events.
    /// See notify_debouncer_full::DebounceDataInner::add_event.
    fn update_file_ids(&self, events: Vec<&DebouncedEvent>) {
        let mut file_ids = self.file_ids.lock().unwrap();
        for event in events {
            let path = &event.paths[0];
            match event.kind {
                NotifyEventKind::Create(_) => file_ids.add_path(path),
                NotifyEventKind::Remove(_) => file_ids.remove_path(path),
                NotifyEventKind::Modify(ModifyKind::Name(rename_mode)) => match rename_mode {
                    RenameMode::Any => {
                        if path.exists() {
                            file_ids.add_path(path);
                        } else {
                            file_ids.remove_path(path);
                        }
                    }

                    RenameMode::Both => {
                        file_ids.remove_path(&event.paths[0]);
                        file_ids.add_path(&event.paths[1]);
                    }

                    RenameMode::From => {
                        file_ids.remove_path(path);
                    }

                    RenameMode::To => {
                        file_ids.add_path(path);
                    }

                    RenameMode::Other => {
                        // ignored
                    }
                },

                _ => {
                    if file_ids.cached_file_id(path).is_none() {
                        file_ids.add_path(path);
                    }
                }
            }
        }
    }

    fn file_id_from_watcher(&self, path: PathBuf) -> StdResult<Option<file_id::FileId>, ()> {
        let (tx, rx) = crossbeam::channel::bounded(1);
        if let Err(_err) = self.command_tx.send(WatcherCommand::FileId { path, tx }) {
            return Err(());
        }

        let id = match rx.recv() {
            Ok(id) => id,
            Err(_err) => {
                return Err(());
            }
        };

        Ok(id)
    }
}

impl FsWatcher {
    /// Convert [file system events](fs_event::Event) to [app events](Event).
    ///
    /// # Returns
    /// Tuple of (events, errors).
    fn process_events_fs_to_app(&self, events: Vec<fs_event::Event>) -> (Vec<Event>, Vec<Error>) {
        let (converted, errors): (Vec<_>, Vec<_>) = events
            .into_iter()
            .map(|fs_event| {
                self.process_event_fs_to_apps(&fs_event).map_err(|err| {
                    let events = fs_event
                        .parents()
                        .into_iter()
                        .map(|parent| parent.clone())
                        .collect::<Vec<_>>();

                    Error::Processing {
                        events,
                        kind: err.into(),
                    }
                })
            })
            .partition(|event| event.is_ok());

        let converted = converted
            .into_iter()
            .flat_map(|events| match events {
                Ok(events) => events,
                _ => unreachable!("elements have been partitioned"),
            })
            .collect();

        let errors = errors
            .into_iter()
            .map(|error| match error {
                Err(error) => error,
                _ => unreachable!("elements have been partitioned"),
            })
            .collect();

        (converted, errors)
    }

    fn process_event_fs_to_apps(
        &self,
        event: &fs_event::Event,
    ) -> StdResult<Vec<Event>, error::processing::Error> {
        let events = match &event.kind {
            fs_event::EventKind::File(fs_event::File::Created(path)) => {
                let event = match self.handle_file_created(&path, &self.app_config) {
                    Ok(kind) => Event::with_time(kind, event.time).add_path(path.clone()),
                    Err(err) => {
                        Event::with_time(EventKind::File(app::ResourceEvent::Created), event.time)
                            .add_path(path.clone())
                    }
                };

                vec![event]
            }

            fs_event::EventKind::File(fs_event::File::Removed(path)) => {
                let event = match self.handle_file_removed(&path, &self.app_config) {
                    Ok(kind) => Event::with_time(kind, event.time).add_path(path.clone()),
                    Err(err) => {
                        Event::with_time(EventKind::File(app::ResourceEvent::Removed), event.time)
                            .add_path(path.clone())
                    }
                };

                vec![event]
            }

            fs_event::EventKind::File(fs_event::File::Moved { from, to }) => {
                Self::handle_file_moved(from.clone(), to.clone(), event.time, &self.app_config)
            }

            fs_event::EventKind::File(fs_event::File::Renamed { from, to }) => {
                Self::handle_file_renamed(from.clone(), to.clone(), event.time, &self.app_config)?
            }

            fs_event::EventKind::File(fs_event::File::DataModified(path)) => {
                let event = match Self::handle_file_data_modified(&path, &self.app_config) {
                    Ok(kind) => Event::with_time(kind, event.time).add_path(path.clone()),
                    Err(err) => {
                        Event::with_time(EventKind::File(app::ResourceEvent::Removed), event.time)
                            .add_path(path.clone())
                    }
                };

                vec![event]
            }

            fs_event::EventKind::File(fs_event::File::Other(path)) => vec![Event::with_time(
                EventKind::File(app::ResourceEvent::Modified(app::ModifiedKind::Other)),
                event.time,
            )
            .add_path(path.clone())],

            fs_event::EventKind::Folder(fs_event::Folder::Created(path)) => {
                let event = match self.handle_folder_created(&path) {
                    Ok(kind) => Event::with_time(kind, event.time).add_path(path.clone()),
                    Err(err) => {
                        tracing::error!(?err);
                        Event::with_time(EventKind::Folder(app::ResourceEvent::Created), event.time)
                            .add_path(path.clone())
                    }
                };

                vec![event]
            }

            fs_event::EventKind::Folder(fs_event::Folder::Removed(path)) => {
                let event = match self.handle_folder_removed(&path) {
                    Ok(kind) => Event::with_time(kind, event.time).add_path(path.clone()),
                    Err(err) => {
                        if matches!(err.kind(), resources::ErrorKind::NotInProject) {
                            if let Ok(manifest) = self.app_config.load_project_manifest() {
                                if manifest.contains(&path) {
                                    return Ok(vec![Event::with_time(
                                        app::Project::Removed.into(),
                                        event.time,
                                    )
                                    .add_path(path.clone())]);
                                }

                                if let Some(parent) = path.parent() {
                                    let parent = parent.to_path_buf();
                                    if manifest.contains(&parent) {
                                        if let Some(file_name) = path.file_name() {
                                            if file_name == local_common::app_dir() {
                                                return Ok(vec![Event::with_time(
                                                    app::Project::ConfigDir(
                                                        app::StaticResourceEvent::Removed,
                                                    )
                                                    .into(),
                                                    event.time,
                                                )
                                                .add_path(path.clone())]);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        tracing::error!(?err);
                        Event::with_time(
                            app::EventKind::Folder(app::ResourceEvent::Removed),
                            event.time,
                        )
                    }
                };

                vec![event]
            }

            fs_event::EventKind::Folder(fs_event::Folder::Moved { from, to }) => {
                self.handle_folder_moved(from.clone(), to.clone(), event.time)
            }

            fs_event::EventKind::Folder(fs_event::Folder::Renamed { from, to }) => {
                assert!(
                    from.parent() == to.parent(),
                    "renamed paths should have same parent"
                );

                self.handle_folder_renamed(from.clone(), to.clone(), event.time)?
            }

            fs_event::EventKind::Folder(fs_event::Folder::Other(path)) => vec![Event::with_time(
                app::EventKind::Folder(app::ResourceEvent::Modified(app::ModifiedKind::Other)),
                event.time,
            )
            .add_path(path.clone())],

            fs_event::EventKind::Any(fs_event::Any::Removed(path)) => {
                // TODO Could check file ids to get if path is file or dir.
                vec![Event::with_time(app::Any::Removed.into(), event.time).add_path(path.clone())]
            }
        };

        Ok(events)
    }

    fn handle_file_created(
        &self,
        path: &PathBuf,
        app_config: &config::AppConfig,
    ) -> StdResult<EventKind, resources::Error> {
        let kind = match resources::resource_kind(path, app_config) {
            Ok(Some(kind)) => Self::convert_resource_to_event_kind_created(kind),
            Ok(None) => EventKind::File(app::ResourceEvent::Created),
            Err(err) => match err.kind() {
                resources::error::ErrorKind::NotInProject => {
                    let roots = self.roots.lock().unwrap();
                    let project = roots
                        .iter()
                        .find(|project| path.starts_with(project))
                        .expect("event should not be triggered if not in a root");

                    assert_ne!(
                        *path,
                        local_common::project_file_of(&project),
                        "NotInProject error indicates project file does not exist"
                    );
                    if *path == local_common::project_settings_file_of(&project) {
                        app::Project::Settings(app::StaticResourceEvent::Created).into()
                    } else if *path == local_common::analyses_file_of(&project) {
                        app::Project::Analysis(app::StaticResourceEvent::Created).into()
                    } else {
                        return Err(err);
                    }
                }
                resources::error::ErrorKind::LoadProject(_) => {
                    let project = syre_local::project::project::project_root_path(path)
                        .expect("LoadProject error indicates we are in a project");

                    if *path == local_common::project_file_of(&project) {
                        app::Project::Properties(app::StaticResourceEvent::Created).into()
                    } else if *path == local_common::project_settings_file_of(&project) {
                        app::Project::Settings(app::StaticResourceEvent::Created).into()
                    } else if *path == local_common::analyses_file_of(&project) {
                        app::Project::Analysis(app::StaticResourceEvent::Created).into()
                    } else {
                        return Err(err);
                    }
                }
                _ => return Err(err),
            },
        };

        Ok(kind)
    }

    fn handle_file_removed(
        &self,
        path: &PathBuf,
        app_config: &config::AppConfig,
    ) -> StdResult<EventKind, resources::Error> {
        let kind = match resources::resource_kind(path, app_config) {
            Ok(Some(kind)) => Self::convert_resource_to_event_kind_removed(kind),
            Ok(None) => EventKind::File(app::ResourceEvent::Removed),
            Err(err) => match err.kind() {
                resources::error::ErrorKind::NotInProject => {
                    let roots = self.roots.lock().unwrap();
                    let project = roots
                        .iter()
                        .find(|project| path.starts_with(project))
                        .expect("event should not be triggered if not in a root");

                    if *path == local_common::project_file_of(&project) {
                        app::Project::Properties(app::StaticResourceEvent::Removed).into()
                    } else if *path == local_common::project_settings_file_of(&project) {
                        app::Project::Settings(app::StaticResourceEvent::Removed).into()
                    } else if *path == local_common::analyses_file_of(&project) {
                        app::Project::Analysis(app::StaticResourceEvent::Removed).into()
                    } else {
                        return Err(err);
                    }
                }
                resources::error::ErrorKind::LoadProject(_) => {
                    let project = syre_local::project::project::project_root_path(path)
                        .expect("LoadProject error indicates the path is in a project");

                    assert_ne!(
                        *path,
                        local_common::project_file_of(&project),
                        "LoadProject error indicates the path is in a project, requiring a project file to be present."
                    );
                    if *path == local_common::project_settings_file_of(&project) {
                        app::Project::Settings(app::StaticResourceEvent::Removed).into()
                    } else if *path == local_common::analyses_file_of(&project) {
                        app::Project::Analysis(app::StaticResourceEvent::Removed).into()
                    } else {
                        return Err(err);
                    }
                }
                _ => return Err(err),
            },
        };

        Ok(kind)
    }

    fn handle_file_moved(
        from: PathBuf,
        to: PathBuf,
        time: Instant,
        app_config: &config::AppConfig,
    ) -> Vec<Event> {
        let from_kind = resources::resource_kind(&from, app_config);
        let to_kind = resources::resource_kind(&to, app_config);
        match (from_kind, to_kind) {
            (Err(from_err), Err(to_err)) => {
                vec![
                    Event::with_time(EventKind::File(app::ResourceEvent::Moved), time)
                        .add_path(from.clone())
                        .add_path(to.clone()),
                ]
            }

            (Ok(from_kind), Err(to_err)) => {
                if let Some(from_kind) = from_kind {
                    let kind = Self::convert_resource_to_event_kind_moved_from(from_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                } else {
                    vec![
                        Event::with_time(EventKind::File(app::ResourceEvent::Moved), time)
                            .add_path(from.clone())
                            .add_path(to.clone()),
                    ]
                }
            }

            (Err(from_err), Ok(to_kind)) => {
                if let Some(to_kind) = to_kind {
                    let kind = Self::convert_resource_to_event_kind_moved_to(to_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                } else {
                    vec![
                        Event::with_time(EventKind::File(app::ResourceEvent::Moved), time)
                            .add_path(from.clone())
                            .add_path(to.clone()),
                    ]
                }
            }

            (Ok(from_kind), Ok(to_kind)) => match (from_kind, to_kind) {
                (None, None) => {
                    vec![
                        Event::with_time(EventKind::File(app::ResourceEvent::Moved), time)
                            .add_path(from.clone())
                            .add_path(to.clone()),
                    ]
                }

                (Some(from_kind), None) => {
                    let kind = Self::convert_resource_to_event_kind_moved_from(from_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                }

                (None, Some(to_kind)) => {
                    let kind = Self::convert_resource_to_event_kind_moved_to(to_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                }

                (Some(from_kind), Some(to_kind)) => {
                    Self::convert_resource_to_event_kind_moved(from_kind, to_kind, from, to, time)
                }
            },
        }
    }

    fn handle_file_renamed(
        from: PathBuf,
        to: PathBuf,
        time: Instant,
        app_config: &config::AppConfig,
    ) -> StdResult<Vec<Event>, error::processing::Error> {
        let from_kind = resources::resource_kind(&from, app_config);
        let to_kind = resources::resource_kind(&to, app_config);
        match (from_kind, to_kind) {
            (Err(from_err), Err(to_err)) => {
                if to_err.kind() != from_err.kind() {
                    return Err(error::processing::Error::InvalidState(format!(
                        "rename errors differ. from: {from_err:?}. to: {to_err:?}."
                    )));
                }

                let event = Event::with_time(EventKind::File(app::ResourceEvent::Renamed), time)
                    .add_path(from.clone())
                    .add_path(to.clone());

                Ok(vec![event])
            }

            (Ok(_from_kind), Err(to_err)) => {
                return Err(error::processing::Error::InvalidState(format!(
                    "rename errors differ. from: Ok. to: {to_err:?}."
                )));
            }

            (Err(from_err), Ok(_to_kind)) => {
                return Err(error::processing::Error::InvalidState(format!(
                    "rename errors differ. from: {from_err:?}. to: Ok."
                )));
            }

            (Ok(from_kind), Ok(to_kind)) => match (from_kind, to_kind) {
                (None, None) => Ok(vec![Event::with_time(
                    EventKind::File(app::ResourceEvent::Renamed),
                    time,
                )
                .add_path(from.clone())
                .add_path(to.clone())]),

                (Some(from_kind), None) => {
                    let kind = Self::convert_resource_to_event_kind_renamed_from(from_kind);
                    Ok(vec![Event::with_time(kind, time)
                        .add_path(from)
                        .add_path(to)])
                }

                (None, Some(to_kind)) => {
                    let kind = Self::convert_resource_to_event_kind_renamed_to(to_kind);
                    Ok(vec![Event::with_time(kind, time)
                        .add_path(from)
                        .add_path(to)])
                }

                (Some(from_kind), Some(to_kind)) => {
                    Self::convert_resource_to_event_kind_renamed(from_kind, to_kind, from, to, time)
                }
            },
        }
    }

    fn handle_file_data_modified(
        path: &PathBuf,
        app_config: &config::AppConfig,
    ) -> StdResult<EventKind, resources::Error> {
        let kind = match resources::resource_kind(path, app_config)? {
            Some(kind) => Self::convert_resource_to_event_kind_data_modified(kind),
            None => app::EventKind::File(app::ResourceEvent::Modified(app::ModifiedKind::Data)),
        };

        Ok(kind)
    }

    fn handle_folder_created(&self, path: &PathBuf) -> StdResult<EventKind, resources::Error> {
        use resources::error::ErrorKind;

        assert!(path.exists());
        let kind = match resources::dir_kind(path) {
            Ok(resources::DirKind::None { .. }) => {
                app::EventKind::Folder(app::ResourceEvent::Created)
            }
            Ok(resources::DirKind::ContainerLike { .. }) => {
                if local_common::container_file_of(path).exists() {
                    app::Graph::Created.into()
                } else {
                    app::EventKind::Folder(app::ResourceEvent::Created)
                }
            }
            Ok(kind) => Self::convert_dir_to_event_kind_created(&kind),
            Err(err) if matches!(err.kind(), resources::error::ErrorKind::NotInProject) => {
                let projects = match self.app_config.load_project_manifest() {
                    Ok(projects) => projects,
                    Err(err) => {
                        return Err(resources::error::Error::new(
                            path.clone(),
                            ErrorKind::LoadProjectManifest(err),
                        ));
                    }
                };

                let Some(project) = projects.iter().find(|project| path.starts_with(project))
                else {
                    return Err(resources::error::Error::new(
                        path.clone(),
                        ErrorKind::NotInProject,
                    ));
                };

                if path == project {
                    app::Project::Created.into()
                } else if path == &syre_local::common::app_dir_of(project) {
                    app::Project::ConfigDir(app::StaticResourceEvent::Created).into()
                } else {
                    return Err(resources::error::Error::new(
                        path.clone(),
                        ErrorKind::NotInProject,
                    ));
                }
            }
            Err(err) => return Err(err),
        };

        Ok(kind)
    }

    fn handle_folder_removed(&self, path: &PathBuf) -> StdResult<EventKind, resources::Error> {
        assert!(!path.exists());
        let kind = match resources::dir_kind(path)? {
            resources::DirKind::None { .. } => app::EventKind::Folder(app::ResourceEvent::Removed),
            kind => Self::convert_dir_to_event_kind_removed(&kind),
        };

        Ok(kind)
    }

    /// Handles a moved folder
    fn handle_folder_moved(&self, from: PathBuf, to: PathBuf, time: Instant) -> Vec<Event> {
        assert!(
            from.parent() != to.parent(),
            "moved paths should have different parent"
        );

        let from_kind = resources::dir_kind(&from);
        let to_kind = resources::dir_kind(&to);
        match (from_kind, to_kind) {
            (Err(from_err), Err(to_err)) => {
                vec![
                    Event::with_time(EventKind::Folder(app::ResourceEvent::Moved), time)
                        .add_path(from.clone())
                        .add_path(to.clone()),
                ]
            }

            (Ok(from_kind), Err(to_err)) => {
                assert!(!matches!(from_kind, resources::DirKind::Container { .. }));
                if matches!(
                    from_kind,
                    resources::DirKind::Project {
                        kind: resources::ProjectDir::Root,
                        ..
                    }
                ) {
                    tracing::warn!("UNUSUAL SITUATION: project moved and replaced");
                    vec![
                        Event::with_time(app::Project::Moved.into(), time)
                            .add_path(from.clone())
                            .add_path(to.clone()),
                        Event::with_time(app::Project::Modified.into(), time).add_path(from),
                    ]
                } else if matches!(to_err.kind(), resources::ErrorKind::NotInProject) {
                    let kind = Self::convert_dir_to_event_kind_moved_from_project(&from_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                } else {
                    vec![
                        Event::with_time(EventKind::Folder(app::ResourceEvent::Moved), time)
                            .add_path(from)
                            .add_path(to),
                    ]
                }
            }

            (Err(from_err), Ok(to_kind)) => {
                if matches!(
                    to_kind,
                    resources::DirKind::Project {
                        kind: resources::ProjectDir::Root,
                        ..
                    },
                ) {
                    vec![Event::with_time(app::Project::Moved.into(), time)
                        .add_path(from)
                        .add_path(to)]
                } else if matches!(from_err.kind(), resources::ErrorKind::NotInProject) {
                    let kind = Self::convert_dir_to_event_kind_moved_to_project(&to_kind);
                    vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                } else {
                    vec![
                        Event::with_time(EventKind::Folder(app::ResourceEvent::Moved), time)
                            .add_path(from)
                            .add_path(to),
                    ]
                }
            }

            (Ok(from_kind), Ok(to_kind)) => {
                assert!(!matches!(from_kind, resources::DirKind::Container { .. }));
                match (from_kind, to_kind) {
                    (resources::DirKind::None { .. }, resources::DirKind::None { .. }) => {
                        vec![
                            Event::with_time(EventKind::Folder(app::ResourceEvent::Moved), time)
                                .add_path(from.clone())
                                .add_path(to.clone()),
                        ]
                    }

                    (from_kind, resources::DirKind::None { .. }) => {
                        let kind = Self::convert_dir_to_event_kind_moved_from(&from_kind);
                        vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                    }

                    (
                        resources::DirKind::None {
                            project: from_project,
                        },
                        to_kind,
                    ) => match to_kind {
                        resources::DirKind::ContainerLike {
                            project: to_project,
                        } => {
                            if from_project == to_project {
                                vec![Event::with_time(
                                    EventKind::Folder(app::ResourceEvent::Moved),
                                    time,
                                )
                                .add_path(from.clone())
                                .add_path(to.clone())]
                            } else {
                                vec![
                                    Event::with_time(
                                        EventKind::Folder(app::ResourceEvent::Removed),
                                        time,
                                    )
                                    .add_path(from.clone()),
                                    Event::with_time(
                                        EventKind::Folder(app::ResourceEvent::Created),
                                        time,
                                    )
                                    .add_path(to.clone()),
                                ]
                            }
                        }
                        _ => {
                            let kind = Self::convert_dir_to_event_kind_moved_to(&to_kind);
                            vec![Event::with_time(kind, time).add_path(from).add_path(to)]
                        }
                    },

                    (from_kind, to_kind) => {
                        Self::convert_dir_to_event_kind_moved(from_kind, to_kind, from, to, time)
                    }
                }
            }
        }
    }

    fn handle_folder_renamed(
        &self,
        from: PathBuf,
        to: PathBuf,
        time: Instant,
    ) -> StdResult<Vec<Event>, error::processing::Error> {
        assert!(
            from.parent() == to.parent(),
            "renamed paths should have same parent"
        );

        let from_kind = resources::dir_kind(&from);
        let to_kind = resources::dir_kind(&to);
        match (from_kind, to_kind) {
            (Err(from_err), Err(to_err)) => {
                if to_err.kind() != from_err.kind() {
                    return Err(error::processing::Error::InvalidState(format!(
                        "rename errors differ. from: {from_err:?}. to: {to_err:?}."
                    )));
                }

                let event = Event::with_time(EventKind::Folder(app::ResourceEvent::Renamed), time)
                    .add_path(from)
                    .add_path(to);

                Ok(vec![event])
            }

            (Ok(from_kind), Err(to_err)) => {
                assert!(!matches!(from_kind, resources::DirKind::Container { .. }));
                if matches!(
                    (from_kind, to_err.kind()),
                    (
                        resources::DirKind::Project {
                            kind: resources::ProjectDir::Root,
                            ..
                        },
                        resources::ErrorKind::NotInProject
                    )
                ) {
                    Ok(vec![
                        Event::with_time(app::Project::Modified.into(), time).add_path(from),
                        Event::with_time(app::Project::Modified.into(), time).add_path(to),
                    ])
                } else {
                    return Err(error::processing::Error::InvalidState(format!(
                        "rename errors differ. from: Ok. to: {to_err:?}."
                    )));
                }
            }

            (Err(from_err), Ok(to_kind)) => {
                if matches!(
                    (from_err.kind(), to_kind),
                    (
                        resources::ErrorKind::NotInProject,
                        resources::DirKind::Project {
                            kind: resources::ProjectDir::Root,
                            ..
                        }
                    )
                ) {
                    Ok(vec![Event::with_time(app::Project::Moved.into(), time)
                        .add_path(from)
                        .add_path(to)])
                } else {
                    return Err(error::processing::Error::InvalidState(format!(
                        "rename errors differ. from: {from_err:?}. to: Ok."
                    )));
                }
            }

            (Ok(from_kind), Ok(to_kind)) => {
                assert!(!matches!(from_kind, resources::DirKind::Container { .. }));
                match (from_kind, to_kind) {
                    (resources::DirKind::None { .. }, resources::DirKind::None { .. }) => {
                        Ok(vec![Event::with_time(
                            EventKind::Folder(app::ResourceEvent::Renamed),
                            time,
                        )
                        .add_path(from.clone())
                        .add_path(to.clone())])
                    }

                    (from_kind, resources::DirKind::None { .. }) => {
                        assert!(
                            !matches!(
                                from_kind,
                                resources::DirKind::Project {
                                    kind: resources::ProjectDir::Root,
                                    ..
                                }
                            ),
                            "renaming project should result in destination being a project"
                        );
                        assert!(
                            !matches!(from_kind, resources::DirKind::ContainerLike {  .. }),
                            "renaming a container like should not result in it not beign a resource");

                        match from_kind {
                            resources::DirKind::Project {
                                kind: resources::ProjectDir::Analysis,
                                ..
                            } => Ok(vec![Event::with_time(
                                app::Project::AnalysisDir(app::ResourceEvent::Renamed).into(),
                                time,
                            )
                            .add_path(from)
                            .add_path(to)]),

                            resources::DirKind::Project {
                                kind: resources::ProjectDir::Data,
                                ..
                            } => Ok(vec![Event::with_time(
                                app::Project::DataDir(app::ResourceEvent::Renamed).into(),
                                time,
                            )
                            .add_path(from)
                            .add_path(to)]),
                            _ => {
                                let kind =
                                    Self::convert_dir_to_event_kind_renamed_from(&from_kind)?;
                                Ok(vec![Event::with_time(kind, time)
                                    .add_path(from)
                                    .add_path(to)])
                            }
                        }
                    }

                    (resources::DirKind::None { .. }, to_kind) => {
                        assert!(
                            !matches!(
                                to_kind,
                                resources::DirKind::Project {
                                    kind: resources::ProjectDir::Root,
                                    ..
                                }
                            ),
                            "renaming project should result in error at original location"
                        );

                        let kind = Self::convert_dir_to_event_kind_renamed_to(&to_kind);
                        Ok(vec![Event::with_time(kind, time)
                            .add_path(from)
                            .add_path(to)])
                    }

                    (from_kind, to_kind) => {
                        Self::convert_dir_to_event_kind_renamed(from_kind, to_kind, from, to, time)
                    }
                }
            }
        }
    }
}

impl FsWatcher {
    fn convert_resource_to_event_kind_created(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => EventKind::Config(
                    app::Config::ProjectManifest(app::StaticResourceEvent::Created),
                ),
                resources::Config::UserManifest => {
                    EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Created))
                }
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => {
                    app::Project::Properties(app::StaticResourceEvent::Created).into()
                }

                resources::Project::Settings => {
                    app::Project::Settings(app::StaticResourceEvent::Created).into()
                }

                resources::Project::Analysis => {
                    app::Project::Analysis(app::StaticResourceEvent::Created).into()
                }
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => {
                    app::Container::Properties(app::StaticResourceEvent::Created).into()
                }

                resources::Container::Settings => {
                    app::Container::Settings(app::StaticResourceEvent::Created).into()
                }

                resources::Container::Assets => {
                    app::Container::Assets(app::StaticResourceEvent::Created).into()
                }
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Created)
            }

            resources::ResourceEvent::Asset { .. } => {
                app::EventKind::AssetFile(app::ResourceEvent::Created)
            }
        }
    }

    fn convert_resource_to_event_kind_removed(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => EventKind::Config(
                    app::Config::ProjectManifest(app::StaticResourceEvent::Removed),
                ),
                resources::Config::UserManifest => {
                    EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Removed))
                }
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => {
                    app::Project::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Settings => {
                    app::Project::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Analysis => {
                    app::Project::Analysis(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => {
                    app::Container::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Settings => {
                    app::Container::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Assets => {
                    app::Container::Assets(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Removed)
            }

            resources::ResourceEvent::Asset { .. } => {
                app::EventKind::AssetFile(app::ResourceEvent::Removed)
            }
        }
    }

    fn convert_resource_to_event_kind_moved_from(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => EventKind::Config(
                    app::Config::ProjectManifest(app::StaticResourceEvent::Removed),
                ),
                resources::Config::UserManifest => {
                    EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Removed))
                }
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => {
                    app::Project::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Settings => {
                    app::Project::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Analysis => {
                    app::Project::Analysis(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => {
                    app::Container::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Settings => {
                    app::Container::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Assets => {
                    app::Container::Assets(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Removed)
            }

            resources::ResourceEvent::Asset { .. } => {
                app::EventKind::AssetFile(app::ResourceEvent::Removed)
            }
        }
    }

    fn convert_resource_to_event_kind_moved_to(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => {
                    EventKind::Config(app::Config::ProjectManifest(
                        app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                    ))
                }

                resources::Config::UserManifest => EventKind::Config(app::Config::UserManifest(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )),
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => app::Project::Properties(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),

                resources::Project::Settings => app::Project::Settings(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),

                resources::Project::Analysis => app::Project::Analysis(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => app::Container::Properties(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),

                resources::Container::Settings => app::Container::Settings(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),

                resources::Container::Assets => app::Container::Assets(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Modified(app::ModifiedKind::Other))
            }

            resources::ResourceEvent::Asset { .. } => {
                app::EventKind::AssetFile(app::ResourceEvent::Modified(app::ModifiedKind::Other))
            }
        }
    }

    fn convert_resource_to_event_kind_moved(
        from_kind: resources::ResourceEvent,
        to_kind: resources::ResourceEvent,
        from: PathBuf,
        to: PathBuf,
        time: Instant,
    ) -> Vec<Event> {
        assert!(
            from.parent() != to.parent(),
            "moved paths should have different parent"
        );

        match (from_kind, to_kind) {
            (
                resources::ResourceEvent::Asset {
                    project: from_project,
                },
                resources::ResourceEvent::Asset {
                    project: to_project,
                },
            ) => {
                let kind = if from_project == to_project {
                    EventKind::AssetFile(app::ResourceEvent::Moved)
                } else {
                    EventKind::AssetFile(app::ResourceEvent::MovedProject)
                };

                vec![Event::with_time(kind, time).add_path(from).add_path(to)]
            }

            (
                resources::ResourceEvent::Analysis {
                    project: from_project,
                },
                resources::ResourceEvent::Analysis {
                    project: to_project,
                },
            ) => {
                let kind = if from_project == to_project {
                    EventKind::AnalysisFile(app::ResourceEvent::Moved)
                } else {
                    EventKind::AnalysisFile(app::ResourceEvent::MovedProject)
                };

                vec![Event::with_time(kind, time).add_path(from).add_path(to)]
            }

            (from_kind, to_kind) => {
                let from_kind = Self::convert_resource_to_event_kind_moved_from(from_kind);
                let to_kind = Self::convert_resource_to_event_kind_moved_to(to_kind);
                let from_event = Event::with_time(from_kind, time).add_path(from);
                let to_event = Event::with_time(to_kind, time).add_path(to);
                vec![from_event, to_event]
            }
        }
    }

    fn convert_resource_to_event_kind_renamed_from(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => EventKind::Config(
                    app::Config::ProjectManifest(app::StaticResourceEvent::Removed),
                ),
                resources::Config::UserManifest => {
                    EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Removed))
                }
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => {
                    app::Project::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Settings => {
                    app::Project::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Project::Analysis => {
                    app::Project::Analysis(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => {
                    app::Container::Properties(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Settings => {
                    app::Container::Settings(app::StaticResourceEvent::Removed).into()
                }

                resources::Container::Assets => {
                    app::Container::Assets(app::StaticResourceEvent::Removed).into()
                }
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Removed)
            }

            resources::ResourceEvent::Asset { .. } => {
                panic!("asset file renaming should not affect resource kind");
            }
        }
    }

    fn convert_resource_to_event_kind_renamed_to(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => EventKind::Config(
                    app::Config::ProjectManifest(app::StaticResourceEvent::Created),
                ),

                resources::Config::UserManifest => {
                    EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Created))
                }
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => {
                    app::Project::Properties(app::StaticResourceEvent::Created).into()
                }

                resources::Project::Settings => {
                    app::Project::Settings(app::StaticResourceEvent::Created).into()
                }

                resources::Project::Analysis => {
                    app::Project::Analysis(app::StaticResourceEvent::Created).into()
                }
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => {
                    app::Container::Properties(app::StaticResourceEvent::Created).into()
                }

                resources::Container::Settings => {
                    app::Container::Settings(app::StaticResourceEvent::Created).into()
                }

                resources::Container::Assets => {
                    app::Container::Assets(app::StaticResourceEvent::Created).into()
                }
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Created)
            }

            resources::ResourceEvent::Asset { .. } => {
                panic!("asset file renaming should not affect resource kind");
            }
        }
    }

    fn convert_resource_to_event_kind_renamed(
        from_kind: resources::ResourceEvent,
        to_kind: resources::ResourceEvent,
        from: PathBuf,
        to: PathBuf,
        time: Instant,
    ) -> StdResult<Vec<Event>, error::processing::Error> {
        match (from_kind, to_kind) {
            (
                resources::ResourceEvent::Asset {
                    project: from_project,
                },
                resources::ResourceEvent::Asset {
                    project: to_project,
                },
            ) => {
                if from_project != to_project {
                    return Err(error::processing::Error::InvalidState(
                        "asset rename should not change project".to_string(),
                    ));
                }

                Ok(vec![Event::with_time(
                    EventKind::AssetFile(app::ResourceEvent::Renamed),
                    time,
                )
                .add_path(from)
                .add_path(to)])
            }

            (
                resources::ResourceEvent::Analysis {
                    project: from_project,
                },
                resources::ResourceEvent::Analysis {
                    project: to_project,
                },
            ) => {
                if from_project != to_project {
                    return Err(error::processing::Error::InvalidState(
                        "analysis rename should not change project".to_string(),
                    ));
                }

                Ok(vec![Event::with_time(
                    EventKind::AnalysisFile(app::ResourceEvent::Renamed),
                    time,
                )
                .add_path(from)
                .add_path(to)])
            }

            (from_kind, to_kind) => {
                let from_kind = Self::convert_resource_to_event_kind_moved_from(from_kind);
                let to_kind = Self::convert_resource_to_event_kind_moved_to(to_kind);
                let from_event = Event::with_time(from_kind, time).add_path(from);
                let to_event = Event::with_time(to_kind, time).add_path(to);
                Ok(vec![from_event, to_event])
            }
        }
    }

    fn convert_resource_to_event_kind_data_modified(kind: resources::ResourceEvent) -> EventKind {
        match kind {
            resources::ResourceEvent::Config(kind) => match kind {
                resources::Config::ProjectManifest => app::Config::ProjectManifest(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),

                resources::Config::UserManifest => app::Config::UserManifest(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),
            },

            resources::ResourceEvent::Project { kind, .. } => match kind {
                resources::Project::Properties => app::Project::Properties(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),

                resources::Project::Settings => app::Project::Settings(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),

                resources::Project::Analysis => app::Project::Analysis(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),
            },

            resources::ResourceEvent::Container { kind, .. } => match kind {
                resources::Container::Properties => app::Container::Properties(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),

                resources::Container::Settings => app::Container::Settings(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),

                resources::Container::Assets => app::Container::Assets(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Data),
                )
                .into(),
            },

            resources::ResourceEvent::Analysis { .. } => {
                app::EventKind::AnalysisFile(app::ResourceEvent::Modified(app::ModifiedKind::Data))
            }

            resources::ResourceEvent::Asset { .. } => {
                app::EventKind::AssetFile(app::ResourceEvent::Modified(app::ModifiedKind::Data))
            }
        }
    }
}

impl FsWatcher {
    fn convert_dir_to_event_kind_created(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Created.into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Modified.into(),
                resources::ProjectDir::Config => {
                    app::Project::ConfigDir(app::StaticResourceEvent::Created).into()
                }
                resources::ProjectDir::Analysis => {
                    app::Project::AnalysisDir(app::ResourceEvent::Created).into()
                }
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Created).into()
                }
            },
            resources::DirKind::ContainerLike { .. } => unreachable!("should be handled elsewhere"),
            resources::DirKind::Container { .. } => EventKind::Graph(app::Graph::Created),
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Container(app::Container::ConfigDir(app::StaticResourceEvent::Created))
            }
            resources::DirKind::None { .. } => EventKind::Folder(app::ResourceEvent::Created),
        }
    }

    fn convert_dir_to_event_kind_removed(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Removed.into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Removed.into(),
                resources::ProjectDir::Config => {
                    app::Project::ConfigDir(app::StaticResourceEvent::Removed).into()
                }

                resources::ProjectDir::Analysis => {
                    app::Project::AnalysisDir(app::ResourceEvent::Removed).into()
                }

                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Removed).into()
                }
            },
            resources::DirKind::ContainerLike { .. } => {
                EventKind::Folder(app::ResourceEvent::Removed)
            }
            resources::DirKind::Container { .. } => EventKind::Graph(app::Graph::Removed),
            resources::DirKind::ContainerConfig { .. } => {
                app::Container::ConfigDir(app::StaticResourceEvent::Removed).into()
            }
            resources::DirKind::None { .. } => EventKind::Folder(app::ResourceEvent::Removed),
        }
    }

    /// Convert a directory event as if it moved from a resource to not.
    fn convert_dir_to_event_kind_moved_from(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Removed.into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Modified.into(),
                resources::ProjectDir::Config => {
                    app::Project::ConfigDir(app::StaticResourceEvent::Removed).into()
                }
                resources::ProjectDir::Analysis => {
                    app::Project::AnalysisDir(app::ResourceEvent::Moved).into()
                }
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Moved).into()
                }
            },
            resources::DirKind::ContainerLike { .. } => unreachable!("should be handled elsewhere"),
            resources::DirKind::Container { .. } => EventKind::Graph(app::Graph::Removed),
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Removed)
            }
            resources::DirKind::None { .. } => unreachable!("should be handled elsewhere"),
        }
    }

    /// Convert a directory event as if the directory went from not being a resource to being one.
    fn convert_dir_to_event_kind_moved_to(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Modified(app::ModifiedKind::Other).into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Modified.into(),
                resources::ProjectDir::Config => app::Project::ConfigDir(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Analysis => app::Project::AnalysisDir(
                    app::ResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Modified(app::ModifiedKind::Other))
                        .into()
                }
            },
            resources::DirKind::ContainerLike { .. } => unreachable!("should be handled elsewhere"),
            resources::DirKind::Container { .. } => {
                app::Graph::Modified(app::ModifiedKind::Other).into()
            }
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Modified(app::ModifiedKind::Other))
            }
            resources::DirKind::None { .. } => unreachable!("should be handled elsewhere"),
        }
    }

    /// Convert event as if it was moved inside a project to outside a project.
    fn convert_dir_to_event_kind_moved_from_project(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Removed.into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Removed.into(),
                resources::ProjectDir::Config => {
                    app::Project::ConfigDir(app::StaticResourceEvent::Removed).into()
                }
                resources::ProjectDir::Analysis => {
                    app::Project::AnalysisDir(app::ResourceEvent::Removed).into()
                }
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Removed).into()
                }
            },
            resources::DirKind::ContainerLike { .. } => unreachable!("should be handled elsewhere"),
            resources::DirKind::Container { .. } => EventKind::Graph(app::Graph::Removed),
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Removed)
            }
            resources::DirKind::None { .. } => EventKind::Folder(app::ResourceEvent::Removed),
        }
    }

    /// Convert event as if it was moved from outside a project into a project.
    fn convert_dir_to_event_kind_moved_to_project(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Modified(app::ModifiedKind::Other).into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Modified.into(),
                resources::ProjectDir::Config => app::Project::ConfigDir(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Analysis => app::Project::AnalysisDir(
                    app::ResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Modified(app::ModifiedKind::Other))
                        .into()
                }
            },
            resources::DirKind::ContainerLike { .. } => {
                app::EventKind::Folder(app::ResourceEvent::Created)
            }
            resources::DirKind::Container { .. } => app::Graph::Created.into(),
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Modified(app::ModifiedKind::Other))
            }
            resources::DirKind::None { .. } => EventKind::Folder(app::ResourceEvent::Created),
        }
    }

    fn convert_dir_to_event_kind_moved(
        from_kind: resources::DirKind,
        to_kind: resources::DirKind,
        from: PathBuf,
        to: PathBuf,
        time: Instant,
    ) -> Vec<Event> {
        assert!(!matches!(from_kind, resources::DirKind::None { .. }));
        assert!(!matches!(to_kind, resources::DirKind::None { .. }));
        assert!(
            from.parent() != to.parent(),
            "move event should have different parents"
        );

        match (from_kind, to_kind) {
            (
                resources::DirKind::Container {
                    project: from_project,
                },
                resources::DirKind::Container {
                    project: to_project,
                },
            ) => {
                if from_project == to_project {
                    vec![app::Event::with_time(app::Graph::Moved.into(), time)
                        .add_path(from)
                        .add_path(to)]
                } else {
                    vec![
                        Event::with_time(app::Graph::Removed.into(), time).add_path(from),
                        Event::with_time(app::Graph::Created.into(), time).add_path(to),
                    ]
                }
            }

            (
                resources::DirKind::ContainerLike {
                    project: from_project,
                },
                resources::DirKind::ContainerLike {
                    project: to_project,
                },
            ) => {
                if from_project == to_project {
                    vec![
                        Event::with_time(app::EventKind::Folder(app::ResourceEvent::Moved), time)
                            .add_path(from)
                            .add_path(to),
                    ]
                } else {
                    vec![
                        Event::with_time(EventKind::Folder(app::ResourceEvent::Removed), time)
                            .add_path(from),
                        Event::with_time(EventKind::Folder(app::ResourceEvent::Created), time)
                            .add_path(to),
                    ]
                }
            }

            (
                resources::DirKind::ContainerLike {
                    project: from_project,
                },
                resources::DirKind::Container {
                    project: to_project,
                },
            ) => {
                if from_project == to_project {
                    vec![Event::with_time(app::Graph::Moved.into(), time)
                        .add_path(from)
                        .add_path(to)]
                } else {
                    vec![
                        Event::with_time(app::Graph::Removed.into(), time).add_path(from),
                        Event::with_time(app::Graph::Created.into(), time).add_path(to),
                    ]
                }
            }

            (
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    project: from_project,
                },
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    project: to_project,
                },
            ) => {
                assert!(
                    from_project != to_project,
                    "project exists in two locations: {from:?} -> {to:?}"
                );

                tracing::warn!("UNUSUAL SITUATION: Project moved and replaced");
                vec![Event::with_time(app::Project::Moved.into(), time)
                    .add_path(from)
                    .add_path(to)]
            }

            (from_kind, to_kind) => {
                vec![
                    app::Event::with_time(
                        Self::convert_dir_to_event_kind_moved_from(&from_kind),
                        time,
                    )
                    .add_path(from),
                    app::Event::with_time(Self::convert_dir_to_event_kind_moved_to(&to_kind), time)
                        .add_path(to),
                ]
            }
        }
    }

    /// Convert directory event as if renaming it caused it to go from being a resource to not.
    fn convert_dir_to_event_kind_renamed_from(
        kind: &resources::DirKind,
    ) -> StdResult<EventKind, error::processing::Error> {
        let kind = match kind {
            resources::DirKind::AppConfig => app::Config::Removed.into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => unreachable!("should be handled elsewhere"),
                resources::ProjectDir::Config => {
                    app::Project::ConfigDir(app::StaticResourceEvent::Removed).into()
                }
                resources::ProjectDir::Analysis => {
                    app::Project::AnalysisDir(app::ResourceEvent::Removed).into()
                }
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Removed).into()
                }
            },
            resources::DirKind::ContainerLike { .. } => unreachable!("should be handled elsewhere"),
            resources::DirKind::Container { .. } => {
                return Err(error::processing::Error::InvalidState(
                    "renaming container should result in container".into(),
                ));
            }
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Removed)
            }
            resources::DirKind::None { .. } => unreachable!("should be handled elsewhere"),
        };

        Ok(kind)
    }

    /// Convert directory event as if renaming it caused it to go from not being a resource to
    /// being one.
    fn convert_dir_to_event_kind_renamed_to(kind: &resources::DirKind) -> EventKind {
        match kind {
            resources::DirKind::AppConfig => app::Config::Modified(app::ModifiedKind::Other).into(),
            resources::DirKind::Project { kind, .. } => match kind {
                resources::ProjectDir::Root => app::Project::Modified.into(),
                resources::ProjectDir::Config => app::Project::ConfigDir(
                    app::StaticResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Analysis => app::Project::AnalysisDir(
                    app::ResourceEvent::Modified(app::ModifiedKind::Other),
                )
                .into(),
                resources::ProjectDir::Data => {
                    app::Project::DataDir(app::ResourceEvent::Modified(app::ModifiedKind::Other))
                        .into()
                }
            },
            resources::DirKind::ContainerLike { .. } => {
                EventKind::Folder(app::ResourceEvent::Renamed)
            }
            resources::DirKind::Container { .. } => {
                panic!("renaming should not result in container");
            }
            resources::DirKind::ContainerConfig { .. } => {
                EventKind::Folder(app::ResourceEvent::Modified(app::ModifiedKind::Other))
            }
            resources::DirKind::None { .. } => unreachable!("should be handled elsewhere"),
        }
    }

    fn convert_dir_to_event_kind_renamed(
        from_kind: resources::DirKind,
        to_kind: resources::DirKind,
        from: PathBuf,
        to: PathBuf,
        time: Instant,
    ) -> StdResult<Vec<Event>, error::processing::Error> {
        assert!(
            from.parent() == to.parent(),
            "renamed paths should have same parent"
        );
        match (from_kind, to_kind) {
            (
                resources::DirKind::Container {
                    project: from_project,
                },
                resources::DirKind::Container {
                    project: to_project,
                },
            )
            | (
                resources::DirKind::ContainerLike {
                    project: from_project,
                },
                resources::DirKind::Container {
                    project: to_project,
                },
            ) => {
                assert!(
                    from_project == to_project,
                    "renaming container should not change project"
                );

                Ok(vec![app::Event::with_time(
                    app::Container::Renamed.into(),
                    time,
                )
                .add_path(from)
                .add_path(to)])
            }

            (
                resources::DirKind::ContainerLike {
                    project: from_project,
                },
                resources::DirKind::ContainerConfig {
                    project: to_project,
                },
            ) => {
                assert!(
                    from_project == to_project,
                    "renaming container should not change project"
                );

                Ok(vec![
                    app::Event::with_time(
                        app::EventKind::Folder(app::ResourceEvent::Removed).into(),
                        time,
                    )
                    .add_path(from),
                    app::Event::with_time(
                        app::Container::ConfigDir(app::StaticResourceEvent::Created).into(),
                        time,
                    )
                    .add_path(to),
                ])
            }

            (
                resources::DirKind::ContainerConfig {
                    project: from_project,
                },
                resources::DirKind::ContainerLike {
                    project: to_project,
                },
            ) => {
                assert!(
                    from_project == to_project,
                    "renaming container should not change project."
                );

                Ok(vec![
                    app::Event::with_time(
                        app::Container::ConfigDir(app::StaticResourceEvent::Removed).into(),
                        time,
                    )
                    .add_path(from),
                    app::Event::with_time(
                        app::EventKind::Folder(app::ResourceEvent::Created).into(),
                        time,
                    )
                    .add_path(to),
                ])
            }

            (
                resources::DirKind::ContainerConfig {
                    project: from_project,
                },
                resources::DirKind::Container {
                    project: to_project,
                },
            ) => {
                assert!(
                    from_project == to_project,
                    "renaming container should not change project."
                );

                Ok(vec![
                    app::Event::with_time(
                        app::Container::ConfigDir(app::StaticResourceEvent::Removed).into(),
                        time,
                    )
                    .add_path(from),
                    app::Event::with_time(app::Graph::Created.into(), time).add_path(to),
                ])
            }

            (
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    ..
                },
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    ..
                },
            ) => Ok(vec![
                app::Event::with_time(app::Project::Moved.into(), time),
                app::Event::with_time(app::Project::Modified.into(), time),
            ]),

            (
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    ..
                },
                resources::DirKind::Project { kind: to_kind, .. },
            ) => {
                return Err(error::processing::Error::InvalidState(format!(
                    "renaming project resulted in {to_kind:?}. {from:?} -> {to:?}"
                )));
            }
            (
                resources::DirKind::Project {
                    kind: from_kind, ..
                },
                resources::DirKind::Project {
                    kind: resources::ProjectDir::Root,
                    ..
                },
            ) => {
                return Err(error::processing::Error::InvalidState(format!(
                    "renaming {from_kind:?} resulted in project. {from:?} -> {to:?}"
                )));
            }

            (from_kind, to_kind) => {
                let from_kind = Self::convert_dir_to_event_kind_renamed_from(&from_kind)?;
                let to_kind = Self::convert_dir_to_event_kind_renamed_to(&to_kind);
                Ok(vec![
                    app::Event::with_time(from_kind, time).add_path(from),
                    app::Event::with_time(to_kind, time).add_path(to),
                ])
            }
        }
    }
}

/// If on Windows, convert to UNC if needed.
/// Otherwise, returns the given path.
fn normalize_path_root(path: impl Into<PathBuf>) -> PathBuf {
    if cfg!(target_os = "windows") {
        local_common::ensure_windows_unc(path)
    } else {
        path.into()
    }
}

pub mod config {
    use std::{io, path::PathBuf};
    use syre_local::{
        error::IoSerde,
        file_resource::SystemResource,
        system::collections::{ProjectManifest, UserManifest},
    };

    #[derive(Clone)]
    pub struct AppConfig {
        user_manifest: PathBuf,
        project_manifest: PathBuf,
    }

    impl AppConfig {
        pub fn new(
            user_manifest: impl Into<PathBuf>,
            project_manifest: impl Into<PathBuf>,
        ) -> Self {
            Self {
                user_manifest: user_manifest.into(),
                project_manifest: project_manifest.into(),
            }
        }

        /// Creates an app config using the paths obtained from the system.
        pub fn try_default() -> Result<Self, io::Error> {
            Ok(Self {
                user_manifest: UserManifest::path()?,
                project_manifest: ProjectManifest::path()?,
            })
        }

        pub fn user_manifest(&self) -> &PathBuf {
            &self.user_manifest
        }

        pub fn project_manifest(&self) -> &PathBuf {
            &self.project_manifest
        }

        pub fn load_user_manifest(&self) -> Result<UserManifest, IoSerde> {
            UserManifest::load_from(self.user_manifest.clone())
        }

        pub fn load_project_manifest(&self) -> Result<ProjectManifest, IoSerde> {
            ProjectManifest::load_from(self.project_manifest.clone())
        }
    }
}

mod resources {
    use super::config;
    pub use error::{Error, ErrorKind};
    use std::path::{Component, Path, PathBuf};
    use syre_core::{project::ScriptLang, types::ResourceId};
    use syre_local::{
        common,
        file_resource::SystemResource,
        project::{project, resources::Project as LocalProject},
        system::collections,
    };

    /// Files of resources represented
    #[derive(Debug, derive_more::From)]
    pub(crate) enum ResourceEvent {
        #[from]
        Config(Config),
        Project {
            project: ResourceId,
            kind: Project,
        },

        Container {
            project: ResourceId,
            kind: Container,
        },

        Asset {
            project: ResourceId,
        },

        Analysis {
            project: ResourceId,
        },
    }

    #[derive(Debug)]
    pub(crate) enum Config {
        ProjectManifest,
        UserManifest,
    }

    #[derive(Debug)]
    pub(crate) enum Project {
        Properties,
        Settings,
        Analysis,
    }

    #[derive(Debug)]
    pub(crate) enum Container {
        Properties,
        Settings,
        Assets,
    }

    #[derive(Debug)]
    pub(crate) enum DirKind {
        AppConfig,
        Project {
            project: ResourceId,
            kind: ProjectDir,
        },

        /// Folder is confirmed to be a container.
        Container {
            project: ResourceId,
        },

        /// Folder could potentially be a container.
        /// i.e. It is in the data folder of a project,
        /// and not within an app (.syre) folder.
        ContainerLike {
            project: ResourceId,
        },

        ContainerConfig {
            project: ResourceId,
        },

        None {
            project: ResourceId,
        },
    }

    #[derive(Debug)]
    pub(crate) enum ProjectDir {
        /// A project's base folder.
        Root,

        /// A project's config (.syre) folder.
        Config,

        /// A project's data root folder.
        Data,

        /// A project's analysis root folder.
        Analysis,
    }

    /// Represents potential of a path to be a config resource based on its location.
    #[derive(Debug)]
    pub(crate) enum ConfigLocationKind {
        /// Not a config resource.
        /// Path either does not contain any app folders (.syre) in it,
        /// or is nested in an app folder such that it could not be a resource.
        ///
        /// e.g. `/my/project/path/data`, `/my/project/path/.syre/nested/folder`
        Not,

        /// A potential config directory.
        /// Path was not nested in any app folders (.syre), and the final
        /// path component is an app folder.
        ///
        /// e.g. `/my/project/path/data/child/.syre`
        Dir,

        /// A potential config resource.
        /// Path is a child of a potential config directory.
        ///
        /// e.g. `/my/project/path/.syre/file.json`
        Child,

        /// Path is a descnedant of an app folder (.syre) but is not a child.
        ///
        /// e.g. `/my/project/path/.syre/nested/file.json`
        Nested,
    }

    /// Gets the kind of resource the path represents.
    ///
    /// # Errors
    /// + If the path does not belong to a valid app location (config or project).
    /// + If the path is in a project that is corrupt.
    pub(crate) fn resource_kind(
        path: &PathBuf,
        app_config: &config::AppConfig,
    ) -> Result<Option<ResourceEvent>, Error> {
        if path == app_config.project_manifest() {
            return Ok(Some(Config::ProjectManifest.into()));
        }

        if path == app_config.user_manifest() {
            return Ok(Some(Config::UserManifest.into()));
        }

        let project = project_by_resource_path(&path)?;
        if path.starts_with(common::app_dir_of(project.base_path())) {
            let kind =
                handle_file_project(path, project.base_path()).map(|kind| ResourceEvent::Project {
                    project: project.rid.clone(),
                    kind,
                });

            return Ok(kind);
        }

        if let Some(analysis_root) = project.analysis_root_path().as_ref() {
            if path.starts_with(analysis_root) {
                return Ok(is_analysis(path).then_some(ResourceEvent::Analysis {
                    project: project.rid.clone(),
                }));
            }
        }

        if path.starts_with(project.data_root_path()) {
            let kind = handle_file_data(path, &project);
            return Ok(kind);
        }

        Ok(None)
    }

    pub(crate) fn dir_kind(path: &PathBuf) -> Result<DirKind, Error> {
        if let Ok(config_dir) = syre_local::system::common::config_dir_path() {
            if *path == config_dir {
                return Ok(DirKind::AppConfig);
            }
        }

        let project = project_by_resource_path(&path)?;
        if *path == project.base_path() {
            return Ok(DirKind::Project {
                project: project.rid.clone(),
                kind: ProjectDir::Root,
            });
        }

        if *path == project.data_root_path() {
            return Ok(DirKind::Project {
                project: project.rid.clone(),
                kind: ProjectDir::Data,
            });
        }

        if let Some(analysis_dir) = project.analysis_root_path() {
            if *path == analysis_dir {
                return Ok(DirKind::Project {
                    project: project.rid.clone(),
                    kind: ProjectDir::Analysis,
                });
            }
        }

        if *path == common::app_dir_of(project.base_path()) {
            return Ok(DirKind::Project {
                project: project.rid.clone(),
                kind: ProjectDir::Config,
            });
        }

        if path.starts_with(project.data_root_path()) {
            let kind = handle_folder_data(path, &project);
            return Ok(kind);
        }

        Ok(DirKind::None {
            project: project.rid.clone(),
        })
    }

    /// Returns the potential type of config directory the path represents.
    ///
    /// # Errors
    /// + If a path segment was required to determine the resource kind,
    /// but it could not be obtained.
    pub(crate) fn config_resource_location(
        path: impl AsRef<Path>,
    ) -> Result<ConfigLocationKind, ()> {
        let app_dir = common::app_dir().as_os_str();
        let path = path.as_ref();
        let kind = match path
            .components()
            .filter(
                |component| matches!(component, Component::Normal(segment) if *segment == app_dir),
            )
            .count()
        {
            0 => ConfigLocationKind::Not,
            1 => {
                let Some(file_name) = path.file_name() else {
                    return Err(());
                };

                if file_name == app_dir {
                    ConfigLocationKind::Dir
                } else {
                    let Some(parent) = path.parent() else {
                        return Err(());
                    };

                    let Some(parent) = parent.file_name() else {
                        return Err(());
                    };

                    if parent == app_dir {
                        ConfigLocationKind::Child
                    } else {
                        ConfigLocationKind::Nested
                    }
                }
            }

            _ => ConfigLocationKind::Nested,
        };

        Ok(kind)
    }

    /// Get a `Project` by a path within it.
    ///
    /// # Errors
    /// + The path is not in a project.
    /// + The path is in a project that can not be loaded.
    fn project_by_resource_path(path: impl Into<PathBuf>) -> Result<LocalProject, Error> {
        let path = path.into();
        let Some(project_path) = project::project_root_path(&path) else {
            return Err(Error::new(path, ErrorKind::NotInProject));
        };

        LocalProject::load_from(&project_path)
            .map_err(|err| Error::new(project_path, ErrorKind::LoadProject(err).into()))
    }

    fn handle_file_project(path: &PathBuf, project: &Path) -> Option<Project> {
        if *path == common::project_file_of(project) {
            Some(Project::Properties)
        } else if *path == common::project_settings_file_of(project) {
            Some(Project::Settings)
        } else if *path == common::analyses_file_of(project) {
            Some(Project::Analysis)
        } else {
            None
        }
    }

    fn is_analysis(path: &PathBuf) -> bool {
        let Some(ext) = path.extension() else {
            return true;
        };

        let ext = ext.to_ascii_lowercase();
        let ext = ext.to_str().unwrap();
        if ScriptLang::supported_extensions().contains(&ext) {
            true
        } else {
            false
        }
    }

    fn handle_file_data(path: &PathBuf, project: &LocalProject) -> Option<ResourceEvent> {
        let Ok(rel_path) = path.strip_prefix(project.base_path()) else {
            return None;
        };

        let Ok(config_location) = config_resource_location(rel_path) else {
            return None;
        };

        match config_location {
            ConfigLocationKind::Not => Some(ResourceEvent::Asset {
                project: project.rid.clone(),
            }),

            ConfigLocationKind::Dir => {
                unreachable!("resource should not be a possible config folder")
            }

            ConfigLocationKind::Nested => None,

            ConfigLocationKind::Child => {
                if path.ends_with(common::container_file()) {
                    Some(ResourceEvent::Container {
                        project: project.rid.clone(),
                        kind: Container::Properties,
                    })
                } else if path.ends_with(common::container_settings_file()) {
                    Some(ResourceEvent::Container {
                        project: project.rid.clone(),
                        kind: Container::Settings,
                    })
                } else if path.ends_with(common::assets_file()) {
                    Some(ResourceEvent::Container {
                        project: project.rid.clone(),
                        kind: Container::Assets,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Obtain the type of resource a folder is that is within a project's data folder.
    fn handle_folder_data(path: &PathBuf, project: &LocalProject) -> DirKind {
        assert!(
            path.starts_with(project.data_root_path()),
            "data folders must begin with data root path"
        );

        let rel_path = path.strip_prefix(project.base_path()).unwrap();
        let Ok(config_location) = config_resource_location(rel_path) else {
            return DirKind::None {
                project: project.rid.clone(),
            };
        };

        match config_location {
            ConfigLocationKind::Not => {
                if common::container_file_of(path).exists() {
                    DirKind::Container {
                        project: project.rid.clone(),
                    }
                } else {
                    DirKind::ContainerLike {
                        project: project.rid.clone(),
                    }
                }
            }

            ConfigLocationKind::Dir => DirKind::ContainerConfig {
                project: project.rid.clone(),
            },

            ConfigLocationKind::Child => DirKind::None {
                project: project.rid.clone(),
            },

            ConfigLocationKind::Nested => DirKind::None {
                project: project.rid.clone(),
            },
        }
    }

    pub mod error {
        //! event errors meant to be reported with events that caused them.
        use std::path::PathBuf;
        use syre_local::error::IoSerde;

        #[derive(Debug)]
        pub struct Error {
            path: PathBuf,
            kind: ErrorKind,
        }

        impl Error {
            pub fn new(path: PathBuf, kind: ErrorKind) -> Self {
                Self { path, kind }
            }

            pub fn path(&self) -> &PathBuf {
                &self.path
            }

            pub fn kind(&self) -> &ErrorKind {
                &self.kind
            }
        }

        #[derive(Debug, PartialEq)]
        pub enum ErrorKind {
            /// The path was not in a valid project.
            NotInProject,

            /// The project failed to load.
            LoadProject(IoSerde),

            /// The project manifest failed to load.
            LoadProjectManifest(IoSerde),
        }
    }
}

#[cfg(test)]
#[path = "watcher_test.rs"]
mod watcher_test;
