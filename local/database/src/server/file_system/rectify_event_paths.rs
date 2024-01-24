use super::super::Database;
use notify::event::{
    CreateKind, Event, EventAttributes, EventKind, ModifyKind, RemoveKind, RenameMode,
};
use notify_debouncer_full::DebouncedEvent;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, io};

impl Database {
    /// Canonicalizes and updates paths if needed.
    /// Updates the file systems watcher's paths if needed.
    /// Removes events whose paths can not be finalized.
    ///
    /// # Notes
    /// + When canonicalizing paths:
    ///     Assume that relative segments are resolved in file paths.
    ///     On Windows, paths are canonicalized to UNC.
    ///     However, `fs::canonicalize` can not be used on `from` paths because file no longer exists,
    ///     so must canonicalize by hand.
    pub fn rectify_event_paths(&mut self, events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        let earliest_instant = events.iter().map(|event| event.time).min().unwrap();
        let earliest_instant = earliest_instant - Duration::from_nanos(1);

        let rectified_events = events
            .into_iter()
            .map(|event| self.rectify_event_path(event))
            .collect::<Vec<_>>();

        let mut events = Vec::with_capacity(rectified_events.len());
        let mut remapped_paths = Vec::with_capacity(rectified_events.len());
        for event in rectified_events {
            let RectifiedEvent { event, path_map } = match event {
                Ok(event) => event,
                Err(err) => {
                    tracing::debug!(?err);
                    continue;
                }
            };

            if let Some(PathMap { from, to }) = path_map {
                if !remapped_paths.contains(&from) {
                    for event in process_path_change_to_event(from.clone(), to) {
                        events.push(DebouncedEvent {
                            event,
                            time: earliest_instant.clone(),
                        });
                    }

                    remapped_paths.push(from);
                }
            }

            events.push(event);
        }

        events
    }

    fn rectify_event_path(&self, event: DebouncedEvent) -> Result<RectifiedEvent, Error> {
        let DebouncedEvent {
            event: Event { kind, paths, attrs },
            time,
        } = event;

        match kind {
            EventKind::Any => {
                return self.rectify_single_path_should_exist(kind, paths, attrs, time)
            }

            EventKind::Access(_) => {
                return self.rectify_single_path_should_exist(kind, paths, attrs, time)
            }

            EventKind::Create(_) => {
                return self.rectify_single_path_should_exist(kind, paths, attrs, time)
            }

            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                return self.rectify_from_to_paths(kind, paths, attrs, time)
            }

            EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                return self.rectify_single_path_should_not_exist(kind, paths, attrs, time)
            }

            EventKind::Modify(_) => {
                return self.rectify_single_path_should_exist(kind, paths, attrs, time)
            }

            EventKind::Remove(_) => {
                return self.rectify_single_path_should_not_exist(kind, paths, attrs, time)
            }

            EventKind::Other => {
                return self.rectify_single_path_should_exist(kind, paths, attrs, time)
            }
        }
    }

    fn rectify_single_path_should_exist(
        &self,
        kind: EventKind,
        paths: Vec<PathBuf>,
        attrs: EventAttributes,
        time: Instant,
    ) -> Result<RectifiedEvent, Error> {
        let [path] = &paths[..] else {
            panic!("invalid paths");
        };

        let RectifiedPath {
            path,
            map: path_map,
        } = self.rectify_path(&path)?;

        Ok(RectifiedEvent {
            event: DebouncedEvent {
                event: Event {
                    kind,
                    paths: vec![path],
                    attrs: attrs,
                },
                time,
            },
            path_map,
        })
    }

    fn rectify_single_path_should_not_exist(
        &self,
        kind: EventKind,
        paths: Vec<PathBuf>,
        attrs: EventAttributes,
        time: Instant,
    ) -> Result<RectifiedEvent, Error> {
        let [path] = &paths[..] else {
            panic!("invalid paths");
        };

        let FinalPath { path, map } = self.rectify_final_path(&path)?;
        let path_map = if map.from == map.to { None } else { Some(map) };
        Ok(RectifiedEvent {
            event: DebouncedEvent {
                event: Event {
                    kind,
                    paths: vec![path],
                    attrs: attrs,
                },
                time,
            },
            path_map,
        })
    }

    fn rectify_from_to_paths(
        &self,
        kind: EventKind,
        paths: Vec<PathBuf>,
        attrs: EventAttributes,
        time: Instant,
    ) -> Result<RectifiedEvent, Error> {
        let [from, to] = &paths[..] else {
            panic!("invalid paths")
        };

        let RectifiedPath {
            path: to,
            map: path_map,
        } = self.rectify_path(&to)?;

        Ok(RectifiedEvent {
            event: DebouncedEvent {
                event: Event {
                    kind,
                    paths: vec![from.clone(), to],
                    attrs: attrs,
                },
                time,
            },
            path_map,
        })
    }

    fn rectify_path(&self, path: &PathBuf) -> Result<RectifiedPath, Error> {
        match fs::canonicalize(path) {
            Ok(path) => {
                return Ok(RectifiedPath { path, map: None });
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                let FinalPath { path, map } = self.rectify_final_path(path)?;
                let map = if map.from == map.to { None } else { Some(map) };
                return Ok(RectifiedPath { path, map });
            }

            Err(err) => return Err(Error::Canonicalize(err.kind())),
        }
    }

    fn rectify_final_path(&self, path: &PathBuf) -> Result<FinalPath, Error> {
        let mut path_map = None;
        let ancestors = path.ancestors().collect::<Vec<_>>();
        for ancestor in ancestors.into_iter().rev() {
            if let Some(to_ancestor) = self.get_final_path(ancestor)? {
                path_map = Some(PathMap {
                    from: ancestor.to_path_buf(),
                    to: to_ancestor,
                });
                break;
            }
        }

        let Some(path_map) = path_map else {
            return Err(Error::NoFinalPath);
        };

        let relative_path = path.strip_prefix(&path_map.from).unwrap();
        let final_path = path_map.to.join(relative_path);

        Ok(FinalPath {
            path: final_path,
            map: path_map,
        })
    }
}

/// Creates the corresponding event to the difference between the `from` and `to` paths.
///
/// # Notes
/// + Assumes that the paths represent the root of a watched file or folder.
fn process_path_change_to_event(from: PathBuf, to: PathBuf) -> Vec<Event> {
    if from == to {
        return vec![];
    } else {
        return vec![Event {
            kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            paths: vec![from.clone(), to.clone()],
            attrs: EventAttributes::new(),
        }];
    }
}

#[derive(Debug)]
struct PathMap {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug)]
struct FinalPath {
    path: PathBuf,
    map: PathMap,
}

#[derive(Debug)]
struct RectifiedPath {
    path: PathBuf,
    map: Option<PathMap>,
}

#[derive(Debug)]
struct RectifiedEvent {
    pub event: DebouncedEvent,

    /// Path map for the related project.
    pub path_map: Option<PathMap>,
}

#[derive(Debug)]
enum Error {
    /// Could not canonicalize the path.
    Canonicalize(io::ErrorKind),

    /// Could not get the final path.
    NoFinalPath,

    FinalPath(file_path_from_id::Error),
}

impl From<file_path_from_id::Error> for Error {
    fn from(value: file_path_from_id::Error) -> Self {
        Self::FinalPath(value)
    }
}
