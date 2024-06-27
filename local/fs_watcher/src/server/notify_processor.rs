use super::{super::event as fs_event, FsWatcher};
use crate::{command::WatcherCommand, error, Error};
use notify::event::{CreateKind, EventKind as NotifyEventKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::{DebouncedEvent, FileIdCache};
use std::{collections::HashMap, ffi::OsStr, fs, io, path::PathBuf, result::Result as StdResult};
use syre_local::common as local_common;

impl FsWatcher {
    /// Process [`notify_debouncer_full::DebouncedEvent`]s into [`file_system::Event`](fs_event::Event)s.
    ///
    /// # Notes
    /// + Events are assumed to have already been preprocessed with paths rectified.
    ///
    /// # Returns
    /// Tuple of (events, errors).
    pub fn process_events_notify_to_fs<'a>(
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
        let events = events
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
                        if file_name == ".DS_Store" {
                            return false;
                        }

                        if let Some(file_name) = file_name.to_str() {
                            if is_lock_file(file_name) {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .collect::<Vec<_>>();

        let events = Self::filter_nested_events(events);
        events
    }

    /// Filters out nested events.
    ///
    /// e.g. If a folder was created/removed with children, and both the parent folder and children
    /// resources creation/removal events are present, the events of the children are filtered out.
    fn filter_nested_events<'a>(events: Vec<&'a DebouncedEvent>) -> Vec<&'a DebouncedEvent> {
        /// A group of events.
        /// The map key is the common ancestor path of the group.
        /// The stand alone event is the common ancestor event.
        /// The events in the `Vec` are nested events.
        type EventGroupMap<'a> = HashMap<PathBuf, (&'a DebouncedEvent, Vec<&'a DebouncedEvent>)>;

        /// Group events based on path.
        fn group_events<'a>(events: &Vec<&'a DebouncedEvent>) -> EventGroupMap<'a> {
            let mut groups = EventGroupMap::new();
            for event in events.iter() {
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                if let Some((_, events)) = groups.iter_mut().find_map(|(key, events)| {
                    if path.starts_with(key) {
                        Some(events)
                    } else {
                        None
                    }
                }) {
                    events.push(event);
                } else if let Some(key) = groups.keys().find(|key| key.starts_with(path)).cloned() {
                    let (key_event, mut events) = groups.remove(&key).unwrap();
                    events.push(key_event);
                    groups.insert(path.clone(), (event, events));
                } else {
                    groups.insert(path.clone(), (event, vec![]));
                }
            }

            groups
        }

        let create_events = events
            .clone()
            .into_iter()
            .filter(|event| matches!(event.kind, NotifyEventKind::Create(_)))
            .collect::<Vec<_>>();

        let create_groups = group_events(&create_events);
        let create_child_events = create_groups
            .values()
            .flat_map(|(_, children)| children)
            .collect::<Vec<_>>();

        let events = events
            .into_iter()
            .filter(|&event| {
                !create_child_events
                    .iter()
                    .any(|&&child| std::ptr::eq(event, child))
            })
            .collect::<Vec<_>>();

        let remove_events = events
            .clone()
            .into_iter()
            .filter(|event| matches!(event.kind, NotifyEventKind::Remove(_)))
            .collect::<Vec<_>>();

        let remove_groups = group_events(&remove_events);
        let remove_child_events = remove_groups
            .values()
            .flat_map(|(_, children)| children)
            .collect::<Vec<_>>();

        let events = events
            .into_iter()
            .filter(|&event| {
                !remove_child_events
                    .iter()
                    .any(|&&child| std::ptr::eq(event, child))
            })
            .collect();

        events
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

            NotifyEventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                // NB: Must check if paths exists due to operation of `notify` crate.
                // See https://github.com/notify-rs/notify/issues/554.
                let [path] = &event.paths[..] else {
                    panic!("invalid paths");
                };

                let path = normalize_path_root(path);
                if path.exists() {
                    if path.is_dir() {
                        Some(fs_event::Event::new(fs_event::Folder::Created(path), time))
                    } else if path.is_file() {
                        Some(fs_event::Event::new(fs_event::File::Created(path), time))
                    } else {
                        return Err(error::Process::UnknownFileType);
                    }
                } else {
                    Some(fs_event::Event::new(fs_event::Any::Removed(path), time))
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

                /// Must check if paths exists due to operation of `notify` crate.
                /// See https://github.com/notify-rs/notify/issues/554.
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

/// If on Windows, convert to UNC if needed.
/// Otherwise, returns the given path.
fn normalize_path_root(path: impl Into<PathBuf>) -> PathBuf {
    if cfg!(target_os = "windows") {
        local_common::ensure_windows_unc(path)
    } else {
        path.into()
    }
}

/// Whether the file name matches that of a lock file.
/// i.e. .~<file_name>#
fn is_lock_file(file_name: impl AsRef<str>) -> bool {
    const START_PATTERN: &str = ".~";
    const END_PATTERN: &str = "#";

    let name: &str = file_name.as_ref();
    name.starts_with(START_PATTERN) && name.ends_with(END_PATTERN)
}

#[cfg(test)]
#[path = "notify_processor_test.rs"]
mod notify_processor_test;
