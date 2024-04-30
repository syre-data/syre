use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{fs, path::PathBuf};
use syre_fs_watcher as watcher;

#[test]
fn main() {
    let seed: u64 = 0;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    tracing::info!("running with seed {seed}");

    // TODO: NONDETERMINISTIC
    let config_dir = tempfile::TempDir::new().unwrap();
    let project_dir = tempfile::TempDir::new().unwrap();
    let (command_tx, command_rx) = crossbeam::channel::unbounded();
    let (event_tx, event_rx) = crossbeam::channel::unbounded();
    let watcher = watcher::FsWatcher::new(command_rx, event_tx);
    let watcher_jh = std::thread::spawn(move || {
        watcher.run().unwrap();
    });
    command_tx
        .send(watcher::Command::Watch(project_dir.path().to_path_buf()))
        .unwrap();

    simulate(project_dir.path().to_path_buf());
}

fn simulate(path: PathBuf) {}
