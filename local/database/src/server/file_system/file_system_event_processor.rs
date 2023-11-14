//! Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](FileSystemEvent)s.
use super::event::file_system::{
    Any as AnyEvent, Event as FileSystemEvent, File as FileEvent, Folder as FolderEvent,
};
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::DebouncedEvent;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thot_local::common;

pub struct FileSystemEventProcessor;
impl FileSystemEventProcessor {
    /// Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](FileSystemEvent)s.
    /// Paths are canonicalized.
    ///
    /// # Notes
    /// + When canonicalizing paths:
    /// Assume that relative segments are resolved in file paths.
    /// On Windows, paths are canonicalized to UNC.
    /// However, `fs::canonicalize` can not be used on `from` paths because file no longer exists,
    /// so must canonicalize by hand.
    pub fn process(events: Vec<DebouncedEvent>) -> Vec<FileSystemEvent> {
        let events = Self::filter_events(events);
        let events = Self::filter_hidden(events);
        let events = Self::filter_created_subevents(events);
        let (mut converted, remaining) = Self::group_events(events);
        converted.append(&mut Self::convert_ungrouped_events(remaining));
        converted
    }

    fn filter_hidden(events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        events
            .into_iter()
            .filter(|event| match event.kind {
                EventKind::Create(_) => {
                    let Some(file_name) = event.paths[0].file_name() else {
                        return true;
                    };

                    let file_name = file_name.to_str().unwrap();
                    !file_name.starts_with(".")
                }

                _ => true,
            })
            .collect()
    }

    /// Filters out uninteresting events.
    fn filter_events(events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        events
            .into_iter()
            .filter(|event| match event.kind {
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Name(_)) => true,

                _ => false,
            })
            .collect()
    }

    /// Filters out subevents of a created folder.
    /// These include
    /// + Created files and folders contained in another created folder.
    fn filter_created_subevents(events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        let create_events = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| match event.kind {
                EventKind::Create(_) => Some((index, &event.paths[0])),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut root_paths = Vec::<(usize, &PathBuf)>::new();
        for (event_index, path) in create_events.iter() {
            let mut new_root = Some(root_paths.len());
            for (root_index, (_, root_path)) in root_paths.iter().enumerate() {
                if path.starts_with(root_path) {
                    new_root.take();
                    break;
                }

                if root_path.starts_with(path) {
                    let _ = new_root.insert(root_index);
                    break;
                }
            }

            match new_root {
                Some(root_index) if root_index == root_paths.len() => {
                    root_paths.push((event_index.clone(), path));
                }

                Some(index) => root_paths[index] = (event_index.clone(), path),

                None => {}
            }
        }

        let root_indices = root_paths
            .into_iter()
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        let create_indices = create_events
            .into_iter()
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        events
            .into_iter()
            .enumerate()
            .filter_map(|(index, event)| {
                if create_indices.contains(&index) && !root_indices.contains(&index) {
                    None
                } else {
                    Some(event)
                }
            })
            .collect()
    }

    /// Tries to convert all events into a single one.
    ///
    /// # Returns
    /// Tuple of (<converted events>, <unconverted events>).
    fn group_events(events: Vec<DebouncedEvent>) -> (Vec<FileSystemEvent>, Vec<DebouncedEvent>) {
        let (mut renamed, remaining) = Self::group_renamed(events);
        let (mut moved, remaining) = Self::group_moved(remaining);

        renamed.append(&mut moved);
        (renamed, remaining)
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
                    if to_name_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&from_name_event.paths[0])
                        } else {
                            from_name_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FileEvent::Renamed {
                                from,
                                to: fs::canonicalize(&to_name_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

                        grouped_rename_keys.push(parent.to_owned());
                    } else if to_name_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&from_name_event.paths[0])
                        } else {
                            from_name_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FolderEvent::Moved {
                                from,
                                to: fs::canonicalize(&to_name_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

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
                    if create_parent_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FileEvent::Renamed {
                                from,
                                to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

                        grouped_remove_create_keys.push(parent.to_owned());
                    } else if create_parent_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FolderEvent::Moved {
                                from,
                                to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

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
                    if create_parent_event.paths[0].is_file() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FileEvent::Moved {
                                from,
                                to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

                        grouped_names.push(file_name.to_owned());
                    } else if create_parent_event.paths[0].is_dir() {
                        let from = if cfg!(target_os = "windows") {
                            common::ensure_windows_unc(&remove_parent_event.paths[0])
                        } else {
                            remove_parent_event.paths[0].clone()
                        };

                        grouped_events.push(
                            FolderEvent::Moved {
                                from,
                                to: fs::canonicalize(&create_parent_event.paths[0]).unwrap(),
                            }
                            .into(),
                        );

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
        match event.kind {
            EventKind::Create(CreateKind::File) => {
                let path = fs::canonicalize(&event.paths[0]).unwrap();
                Some(FileEvent::Created(path).into())
            }

            EventKind::Create(CreateKind::Folder) => {
                let path = fs::canonicalize(&event.paths[0]).unwrap();
                Some(FolderEvent::Created(path).into())
            }

            EventKind::Create(CreateKind::Any) => {
                let Ok(path) = fs::canonicalize(&event.paths[0]) else {
                    return Some(AnyEvent::Created(event.paths[0].to_owned()).into());
                };
                if path.is_file() {
                    Some(FileEvent::Created(path).into())
                } else if path.is_dir() {
                    Some(FolderEvent::Created(path).into())
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
                    Some(FileEvent::Renamed { from, to }.into())
                } else if to.is_dir() {
                    Some(FolderEvent::Renamed { from, to }.into())
                } else {
                    None
                }
            }

            EventKind::Remove(RemoveKind::File) => {
                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(&event.paths[0])
                } else {
                    event.paths[0].clone()
                };

                Some(FileEvent::Removed(path).into())
            }

            EventKind::Remove(RemoveKind::Folder) => {
                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(&event.paths[0])
                } else {
                    event.paths[0].clone()
                };

                Some(FolderEvent::Removed(path).into())
            }

            EventKind::Remove(RemoveKind::Any) => {
                let path = if cfg!(target_os = "windows") {
                    common::ensure_windows_unc(&event.paths[0])
                } else {
                    event.paths[0].clone()
                };

                Some(AnyEvent::Removed(path).into())
            }

            _ => None,
        }
    }
}
