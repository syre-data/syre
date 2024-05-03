use crate::{resources, state};
use crossbeam::channel::{Receiver, Sender};
use options::Options;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{fs, io, ops::Range, path::PathBuf, thread};
use syre_fs_watcher as watcher;

pub struct Simulator {
    options: Options,
    state: State,
    rng: ChaCha8Rng,
    event_rx: Receiver<watcher::EventResult>,
    command_tx: Sender<watcher::Command>,
    watcher_thread: thread::JoinHandle<()>,
}

impl Simulator {
    pub fn new(options: Options) -> Self {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();
        fs::write(options.app_config().user_manifest(), b"[]").unwrap();
        fs::write(options.app_config().project_manifest(), b"[]").unwrap();

        let rng = ChaCha8Rng::seed_from_u64(options.seed());
        let watcher = watcher::FsWatcher::new(command_rx, event_tx, options.app_config().clone());
        let watcher_thread = std::thread::spawn(move || {
            watcher.run().unwrap();
        });

        Self {
            options,
            state: State::default(),
            rng,
            command_tx,
            event_rx,
            watcher_thread,
        }
    }
}

impl Simulator {
    pub fn run(&mut self) {
        while self.state.current_tick < self.options.max_ticks() {
            let actions = self.choose_actions();
            self.perform_actions(actions);
        }

        self.command_tx
            .send(watcher::Command::Watch(self.options.base_path().clone()))
            .unwrap();
    }

    fn choose_actions(&mut self) -> Vec<resources::app::Action> {
        let action_count = self.rng.gen_range(self.options.action_count_range());
        let mut actions = Vec::with_capacity(action_count as usize);
        while actions.len() < action_count as usize {
            let action = self.choose_action();
            if self.valid_action(&action) {
                continue;
            }

            match (action.resource(), action.action()) {
                _ => todo!(),
            }

            actions.push(action);
        }

        actions
    }

    fn choose_action(&self) -> resources::app::Action {
        rand::random()
    }

    /// Verifies that an action can be taken given the current state of the simulator.
    fn valid_action(&self, action: &resources::app::Action) -> bool {
        true
    }

    fn perform_actions(&self, actions: Vec<resources::app::Action>) -> Result<(), io::Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    current_tick: usize,
    app: state::App,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

pub mod options {
    use std::{ops::Range, path::PathBuf};
    use syre_fs_watcher::config::AppConfig;

    pub struct Options {
        seed: u64,
        base_path: PathBuf,
        max_ticks: usize,

        /// Range [min, max) of actions to be performed on each tick.
        action_count_range: Range<u8>,
        app_config: AppConfig,
    }

    impl Options {
        pub fn seed(&self) -> u64 {
            self.seed
        }

        pub fn base_path(&self) -> &PathBuf {
            &self.base_path
        }

        pub fn max_ticks(&self) -> usize {
            self.max_ticks
        }

        pub fn action_count_range(&self) -> Range<u8> {
            self.action_count_range.clone()
        }

        pub fn app_config(&self) -> &AppConfig {
            &self.app_config
        }
    }

    pub struct Builder {
        seed: u64,
        base_path: PathBuf,
        max_ticks: usize,
        action_count_range: Range<u8>,
        user_manifest: Option<PathBuf>,
        project_manifest: Option<PathBuf>,
    }

    impl Builder {
        pub fn new(base_path: PathBuf) -> Self {
            Self {
                seed: 0,
                base_path,
                max_ticks: 1_000,
                action_count_range: 0..10,
                user_manifest: None,
                project_manifest: None,
            }
        }

        /// Initialize with a random seed.
        pub fn with_random_seed(base_path: PathBuf) -> Self {
            let seed = rand::random();
            Self {
                seed,
                base_path,
                max_ticks: 1_000,
                action_count_range: 0..10,
                user_manifest: None,
                project_manifest: None,
            }
        }

        pub fn seed(&self) -> u64 {
            self.seed
        }

        pub fn set_seed(&mut self, seed: u64) {
            self.seed = seed;
        }

        pub fn max_ticks(&mut self) -> usize {
            self.max_ticks
        }

        pub fn set_max_ticks(&mut self, max_ticks: usize) {
            self.max_ticks = max_ticks;
        }

        pub fn set_action_count(&mut self, range: Range<u8>) {
            self.action_count_range = range;
        }

        pub fn set_user_manifest(&mut self, path: impl Into<PathBuf>) {
            let _ = self.user_manifest.insert(path.into());
        }

        pub fn set_project_manifest(&mut self, path: impl Into<PathBuf>) {
            let _ = self.project_manifest.insert(path.into());
        }

        pub fn build(self) -> Options {
            let app_config =
                AppConfig::new(self.user_manifest.unwrap(), self.project_manifest.unwrap());

            Options {
                seed: self.seed,
                base_path: self.base_path,
                max_ticks: self.max_ticks,
                action_count_range: self.action_count_range,
                app_config,
            }
        }
    }
}
