use crate::state::{self, actions};
use crossbeam::{
    channel::{Receiver, Sender},
    utils,
};
use options::Options;
use rand::{
    distributions::{self, DistString},
    prelude::*,
};
use rand_chacha::ChaCha8Rng;
use syre_core::project::excel_template::utils;
use std::{
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    thread,
};
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
            let action_count = self.rng.gen_range(self.options.action_count_range());
            let (actions, expected_state) = Self::choose_actions(
                action_count,
                self.state.app.clone(),
                &self.options,
                &mut self.rng,
            );

            tracing::debug!(?actions);
            self.perform_actions(actions);
        }

        self.command_tx
            .send(watcher::Command::Watch(self.options.base_path().clone()))
            .unwrap();
    }

    /// Choose actions to perform.
    ///
    /// # Arguments
    /// #. `num`: Number of actions to select.
    /// #. `state`: Current State to operate on. Used to select valid actions.
    ///
    /// # Returns
    /// Tuple of (actions, final state),
    /// where the final state is the state of the app after applying all actions.
    fn choose_actions<R>(
        num: u8,
        mut state: state::App,
        options: &Options,
        rng: &mut R,
    ) -> (Vec<state::Action>, state::App)
    where
        R: rand::Rng,
    {
        let num = num as usize;
        let mut actions = Vec::with_capacity(num);
        while actions.len() < num {
            let action = Self::choose_action(&state, options, rng);
            state.transition(&action).unwrap();
            actions.push(action);
        }

        (actions, state)
    }

    fn choose_action<R>(state: &state::App, options: &Options, rng: &mut R) -> state::Action
    where
        R: rand::Rng,
    {
        let mut valid_actions = Self::valid_actions(&state, options, rng);
        let index = rng.gen_range(0..valid_actions.len());
        valid_actions.swap_remove(index)
    }

    /// Returns a list of all valid actions given a state.
    fn valid_actions<R>(state: &state::App, options: &Options, rng: &mut R) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        let mut actions = Self::app_actions(state, options, rng)
            .into_iter()
            .map(|action| action.into())
            .collect::<Vec<state::Action>>();

        for project in state.projects.iter() {
            actions.extend(
                Self::project_actions(project, options, rng)
                    .into_iter()
                    .map(|action| {
                        state::Action::Project {
                            project: project.rid().clone(),
                            action,
                        }
                        .into()
                    }),
            );
        }

        actions
    }

    fn app_actions<R>(
        state: &state::App,
        options: &Options,
        rng: &mut R,
    ) -> Vec<actions::AppResource>
    where
        R: rand::Rng,
    {
        let rename_path = PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16));
        let move_path = utils::random_move_path(
            options.app_config().user_manifest(),
            options.base_path(),
            rng,
        );

        let mut actions = vec![];
        match state.user_manifest {
            state::Resource::NotPresent => actions.push(actions::AppResource::UserManifest(
                actions::Manifest::Create,
            )),

            state::Resource::Invalid => {
                actions.extend(vec![
                    actions::AppResource::UserManifest(actions::Manifest::Remove),
                    actions::AppResource::UserManifest(actions::Manifest::Rename(rename_path)),
                    actions::AppResource::UserManifest(actions::Manifest::Move(move_path)),
                    actions::AppResource::UserManifest(actions::Manifest::Repair),
                ]);
            }

            state::Resource::Valid => {
                actions.extend(vec![
                    actions::AppResource::UserManifest(actions::Manifest::Remove),
                    actions::AppResource::UserManifest(actions::Manifest::Rename(rename_path)),
                    actions::AppResource::UserManifest(actions::Manifest::Move(move_path)),
                    actions::AppResource::UserManifest(actions::Manifest::Corrupt),
                    actions::AppResource::UserManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Add,
                    )),
                    actions::AppResource::UserManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Remove,
                    )),
                    actions::AppResource::UserManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Alter,
                    )),
                ]);
            }
        }

        let rename_path = PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16));
        let move_path = utils::random_move_path(
            options.app_config().user_manifest(),
            options.base_path(),
            rng,
        );

        match state.project_manifest {
            state::Resource::NotPresent => actions.push(actions::AppResource::ProjectManifest(
                actions::Manifest::Create,
            )),

            state::Resource::Invalid => {
                actions.extend(vec![
                    actions::AppResource::ProjectManifest(actions::Manifest::Remove),
                    actions::AppResource::ProjectManifest(actions::Manifest::Rename(rename_path)),
                    actions::AppResource::ProjectManifest(actions::Manifest::Move(move_path)),
                    actions::AppResource::ProjectManifest(actions::Manifest::Repair),
                ]);
            }

            state::Resource::Valid => {
                actions.extend(vec![
                    actions::AppResource::ProjectManifest(actions::Manifest::Remove),
                    actions::AppResource::ProjectManifest(actions::Manifest::Rename(rename_path)),
                    actions::AppResource::ProjectManifest(actions::Manifest::Move(move_path)),
                    actions::AppResource::ProjectManifest(actions::Manifest::Corrupt),
                    actions::AppResource::ProjectManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Add,
                    )),
                    actions::AppResource::ProjectManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Remove,
                    )),
                    actions::AppResource::ProjectManifest(actions::Manifest::Modify(
                        actions::ModifyManifest::Alter,
                    )),
                ]);
            }
        }

        actions
    }

    fn project_actions<R>(
        state: &state::Project,
        options: &Options,
        rng: &mut R,
    ) -> Vec<actions::ProjectResource>
    where
        R: rand::Rng,
    {
        let mut actions = Self::project_resource_actions(state)
            .into_iter()
            .map(|action| action.into())
            .collect::<Vec<actions::ProjectResource>>();

        match &state.graph {
            state::Reference::NotPresent => {
                let path = state.path.join(utils::random_file_name(rng));
                actions.push(actions::Project::DataDir(actions::Dir::Create(path)).into())
            }
            state::Reference::Present(graph) => {
                let rename_path = utils::random_file_name(rng);
                let copy_path = utils::random_file_name(rng);
                let move_path = utils::random_move_path(&graph.root_path, &state.path, rng);
                actions.extend(vec![
                    actions::Project::DataDir(actions::Dir::Remove).into(),
                    actions::Project::DataDir(actions::Dir::Rename(rename_path)).into(),
                    actions::Project::DataDir(actions::Dir::Move(move_path)).into(),
                    actions::Project::DataDir(actions::Dir::Copy(copy_path)).into(),
                ]);

                actions.extend(Self::project_graph_actions(&graph));
            }
        }

        actions
    }

    fn project_resource_actions<R>(state: &state::Project, rng: &mut R) -> Vec<actions::Project>
    where
        R: rand::Rng,
    {
        let mut actions = vec![];
        match &state.config {
            state::Reference::NotPresent => {
                actions.push(actions::Project::ConfigDir(actions::StaticDir::Create));
            }

            state::Reference::Present(config) => {
                let rename_path = utils::random_file_name(rng);
                let copy_path = utils::random_file_name(rng);
                let move_path = utils::random_move_path(
                    syre_local::common::app_dir_of(&state.path),
                    &state.path,
                    rng,
                );

                actions.extend(vec![
                    actions::Project::ConfigDir(actions::StaticDir::Remove),
                    actions::Project::ConfigDir(actions::StaticDir::Rename(rename_path)),
                    actions::Project::ConfigDir(actions::StaticDir::Move(move_path)),
                    actions::Project::ConfigDir(actions::StaticDir::Copy(copy_path)),
                ]);

                actions.extend(Self::project_config_actions(&config));
            }
        }

        match &state.analyses {
            None => {}
            Some(state::Reference::NotPresent) => {
                let filename = utils::random_file_name(rng);
                actions.push(actions::Project::AnalysisDir(actions::Dir::Create(
                    filename,
                )))
            }

            Some(state::Reference::Present(_)) => {
                let rename_path = utils::random_file_name(rng);
                let copy_path = utils::random_file_name(rng);
                let move_path = utils::random_move_path(
                    syre_local::common::app_dir_of(&state.path),
                    &state.path,
                    rng,
                );

                actions.extend(vec![
                    actions::Project::AnalysisDir(actions::Dir::Remove),
                    actions::Project::AnalysisDir(actions::Dir::Rename(rename_path)),
                    actions::Project::AnalysisDir(actions::Dir::Move(move_path)),
                    actions::Project::AnalysisDir(actions::Dir::Copy(copy_path)),
                ]);
            }
        }

        actions
    }

    fn project_graph_actions(state: &state::Graph) -> Vec<actions::ProjectResource> {
        let mut actions = vec![];
        for node in state.nodes() {
            actions.extend(Self::container_actions(node.borrow().deref()));
        }

        actions
    }

    fn container_actions(state: &state::Container, graph_root) -> Vec<actions::ProjectResource> {
        let mut actions = vec![];
        match &state.config {
            state::Reference::NotPresent => actions.push(actions::ProjectResource::Container {
                container: state.rid().clone(),
                action: actions::Container::ConfigDir(actions::StaticDir::Create).into(),
            }),

            state::Reference::Present(config) => actions.extend(
                Self::container_config_actions(&config)
                    .into_iter()
                    .map(|action| actions::ProjectResource::Container {
                        container: state.rid().clone(),
                        action,
                    }),
            ),
        }

        for asset in state.assets.iter() {
            actions.extend(Self::asset_actions(asset).into_iter().map(|action| {
                actions::ProjectResource::AssetFile {
                    container: state.rid().clone(),
                    asset: asset.rid().clone(),
                    action,
                }
            }));
        }

        actions
    }

    fn asset_actions<R>(state: &state::Asset, rng: &mut R) -> Vec<actions::AssetFile>
    where
        R: rand::Rng,
    {
        match state.file {
            state::Reference::NotPresent => {
                let path = utils::random_file_name(rng);
                vec![actions::AssetFile::Create(path)]
            }

            state::Reference::Present(_) => {
                let rename_path = utils::random_file_name(rng);
                let move_path = utils::random_move_path(base_path, root_path, rng)
                vec![
                    actions::AssetFile::Remove,
                    actions::AssetFile::Rename(rename_path),
                    actions::AssetFile::Move,
                    actions::AssetFile::Copy,
                    actions::AssetFile::Modify,
                ]
            }
        }
    }

    fn container_config_actions(state: &state::ContainerConfig) -> Vec<actions::Container> {
        let mut actions = vec![];
        match &state.properties {
            state::Resource::NotPresent => {
                actions.push(actions::Container::Properties(actions::StaticFile::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Container::Properties(actions::StaticFile::Remove),
                actions::Container::Properties(actions::StaticFile::Rename),
                actions::Container::Properties(actions::StaticFile::Move),
                actions::Container::Properties(actions::StaticFile::Copy),
                actions::Container::Properties(actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Container::Properties(actions::StaticFile::Remove),
                actions::Container::Properties(actions::StaticFile::Rename),
                actions::Container::Properties(actions::StaticFile::Move),
                actions::Container::Properties(actions::StaticFile::Copy),
                actions::Container::Properties(actions::StaticFile::Modify),
                actions::Container::Properties(actions::StaticFile::Corrupt),
            ]),
        }

        match &state.settings {
            state::Resource::NotPresent => {
                actions.push(actions::Container::Settings(actions::StaticFile::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Container::Settings(actions::StaticFile::Remove),
                actions::Container::Settings(actions::StaticFile::Rename),
                actions::Container::Settings(actions::StaticFile::Move),
                actions::Container::Settings(actions::StaticFile::Copy),
                actions::Container::Settings(actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Container::Settings(actions::StaticFile::Remove),
                actions::Container::Settings(actions::StaticFile::Rename),
                actions::Container::Settings(actions::StaticFile::Move),
                actions::Container::Settings(actions::StaticFile::Copy),
                actions::Container::Settings(actions::StaticFile::Modify),
                actions::Container::Settings(actions::StaticFile::Corrupt),
            ]),
        }

        match &state.assets {
            state::Resource::NotPresent => {
                actions.push(actions::Container::Assets(actions::Manifest::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Container::Assets(actions::Manifest::Remove),
                actions::Container::Assets(actions::Manifest::Rename),
                actions::Container::Assets(actions::Manifest::Move),
                actions::Container::Assets(actions::Manifest::Copy),
                actions::Container::Assets(actions::Manifest::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Container::Assets(actions::Manifest::Remove),
                actions::Container::Assets(actions::Manifest::Rename),
                actions::Container::Assets(actions::Manifest::Move),
                actions::Container::Assets(actions::Manifest::Copy),
                actions::Container::Assets(actions::Manifest::Corrupt),
                actions::Container::Assets(actions::Manifest::Modify(actions::ModifyManifest::Add)),
                actions::Container::Assets(actions::Manifest::Modify(
                    actions::ModifyManifest::Remove,
                )),
                actions::Container::Assets(actions::Manifest::Modify(
                    actions::ModifyManifest::Alter,
                )),
            ]),
        }

        actions
    }

    fn project_config_actions(state: &state::ProjectConfig) -> Vec<actions::Project> {
        let mut actions = vec![];
        match state.properties {
            state::Resource::NotPresent => {
                actions.push(actions::Project::Properties(actions::StaticFile::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Project::Properties(actions::StaticFile::Remove),
                actions::Project::Properties(actions::StaticFile::Rename),
                actions::Project::Properties(actions::StaticFile::Move),
                actions::Project::Properties(actions::StaticFile::Copy),
                actions::Project::Properties(actions::StaticFile::Modify),
                actions::Project::Properties(actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Project::Properties(actions::StaticFile::Remove),
                actions::Project::Properties(actions::StaticFile::Rename),
                actions::Project::Properties(actions::StaticFile::Move),
                actions::Project::Properties(actions::StaticFile::Copy),
                actions::Project::Properties(actions::StaticFile::Modify),
                actions::Project::Properties(actions::StaticFile::Corrupt),
            ]),
        }

        match state.settings {
            state::Resource::NotPresent => {
                actions.push(actions::Project::Settings(actions::StaticFile::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Project::Settings(actions::StaticFile::Remove),
                actions::Project::Settings(actions::StaticFile::Rename),
                actions::Project::Settings(actions::StaticFile::Move),
                actions::Project::Settings(actions::StaticFile::Copy),
                actions::Project::Settings(actions::StaticFile::Modify),
                actions::Project::Settings(actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Project::Settings(actions::StaticFile::Remove),
                actions::Project::Settings(actions::StaticFile::Rename),
                actions::Project::Settings(actions::StaticFile::Move),
                actions::Project::Settings(actions::StaticFile::Copy),
                actions::Project::Settings(actions::StaticFile::Modify),
                actions::Project::Settings(actions::StaticFile::Corrupt),
            ]),
        }

        match state.analyses {
            state::Resource::NotPresent => {
                actions.push(actions::Project::Analyses(actions::Manifest::Create))
            }

            state::Resource::Invalid => actions.extend(vec![
                actions::Project::Analyses(actions::Manifest::Remove),
                actions::Project::Analyses(actions::Manifest::Rename),
                actions::Project::Analyses(actions::Manifest::Move),
                actions::Project::Analyses(actions::Manifest::Copy),
                actions::Project::Analyses(actions::Manifest::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                actions::Project::Analyses(actions::Manifest::Remove),
                actions::Project::Analyses(actions::Manifest::Rename),
                actions::Project::Analyses(actions::Manifest::Move),
                actions::Project::Analyses(actions::Manifest::Copy),
                actions::Project::Analyses(actions::Manifest::Corrupt),
                actions::Project::Analyses(actions::Manifest::Modify(actions::ModifyManifest::Add)),
                actions::Project::Analyses(actions::Manifest::Modify(
                    actions::ModifyManifest::Remove,
                )),
                actions::Project::Analyses(actions::Manifest::Modify(
                    actions::ModifyManifest::Alter,
                )),
            ]),
        }

        actions
    }

    fn perform_actions(&self, actions: Vec<state::Action>) -> Result<(), io::Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    current_tick: usize,
    pub app: state::App,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

mod utils {
    use rand::distributions::{self, DistString, Distribution};
    use std::path::{Path, PathBuf};

    pub fn random_file_name<R>(rng: &mut R) -> PathBuf
    where
        R: rand::Rng,
    {
        PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
    }

    pub fn random_move_path<R>(
        base_path: impl AsRef<Path>,
        root_path: impl AsRef<Path>,
        rng: &mut R,
    ) -> PathBuf
    where
        R: rand::Rng,
    {
        let (paths, distances): (Vec<_>, Vec<_>) = path_distances(base_path, root_path)
            .into_iter()
            .filter(|(_, distance)| *distance > 0)
            .unzip();

        let distance_bound = distances.iter().max().unwrap() + 1;
        let weights = distances
            .into_iter()
            .map(|dist| distance_bound - dist)
            .collect::<Vec<_>>();

        let path_dist = distributions::WeightedIndex::new(&weights).unwrap();
        paths[path_dist.sample(rng)].clone()

        // let kind: actions::MoveKind = rng.sample(distributions::Standard);
        // match kind {
        //     actions::MoveKind::Ancestor => {
        //         if let Some(parent) = base_path.parent() {
        //             let mut parent = parent.to_path_buf();
        //             parent.set_file_name(base_path.file_name().unwrap());
        //             parent
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     actions::MoveKind::Descendant => {
        //         if let Some(parent) = base_path.parent() {
        //             let filename = base_path.file_name().unwrap();
        //             parent
        //                 .join(distributions::Alphanumeric.sample_string(rng, 16))
        //                 .join(filename)
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     actions::MoveKind::Sibling => {
        //         if let Some(parent) = base_path.parent() {
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     actions::MoveKind::OutOfResource => {
        //         PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //     }
        // }
    }

    /// Returns the distance between all paths in the root directory and the base path.
    fn path_distances(
        base_path: impl AsRef<Path>,
        root_path: impl AsRef<Path>,
    ) -> Vec<(PathBuf, usize)> {
        let base_path = base_path.as_ref();
        let root_path = root_path.as_ref();
        walkdir::WalkDir::new(root_path)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let dist = path_distance(entry.path(), base_path);
                Some((entry.path().to_path_buf(), dist))
            })
            .collect()
    }

    /// Calculate the nuber of steps to go from one path to another.
    ///
    /// # Notes
    /// + Assumes the paths a relative to the same root.
    fn path_distance(a: impl AsRef<Path>, b: impl AsRef<Path>) -> usize {
        let mut a = a.as_ref().components();
        let mut b = b.as_ref().components();

        while a.next() == b.next() {}
        a.count() + b.count()
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

#[cfg(test)]
#[path = "simulator_test.rs"]
mod simulator_test;
