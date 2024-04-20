use file_id::FileId;
use std::{path::PathBuf, sync::mpsc};
use tokio::sync::oneshot;

pub enum Command {
    Watch(PathBuf),
    Unwatch(PathBuf),

    /// Gets the final path of the given path if it is being tracked.
    FinalPath {
        path: PathBuf,
        tx: mpsc::Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
    },
}

pub(crate) enum WatcherCommand {
    Watch(PathBuf),
    Unwatch(PathBuf),
    FileId {
        path: PathBuf,
        tx: oneshot::Sender<Option<FileId>>,
    },
}
