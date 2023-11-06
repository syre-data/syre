use crate::server::Event;
use notify::{self, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const DEBOUNCE_TIMEOUT: Duration = Duration::from_millis(100);

pub enum FileSystemActorCommand {
    Watch(PathBuf),
    Unwatch(PathBuf),
}

pub struct FileSystemActor {
    command_rx: mpsc::Receiver<FileSystemActorCommand>,
    watcher: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl FileSystemActor {
    /// Create a new actor to watch the file system.
    /// Begins watching upon creation.
    pub fn new(
        event_tx: mpsc::Sender<Event>,
        command_rx: mpsc::Receiver<FileSystemActorCommand>,
    ) -> Self {
        let watcher = {
            let event_tx = event_tx.clone();
            new_debouncer(DEBOUNCE_TIMEOUT, None, move |event: DebounceEventResult| {
                event_tx.send(Event::FileSystem(event)).unwrap();
            })
            .unwrap()
        };

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
}
