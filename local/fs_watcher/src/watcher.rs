//! File system watcher.
use crate::{
    actor::FileSystemActor,
    command::WatcherCommand,
    event::file_system::{
        Any as FsAnyEvent, Event as FsEvent, File as FsFileEvent, Folder as FsFolderEvent,
    },
    Command, Error, Event,
};
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, FileIdCache, FileIdMap};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
};
use syre_local::common;
use tokio::{
    fs, select,
    sync::{mpsc as tokio_mpsc, oneshot},
};
use tokio_stream::StreamExt;

/// Listens for events on the file system.
pub struct FsWatcher {
    /// Sends events to the client.
    event_tx: mpsc::Sender<Result<Vec<Event>, Vec<Error>>>,

    // Recieve commands from the client.
    command_rx: tokio_mpsc::UnboundedReceiver<Command>,

    /// Send commands to the file system watcher.
    command_tx: mpsc::Sender<WatcherCommand>,

    /// Recieve events from the file system watcher.
    event_rx: tokio_mpsc::UnboundedReceiver<DebounceEventResult>,

    // Must use own cache because the one being used by the notify watcher is automatically updated
    // on events recieved before we have access.
    /// Cache to hold file ids.
    file_ids: Arc<Mutex<FileIdMap>>,
}

impl FsWatcher {
    /// Creates a new file system watcher.
    /// The watcher immediately begins listening for file system events.
    /// Call the `run` method to listen for events.
    ///
    /// # Arguments
    /// 1. `command_rx`: Channel to recieve commands over.
    /// 2. `event_tx`: Channel to send events over.
    pub fn new(
        command_rx: tokio_mpsc::UnboundedReceiver<Command>,
        event_tx: mpsc::Sender<Result<Vec<Event>, Vec<Error>>>,
    ) -> Self {
        let (fs_tx, fs_rx) = tokio_mpsc::unbounded_channel();
        let (fs_command_tx, fs_command_rx) = mpsc::channel();
        let mut file_system_actor = FileSystemActor::new(fs_tx, fs_command_rx);
        thread::spawn(move || file_system_actor.run());

        Self {
            event_tx,
            command_rx,
            command_tx: fs_command_tx,
            event_rx: fs_rx,
            file_ids: Arc::new(Mutex::new(FileIdMap::new())),
        }
    }

