use crossbeam::channel::Sender;
use file_id::FileId;
use std::path::PathBuf;

pub enum Command {
    Watch(PathBuf),
    Unwatch(PathBuf),

    /// Gets the final path of the given path if it is being tracked.
    FinalPath {
        path: PathBuf,
        tx: Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
    },

    /// Shutdown the watcher.
    Shutdown,
}

pub(crate) enum WatcherCommand {
    Watch(PathBuf),
    Unwatch(PathBuf),
    FileId {
        path: PathBuf,
        tx: Sender<Option<FileId>>,
    },
}
