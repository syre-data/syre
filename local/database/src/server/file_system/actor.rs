use crate::server::Event;
use notify::{self, RecursiveMode, Watcher};
use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdCache, FileIdMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const DEBOUNCE_TIMEOUT: Duration = Duration::from_millis(100);

#[cfg(target_os = "macos")]
type FileSystemWatcher = notify::PollWatcher;
#[cfg(not(target_os = "macos"))]
type FileSystemWatcher = notify::RecommendedWatcher;

pub enum FileSystemActorCommand {
    Watch(PathBuf),
    Unwatch(PathBuf),

    /// Gets the final path of the given path if it is being tracked.
    FinalPath {
        path: PathBuf,
        tx: mpsc::Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
    },
}

pub struct FileSystemActor {
    command_rx: mpsc::Receiver<FileSystemActorCommand>,
    watcher: Debouncer<FileSystemWatcher, FileIdMap>,
}

impl FileSystemActor {
    /// Create a new actor to watch the file system.
    /// Begins watching upon creation.
    #[cfg(target_os = "macos")]
    pub fn new(
        event_tx: mpsc::Sender<Event>,
        command_rx: mpsc::Receiver<FileSystemActorCommand>,
    ) -> Self {
        let watcher: Debouncer<FileSystemWatcher, _> = {
            let event_tx = event_tx.clone();
            let config = notify::Config::default()
                .with_poll_interval(DEBOUNCE_TIMEOUT)
                .with_compare_contents(true);

            notify_debouncer_full::new_debouncer_opt(
                DEBOUNCE_TIMEOUT,
                None,
                move |event: DebounceEventResult| {
                    event_tx.send(Event::FileSystem(event)).unwrap();
                },
                notify_debouncer_full::FileIdMap::new(),
                config,
            )
            .unwrap()
        };

        Self {
            command_rx,
            watcher,
        }
    }

    /// Create a new actor to watch the file system.
    /// Begins watching upon creation.
    #[cfg(not(target_os = "macos"))]
    pub fn new(
        event_tx: mpsc::Sender<Event>,
        command_rx: mpsc::Receiver<FileSystemActorCommand>,
    ) -> Self {
        let watcher = notify_debouncer_full::new_debouncer(
            DEBOUNCE_TIMEOUT,
            None,
            move |event: DebounceEventResult| {
                event_tx.send(Event::FileSystem(event)).unwrap();
            },
        )
        .unwrap();

        Self {
            command_rx,
            watcher,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.command_rx.recv().unwrap() {
                FileSystemActorCommand::Watch(path) => self.watch(path),
                FileSystemActorCommand::Unwatch(path) => self.unwatch(path),
                FileSystemActorCommand::FinalPath { path, tx } => self.final_path(path, tx),
            }
        }
    }

    fn watch(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.watcher
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .unwrap();

        self.watcher
            .cache()
            .add_root(path, RecursiveMode::Recursive);
    }

    fn unwatch(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.watcher.watcher().unwatch(path).unwrap();
        self.watcher.cache().remove_root(path);
    }

    /// Gets the final path of a file.
    ///
    /// # Returns
    /// + `None` if the path is not in the watcher's cache.
    ///
    /// # Errors
    /// + If the final path could not be obtained.
    fn final_path(
        &mut self,
        path: impl AsRef<Path>,
        tx: mpsc::Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
    ) {
        let path = path.as_ref();
        let cache = self.watcher.cache();
        let Some(id) = cache.cached_file_id(path) else {
            match tx.send(Ok(None)) {
                Ok(_) => {}
                Err(err) => tracing::debug!(?err),
            };
            return;
        };

        let path_res = match file_path_from_id::path_from_id(id) {
            Ok(path) => Ok(Some(path)),
            Err(err) => Err(err),
        };

        match tx.send(path_res) {
            Ok(_) => {}
            Err(err) => tracing::debug!(?err),
        }
    }
}
