//! Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](FileSystemEvent)s.
use crate::{
    command::WatcherCommand,
    event::file_system::{
        Any as AnyEvent, Event as FileSystemEvent, File as FileEvent, Folder as FolderEvent,
    },
    FsWatcher,
};
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::{DebouncedEvent, FileIdCache};
use std::{collections::HashMap, fs, io};
use syre_local::common;
use tokio::sync::oneshot;

impl FsWatcher {
    /// Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](FileSystemEvent)s.
    ///
    /// # Notes
    /// + Events are assumed to have already been preprocessed with paths rectified.
    /// # Returns
    /// Tuple of (events, errors).
    async fn process_events_notify_to_fs(
        &mut self,
        events: Vec<DebouncedEvent>,
    ) -> (Vec<Event>, Vec<Error>) {
        let events = Self::filter_events(events);
        let (mut converted, remaining) = Self::group_events(events);
        converted.append(&mut Self::convert_ungrouped_events(remaining));
        converted
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
    fn group_events(
        &mut self,
        events: Vec<DebouncedEvent>,
    ) -> (Vec<FileSystemEvent>, Vec<DebouncedEvent>) {
        // let (mut renamed, remaining) = Self::group_renamed(events);
        // let (mut moved, remaining) = Self::group_moved(remaining);

        // renamed.append(&mut moved);
        // (renamed, remaining)
        let remaining = Vec::with_capacity(events.len());
        let grouped = HashMap::with_capacity(events.len());
        for event in events {
            match event.kind {
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) | EventKind::Remove(_) => {
                    let Some(id) = self.file_ids.cached_file_id(&event.paths[0]) else {
                        remaining.push(event);
                        continue;
                    };

                    let entry = file_ids.entry(id).or_insert(vec![]);
                    entry.push(event);
                }

                EventKind::Modify(ModifyKind::Name(RenameMode::To)) | EventKind::Create(_) => {
                    let (tx, rx) = oneshot::channel();

                    let Some(id) = self
                        .command_tx
                        .send(WatcherCommand::FileId { path, tx })
                        .await
                    else {
                        remaining.push(event);
                        continue;
                    };

                    let entry = file_ids.entry(id).or_insert(vec![]);
                    entry.push(event);
                }

                _ => {
                    remaining.push(event);
                }
            }
        }

        (grouped, remaining)
    }

