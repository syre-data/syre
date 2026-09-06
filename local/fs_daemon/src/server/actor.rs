//! File system daemon.
const DEBOUNCE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

#[cfg(target_os = "windows")]
pub use windows::FileSystemActor;

#[cfg(target_os = "macos")]
pub use macos::FileSystemActor;

#[cfg(target_os = "linux")]
pub use linux::FileSystemActor;

#[cfg(target_os = "windows")]
mod windows {
    use super::DEBOUNCE_TIMEOUT;
    use crate::command::WatcherCommand as Command;
    use crossbeam::channel::{Receiver, Sender};
    use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdMap};
    use std::path::{Path, PathBuf};

    type FileSystemWatcher = notify::RecommendedWatcher;
    pub struct FileSystemActor {
        command_rx: Receiver<Command>,
        watcher: Debouncer<FileSystemWatcher, FileIdMap>,
    }

    impl FileSystemActor {
        /// Create a new actor to watch the file system.
        /// Begins watching upon creation.
        pub fn new(event_tx: Sender<DebounceEventResult>, command_rx: Receiver<Command>) -> Self {
            let watcher =
                notify_debouncer_full::new_debouncer(DEBOUNCE_TIMEOUT, None, event_tx).unwrap();

            Self {
                command_rx,
                watcher,
            }
        }

        pub fn run(&mut self) {
            loop {
                let cmd = match self.command_rx.recv() {
                    Ok(cmd) => cmd,
                    Err(err) => break,
                };

                match cmd {
                    Command::Watch { path, tx } => self.watch(path, tx),
                    Command::Unwatch { path, tx } => self.unwatch(path, tx),
                }
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("command channel closed, shutting down");
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn watch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            if let Err(err) = self.watcher.watch(path, notify::RecursiveMode::Recursive) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not watch path `{path:?}`: {err:?}");
                if let Err(send_err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send watcher error: {send_err:?}");
                }

                return;
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("watching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn unwatch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            if let Err(err) = self.watcher.unwatch(path) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not unwatch path `{path:?}`: {err:?}");
                if let Err(send_err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send unwatch error: {send_err:?}");
                }

                return;
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("unwatching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }

        // TODO: Respond with an `Err` if can not get file id.
        /// Gets the final path of a file.
        ///
        /// # Response
        ///
        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn final_path(
            &mut self,
            path: impl AsRef<Path>,
            tx: Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
        ) {
            let path = path.as_ref();
            let id = match file_id::get_file_id(path) {
                Ok(id) => id,
                Err(err) => {
                    if let Err(send_err) = tx.send(Ok(None)) {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("could not send file id error: {err:?}");
                    }

                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not get file id of `{path:?}: {err:?}");
                    return;
                }
            };

            let path_res = file_path_from_id::path_from_id(&id).map(|path| Some(path));
            if let Err(err) = tx.send(path_res) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send file id: {err:?}");
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::DEBOUNCE_TIMEOUT;
    use crate::command::WatcherCommand as Command;
    use crossbeam::channel::{Receiver, Sender};
    use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdCache, FileIdMap};
    use std::{fs, io, path::Path};

    type FileSystemWatcher = notify::PollWatcher;

    pub struct FileSystemActor {
        command_rx: Receiver<Command>,
        watcher: Debouncer<FileSystemWatcher, FileIdMap>,
    }

    impl FileSystemActor {
        /// Create a new actor to watch the file system.
        /// Begins watching upon creation.
        ///
        /// # Notes
        /// + On macOS, PollWatcher is used for more informative events.
        pub fn new(event_tx: Sender<DebounceEventResult>, command_rx: Receiver<Command>) -> Self {
            let watcher: Debouncer<FileSystemWatcher, _> = {
                let event_tx = event_tx.clone();
                let config = notify::Config::default()
                    .with_poll_interval(DEBOUNCE_TIMEOUT)
                    .with_compare_contents(true);

                notify_debouncer_full::new_debouncer_opt(
                    DEBOUNCE_TIMEOUT,
                    None,
                    move |event: DebounceEventResult| {
                        event_tx.send(event).unwrap();
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

        pub fn run(&mut self) {
            loop {
                let Ok(cmd) = self.command_rx.recv() else {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("command channel closed, shutting down");
                    break;
                };

                match cmd {
                    Command::Watch { path, tx } => self.watch(path, tx),
                    Command::Unwatch { path, tx } => self.unwatch(path, tx),
                }
            }
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn watch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            // `PollWatcher` silently fails if path does not exist (or other failure).
            // Must manually check path exists.
            // See [https://github.com/notify-rs/notify/issues/998].
            if let Err(err) = fs::metadata(path) {
                let err_kind = match err.kind() {
                    io::ErrorKind::NotFound => notify::ErrorKind::PathNotFound,
                    io_err => notify::ErrorKind::Io(err),
                };
                let nerr = notify::Error::new(err_kind).add_path(path.to_path_buf());

                #[cfg(feature = "tracing")]
                tracing::warn!("could not watch path `{path:?}`: {nerr:?}");

                if let Err(send_err) = tx.send(Err(nerr)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send watcher error: {send_err:?}");
                }

                return;
            }

            if let Err(err) = self.watcher.watch(path, notify::RecursiveMode::Recursive) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not watch path `{path:?}`: {err:?}");
                if let Err(send_err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send watcher error: {send_err:?}");
                }

                return;
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("watching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn unwatch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            if let Err(err) = self.watcher.unwatch(path) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not unwatch path `{path:?}`: {err:?}");
                if let Err(send_err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send unwatch error: {send_err:?}");
                }

                return;
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("unwatching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::DEBOUNCE_TIMEOUT;
    use crate::command::WatcherCommand as Command;
    use crossbeam::channel::{Receiver, Sender};
    use notify::{self, Watcher, notify::RecursiveMode};
    use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdCache, FileIdMap};
    use std::path::{Path, PathBuf};

    type FileSystemWatcher = notify::RecommendedWatcher;

    pub struct FileSystemActor {
        command_rx: Receiver<Command>,
        watcher: Debouncer<FileSystemWatcher, FileIdMap>,
    }

    impl FileSystemActor {
        /// Create a new actor to watch the file system.
        /// Begins watching upon creation.
        pub fn new(event_tx: Sender<DebounceEventResult>, command_rx: Receiver<Command>) -> Self {
            let watcher =
                notify_debouncer_full::new_debouncer(DEBOUNCE_TIMEOUT, None, event_tx).unwrap();

            Self {
                command_rx,
                watcher,
            }
        }

        pub fn run(&mut self) {
            loop {
                let cmd = match self.command_rx.recv() {
                    Ok(cmd) => cmd,
                    Err(err) => break,
                };

                match cmd {
                    Command::Watch { path, tx } => self.watch(path, tx),
                    Command::Unwatch { path, tx } => self.unwatch(path, tx),
                    Command::FileId { path, tx } => {
                        if let Err(err) =
                            tx.send(self.watcher.cache().cached_file_id(&path).cloned())
                        {
                            #[cfg(feature = "tracing")]
                            tracing::warn!("could not send file id: {err:?}");
                        };
                    }
                }
            }

            #[cfg(feature = "tracing")]
            tracing::trace!("command channel closed, shutting down");
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn watch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            if let Err(err) = self
                .watcher
                .watcher()
                .watch(path, notify::RecursiveMode::Recursive)
            {
                if let Err(err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send watch error: {err:?}");
                }

                return;
            }

            self.watcher
                .cache()
                .add_root(path, notify::RecursiveMode::Recursive);

            #[cfg(feature = "tracing")]
            tracing::trace!("watching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn unwatch(&mut self, path: impl AsRef<Path>, tx: Sender<notify::Result<()>>) {
            let path = path.as_ref();
            if let Err(err) = self.watcher.watcher().unwatch(path) {
                if let Err(send_err) = tx.send(Err(err)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send unwatch error: {err:?}");
                }

                #[cfg(feature = "tracing")]
                tracing::warn!("could not unwatch `{path:?}`: {err:?}");
                return;
            }

            self.watcher.cache().remove_root(path);
            tracing::trace!("unwatching {path:?}");
            if let Err(err) = tx.send(Ok(())) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send `Ok`: {err:?}");
            }
        }

        /// Gets the final path of a file.
        ///
        /// # Returns
        /// + `None` if the path is not in the watcher's cache.
        ///
        /// # Errors
        /// + If the final path could not be obtained.
        #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
        fn final_path(
            &mut self,
            path: impl AsRef<Path>,
            tx: Sender<Result<Option<PathBuf>, file_path_from_id::Error>>,
        ) {
            let path = path.as_ref();
            let cache = self.watcher.cache();
            let Some(id) = cache.cached_file_id(path) else {
                if let Err(err) = tx.send(Ok(None)) {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not send response: {err:?}");
                }
                return;
            };

            let path_res = match file_path_from_id::path_from_id(id) {
                Ok(path) => Ok(Some(path)),
                Err(err) => Err(err),
            };

            if let Err(err) = tx.send(path_res) {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not send result: {err:?}");
            }
        }
    }
}