    /// Begins responsiveness allowing events to be sent.
    pub async fn run(&mut self) {
        loop {
            select! {
                Some(command) = self.command_rx.recv() => {tokio::spawn(self.handle_command(command));},
                Some(events) = self.event_rx.recv() => {tokio::spawn(self.handle_events(events));},
                else => {
                    tracing::info!("channels closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&self, command: Command) {
        match command {
            Command::Watch(path) => {
                let mut file_ids = self.file_ids.lock().unwrap();
                file_ids.add_root(path.clone(), notify::RecursiveMode::Recursive);

                self.command_tx.send(WatcherCommand::Watch(path));
            }

            Command::Unwatch(path) => {
                let mut file_ids = self.file_ids.lock().unwrap();
                file_ids.remove_root(&path);
                self.command_tx.send(WatcherCommand::Unwatch(path));
            }

            Command::FinalPath { path, tx } => {
                self.final_path(path, tx).await;
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
    async fn final_path(
        &self,
        path: impl AsRef<Path>,
        tx: mpsc::Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
    ) {
        let path = path.as_ref();
        let id = {
            let file_ids = self.file_ids.lock().unwrap();
            let Some(id) = file_ids.cached_file_id(path).cloned() else {
                if let Err(err) = tx.send(Ok(None)) {
                    tracing::error!(?err);
                };
                return;
            };

            id
        };

        let path = match tokio::task::spawn_blocking(move || {
            file_path_from_id::path_from_id(&id).map(|path| Some(path))
        })
        .await
        {
            Ok(path) => path,
            Err(err) => {
                tracing::error!(?err);
                std::panic::resume_unwind(err.into_panic());
            }
        };

        if let Err(err) = tx.send(path) {
            tracing::error!(?err);
        }
    }

    async fn handle_events(&self, events: DebounceEventResult) {
        let Ok(events) = events else {
            tracing::error!("events error: {events:?}");
            todo!();
            // if let Err(err) = self.event_tx.send(events) {
            //     tracing::error!(?err);
            // }

            // return;
        };

        let (events, errors) = self.process_events(events).await;
        if !events.is_empty() {
            self.event_tx.send(Ok(events));
        }
        if !errors.is_empty() {
            tracing::error!("could not process events: {errors:?}");
            todo!();
            // self.event_tx.send(Err(errors));
        }
    }

    /// Process file system events into app events.
    ///
    /// # Returns
    /// Tuple of (events, errors).
    async fn process_events(
        &self,
        events: Vec<notify_debouncer_full::DebouncedEvent>,
    ) -> (Vec<Event>, Vec<String>) {
        let (events, fs_errors) = self.process_events_notify_to_fs(events).await;
        let (events, app_errors) = self.process_events_fs_to_app(events).await;
        let errors = fs_errors
            .into_iter()
            .map(|err| format!("{err:?}"))
            .chain(app_errors.into_iter().map(|err| format!("{err:?}")))
            .collect();

        (events, errors)
    }
}

impl FsWatcher {
    /// Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](FsEvent)s.
    ///
    /// # Notes
    /// + Events are assumed to have already been preprocessed with paths rectified.
    /// # Returns
    /// Tuple of (events, errors).
    async fn process_events_notify_to_fs(
        &self,
        events: Vec<DebouncedEvent>,
    ) -> (Vec<FsEvent>, Vec<DebouncedEvent>) {
        let events = Self::filter_events(events);
        let (grouped, remaining) = self.group_events(events).await;
        let (mut converted, remaining) = self.convert_ungrouped_events(remaining).await;
        converted.extend(grouped);
        (converted, remaining)
    }

    /// Filters out uninteresting events.
    fn filter_events(events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        events
            .into_iter()
            .filter(|event| match event.kind {
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Data(_))
                | EventKind::Modify(ModifyKind::Name(_))
                | EventKind::Modify(ModifyKind::Any) => true,

                _ => false,
            })
            .collect()
    }

    /// Tries to convert all events into a single one.
    ///
    /// # Returns
    /// Tuple of (<converted events>, <unconverted events>).
    async fn group_events(
        &self,
        events: Vec<DebouncedEvent>,
    ) -> (Vec<FsEvent>, Vec<DebouncedEvent>) {
        // let (mut renamed, remaining) = Self::group_renamed(events);
        // let (mut moved, remaining) = Self::group_moved(remaining);

        // renamed.append(&mut moved);
        // (renamed, remaining)
        let mut remaining = Vec::with_capacity(events.len());
        let mut grouped = HashMap::with_capacity(events.len());
        for event in events {
            match event.kind {
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) | EventKind::Remove(_) => {
                    let file_ids = self.file_ids.lock().unwrap();
                    let Some(id) = file_ids.cached_file_id(&event.paths[0]).cloned() else {
                        remaining.push(event);
                        continue;
                    };

                    let entry = grouped.entry(id).or_insert(vec![]);
                    entry.push(event);
                }

                EventKind::Modify(ModifyKind::Name(RenameMode::To)) | EventKind::Create(_) => {
                    let (tx, rx) = oneshot::channel();
                    match tokio::task::spawn_blocking({
                        let command_tx = self.command_tx.clone();
                        let path = event.paths[0].clone();
                        move || command_tx.send(WatcherCommand::FileId { path, tx })
                    })
                    .await
                    {
                        Ok(send_res) => {
                            if let Err(err) = send_res {
                                tracing::error!(?err);
                                remaining.push(event);
                                continue;
                            }
                        }
                        Err(err) => {
                            tracing::error!(?err);
                            remaining.push(event);
                            continue;
                        }
                    };

                    let id = match rx.await {
                        Ok(id) => id,
                        Err(err) => {
                            tracing::error!(?err);
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

                _ => {
                    remaining.push(event);
                }
            }
        }

        let mut converted = Vec::with_capacity(grouped.len() / 2);
        for mut events in grouped.into_values() {
            events.sort_unstable_by_key(|event| event.time);
            match &events[..] {
                [e] => remaining.push(e.clone()),

                [e1, e2] => {
                    if matches!(
                        [e1.kind, e2.kind],
                        [
                            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
                            EventKind::Modify(ModifyKind::Name(RenameMode::To))
                        ]
                    ) {
                        let path_from = e1.paths[0].clone();
                        let path_to = e2.paths[0].clone();
                        if path_from.parent() == path_to.parent() {
                            if path_to.is_file() {
                                converted.push(FsEvent::new(
                                    FsFileEvent::Renamed {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                ));
                            } else if path_to.is_dir() {
                                converted.push(FsEvent::new(
                                    FsFolderEvent::Renamed {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                ))
                            } else {
                                remaining.push(e1.clone());
                                remaining.push(e2.clone());
                            }
                        } else {
                            if path_to.is_file() {
                                converted.push(FsEvent::new(
                                    FsFileEvent::Moved {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                ));
                            } else if path_to.is_dir() {
                                converted.push(FsEvent::new(
                                    FsFolderEvent::Moved {
                                        from: path_from,
                                        to: path_to,
                                    },
                                    e2.time,
                                ))
                            } else {
                                remaining.push(e1.clone());
                                remaining.push(e2.clone());
                            }
                        }
                    } else if matches!(
                        [e1.kind, e2.kind],
                        [
                            EventKind::Remove(RemoveKind::File),
                            EventKind::Create(CreateKind::File)
                        ]
                    ) {
                        let path_from = e1.paths[0].clone();
                        let path_to = e2.paths[0].clone();
                        if path_from.parent() == path_to.parent() {
                            converted.push(FsEvent::new(
                                FsFileEvent::Renamed {
                                    from: path_from,
                                    to: path_to,
                                },
                                e2.time,
                            ));
                        } else {
                            converted.push(FsEvent::new(
                                FsFileEvent::Moved {
                                    from: path_from,
                                    to: path_to,
                                },
                                e2.time,
                            ));
                        }
                    } else if matches!(
                        [e1.kind, e2.kind],
                        [
                            EventKind::Remove(RemoveKind::Folder),
                            EventKind::Create(CreateKind::Folder)
                        ]
                    ) {
                        let path_from = e1.paths[0].clone();
                        let path_to = e2.paths[0].clone();
                        if path_from.parent() == path_to.parent() {
                            converted.push(FsEvent::new(
                                FsFolderEvent::Renamed {
                                    from: path_from,
                                    to: path_to,
                                },
                                e2.time,
                            ));
                        } else {
                            converted.push(FsEvent::new(
                                FsFolderEvent::Moved {
                                    from: path_from,
                                    to: path_to,
                                },
                                e2.time,
                            ));
                        }
                    }
                }

                _ => {
                    remaining.extend(events);
                }
            }
        }

        (converted, remaining)
    }

    async fn convert_ungrouped_events(
        &self,
        events: Vec<DebouncedEvent>,
    ) -> (Vec<FsEvent>, Vec<DebouncedEvent>) {
        enum ConversionResult {
            Converted(FsEvent),
            Unconverted(DebouncedEvent),
        }

        let conversion_res = tokio_stream::iter(events.clone())
            .then(|event| {
                tokio::spawn(async move {
                    match Self::convert_event(&event).await {
                        Some(converted) => ConversionResult::Converted(converted),
                        None => ConversionResult::Unconverted(event),
                    }
                })
            })
            .collect::<Vec<_>>()
            .await;

        let (converted, remaining) = conversion_res
            .into_iter()
            .enumerate()
            .map(|(index, event)| match event {
                Ok(event) => event,
                Err(err) => {
                    tracing::error!(?err);
                    ConversionResult::Unconverted(events[index].clone())
                }
            })
            .partition::<Vec<_>, _>(|event| match event {
                ConversionResult::Converted(_) => true,
                ConversionResult::Unconverted(_) => false,
            });

        let converted = converted
            .into_iter()
            .map(|event| match event {
                ConversionResult::Converted(event) => event,
                _ => unreachable!("events are sorted"),
            })
            .collect();

        let remaining = remaining
            .into_iter()
            .map(|event| match event {
                ConversionResult::Unconverted(event) => event,
                _ => unreachable!("events are sorted"),
            })
            .collect();

        (converted, remaining)
    }

    async fn convert_event(event: &DebouncedEvent) -> Option<FsEvent> {
        let time = event.time.clone();
        match event.kind {
            EventKind::Create(CreateKind::File) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).await.unwrap();
                Some(FsEvent::new(FsFileEvent::Created(path), time))
            }

            EventKind::Create(CreateKind::Folder) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).await.unwrap();
                Some(FsEvent::new(FsFolderEvent::Created(path), time))
            }

            EventKind::Create(CreateKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).await.unwrap();
                if path.is_file() {
                    Some(FsEvent::new(FsFileEvent::Created(path), time))
                } else if path.is_dir() {
                    Some(FsEvent::new(FsFolderEvent::Created(path), time))
                } else {
                    None
                }
            }

            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                let [from, to] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let to = fs::canonicalize(to).await.unwrap();
                let from = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(&from)
                } else {
                    from.clone()
                };

                if to.is_file() {
                    Some(FsEvent::new(FsFileEvent::Renamed { from, to }, time))
                } else if to.is_dir() {
                    Some(FsEvent::new(FsFolderEvent::Renamed { from, to }, time))
                } else {
                    None
                }
            }

            EventKind::Modify(ModifyKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = match fs::canonicalize(path).await {
                    Ok(path) => path,
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            panic!("change");
                        }

                        _ => {
                            tracing::debug!("failed to canonicalize path `{path:?}`: {err:?}");
                            return None;
                        }
                    },
                };

                if path.is_file() {
                    Some(FsEvent::new(FsFileEvent::Modified(path), time))
                } else if path.is_dir() {
                    Some(FsEvent::new(FsFolderEvent::Modified(path), time))
                } else {
                    None
                }
            }

            EventKind::Remove(RemoveKind::File) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(path)
                } else {
                    path.clone()
                };

                Some(FsEvent::new(FsFileEvent::Removed(path), time))
            }

            EventKind::Remove(RemoveKind::Folder) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(path)
                } else {
                    path.clone()
                };

                Some(FsEvent::new(FsFolderEvent::Removed(path), time))
            }

            EventKind::Remove(RemoveKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(path)
                } else {
                    path.clone()
                };

                Some(FsEvent::new(FsAnyEvent::Removed(path), time))
            }

            _ => None,
        }
    }
}

impl FsWatcher {
    /// Convert [file system events](FsEvent) to [app events](Event).
    ///
    /// # Returns
    /// Tuple of (converted, unconverted).
    async fn process_events_fs_to_app(&self, events: Vec<FsEvent>) -> (Vec<Event>, Vec<FsEvent>) {
        todo!();
    }
}

#[cfg(test)]
#[path = "watcher_test.rs"]
mod watcher_test;