    /// Converts groups of events that represent a renaming.
    ///
    /// # Returns
    /// Tuple of (<converted events>, <unconverted events>).
    fn group_renamed(events: Vec<DebouncedEvent>) -> (Vec<FileSystemEvent>, Vec<DebouncedEvent>) {
        let mut other_events = Vec::with_capacity(events.len());
        let mut from_events = HashMap::with_capacity(events.len() / 2);
        let mut to_events = HashMap::with_capacity(events.len() / 2);
        let mut remove_events = HashMap::with_capacity(events.len() / 2);
        let mut create_events = HashMap::with_capacity(events.len() / 2);
        for event in events {
            match event.kind {
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    let parent = event.paths[0].parent().unwrap();
                    let event_map = from_events
                        .entry(parent.to_path_buf())
                        .or_insert(Vec::new());

                    event_map.push(event);
                }

                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    let parent = event.paths[0].parent().unwrap();
                    let event_map = to_events.entry(parent.to_path_buf()).or_insert(Vec::new());

                    event_map.push(event);
                }

                EventKind::Remove(_) => {
                    let parent = event.paths[0].parent().unwrap();
                    let event_map = remove_events
                        .entry(parent.to_path_buf())
                        .or_insert(Vec::new());

                    event_map.push(event);
                }

                EventKind::Create(_) => {
                    let parent = event.paths[0].parent().unwrap();
                    let event_map = create_events
                        .entry(parent.to_path_buf())
                        .or_insert(Vec::new());

                    event_map.push(event);
                }

                _ => other_events.push(event),
            }
        }

        // rename events
        let mut grouped_events = Vec::with_capacity(from_events.len() + remove_events.len());
        let mut grouped_rename_keys = Vec::with_capacity(from_events.len() + remove_events.len());
        for (parent, from_name_events) in from_events.iter() {
            let Some(to_name_events) = to_events.get(parent) else {
                continue;
            };

            match (&from_name_events[..], &to_name_events[..]) {
                ([from_name_event], [to_name_event]) => {
                    let time = vec![from_name_event.time, to_name_event.time]
                        .iter()
                        .min()
                        .unwrap()
                        .clone();

                    if to_name_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&from_name_event.paths[0])
                        } else {
                            from_name_event.paths[0].clone()
                        };

                        let kind = FileEvent::Renamed {
                            from,
                            to: fs::canonicalize(&to_name_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_rename_keys.push(parent.to_owned());
                    } else if to_name_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&from_name_event.paths[0])
                        } else {
                            from_name_event.paths[0].clone()
                        };

                        let kind = FolderEvent::Moved {
                            from,
                            to: fs::canonicalize(&to_name_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_rename_keys.push(parent.to_owned());
                    }
                }
                _ => {}
            }
        }

        // remove / create events
        let mut grouped_remove_create_keys = Vec::with_capacity(remove_events.len());
        for (parent, remove_parent_events) in remove_events.iter() {
            let Some(create_parent_events) = create_events.get(parent) else {
                continue;
            };

            match (&remove_parent_events[..], &create_parent_events[..]) {
                ([remove_parent_event], [create_parent_event]) => {
                    let time = vec![remove_parent_event.time, create_parent_event.time]
                        .iter()
                        .min()
                        .unwrap()
                        .clone();

                    if create_parent_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        let kind = FileEvent::Renamed {
                            from,
                            to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_remove_create_keys.push(parent.to_owned());
                    } else if create_parent_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        let kind = FolderEvent::Moved {
                            from,
                            to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_remove_create_keys.push(parent.to_owned());
                    }
                }
                _ => {}
            }
        }

        // sort remaining
        for name in grouped_rename_keys {
            from_events.remove(&name);
            to_events.remove(&name);
        }

        for name in grouped_remove_create_keys {
            remove_events.remove(&name);
            create_events.remove(&name);
        }

        for mut from_parent_events in from_events.into_values() {
            other_events.append(&mut from_parent_events);
        }

        for mut to_parent_events in to_events.into_values() {
            other_events.append(&mut to_parent_events);
        }

        for mut remove_parent_events in remove_events.into_values() {
            other_events.append(&mut remove_parent_events);
        }

        for mut create_parent_events in create_events.into_values() {
            other_events.append(&mut create_parent_events);
        }

        (grouped_events, other_events)
    }

    /// Converts groups of events that represent a move.
    ///
    /// # Returns
    /// Tuple of (<converted events>, <unconverted events>).
    fn group_moved(events: Vec<DebouncedEvent>) -> (Vec<FileSystemEvent>, Vec<DebouncedEvent>) {
        let mut other_events = Vec::with_capacity(events.len());
        let mut remove_events = HashMap::with_capacity(events.len() / 2);
        let mut create_events = HashMap::with_capacity(events.len() / 2);
        for event in events {
            match event.kind {
                EventKind::Remove(_) => {
                    let file_name = event.paths[0].file_name().unwrap();
                    let event_map = remove_events
                        .entry(file_name.to_owned())
                        .or_insert(Vec::new());

                    event_map.push(event);
                }

                EventKind::Create(_) => {
                    let file_name = event.paths[0].file_name().unwrap();
                    let event_map = create_events
                        .entry(file_name.to_owned())
                        .or_insert(Vec::new());

                    event_map.push(event);
                }

                _ => other_events.push(event),
            }
        }

        let mut grouped_events = Vec::with_capacity(remove_events.len());
        let mut grouped_names = Vec::with_capacity(remove_events.len());
        for (file_name, remove_parent_events) in remove_events.iter() {
            let Some(create_parent_events) = create_events.get(file_name) else {
                continue;
            };

            match (&remove_parent_events[..], &create_parent_events[..]) {
                ([remove_parent_event], [create_parent_event]) => {
                    let time = vec![remove_parent_event.time, create_parent_event.time]
                        .iter()
                        .min()
                        .unwrap()
                        .clone();

                    if create_parent_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        let kind = FileEvent::Moved {
                            from,
                            to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_names.push(file_name.to_owned());
                    } else if create_parent_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        let kind = FolderEvent::Moved {
                            from,
                            to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                        };

                        grouped_events.push(FileSystemEvent::new(kind, time));
                        grouped_names.push(file_name.to_owned());
                    }
                }
                _ => {}
            }
        }

        for name in grouped_names {
            remove_events.remove(&name);
            create_events.remove(&name);
        }

        for mut remove_name_events in remove_events.into_values() {
            other_events.append(&mut remove_name_events);
        }

        for mut create_name_events in create_events.into_values() {
            other_events.append(&mut create_name_events);
        }

        (grouped_events, other_events)
    }

    fn convert_ungrouped_events(events: Vec<DebouncedEvent>) -> Vec<FileSystemEvent> {
        events
            .iter()
            .filter_map(|event| Self::convert_event(event))
            .collect()
    }

    fn convert_event(event: &DebouncedEvent) -> Option<FileSystemEvent> {
        let time = event.time.clone();
        match event.kind {
            EventKind::Create(CreateKind::File) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                Some(FileSystemEvent::new(FileEvent::Created(path), time))
            }

            EventKind::Create(CreateKind::Folder) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                Some(FileSystemEvent::new(FolderEvent::Created(path), time))
            }

            EventKind::Create(CreateKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = fs::canonicalize(path).unwrap();
                if path.is_file() {
                    Some(FileSystemEvent::new(FileEvent::Created(path), time))
                } else if path.is_dir() {
                    Some(FileSystemEvent::new(FolderEvent::Created(path), time))
                } else {
                    None
                }
            }

            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                let [from, to] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let to = fs::canonicalize(to).unwrap();
                let from = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(&from)
                } else {
                    from.clone()
                };

                if to.is_file() {
                    Some(FileSystemEvent::new(FileEvent::Renamed { from, to }, time))
                } else if to.is_dir() {
                    Some(FileSystemEvent::new(
                        FolderEvent::Renamed { from, to },
                        time,
                    ))
                } else {
                    None
                }
            }

            EventKind::Modify(ModifyKind::Any) => {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = match fs::canonicalize(path) {
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
                    Some(FileSystemEvent::new(FileEvent::Modified(path), time))
                } else if path.is_dir() {
                    Some(FileSystemEvent::new(FolderEvent::Modified(path), time))
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

                Some(FileSystemEvent::new(FileEvent::Removed(path), time))
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

                Some(FileSystemEvent::new(FolderEvent::Removed(path), time))
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

                Some(FileSystemEvent::new(AnyEvent::Removed(path), time))
            }

            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "./file_system_event_processor_test.rs"]
mod file_system_event_processor_test;
