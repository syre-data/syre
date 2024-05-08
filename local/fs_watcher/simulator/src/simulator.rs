use crate::state;
use crossbeam::channel::{Receiver, Sender};
use options::Options;
use rand::{
    distributions::{self, DistString},
    prelude::*,
};
use rand_chacha::ChaCha8Rng;
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
            let (actions, app_state_expected, fs_state_expected) = Self::choose_actions(
                action_count,
                self.state.app.clone(),
                self.state.fs.clone(),
                &mut self.rng,
            );

            tracing::debug!(?actions);
            self.perform_actions(actions);
        }

        self.command_tx
            .send(watcher::Command::Watch(self.options.base_path().clone()))
            .unwrap();
    }
}

// impl Simulator {
//     fn choose_actions<R>(
//         num: u8,
//         mut fs_state: state::fs::State,
//         rng: &mut R,
//     ) -> (Vec<state::fs::Action>, state::fs::State)
//     where
//         R: rand::Rng,
//     {
//         let num = num as usize;
//         let mut actions = Vec::with_capacity(num);
//         while actions.len() < num {
//             let action = Self::choose_action(&fs_state, rng);
//             fs_state.transition(action.clone()).unwrap();
//             actions.push(action);
//         }

//         (actions, fs_state)
//     }

//     fn choose_action<R>(state: &state::fs::State, rng: &mut R) -> state::fs::Action
//     where
//         R: rand::Rng,
//     {
//         let mut valid_actions = Self::valid_actions(&state, rng);
//         let index = rng.gen_range(0..valid_actions.len());
//         valid_actions.swap_remove(index)
//     }

//     /// Returns a list of all valid actions given a state.
//     fn valid_actions<R>(state: &state::fs::State, rng: &mut R) -> Vec<state::fs::Action>
//     where
//         R: rand::Rng,
//     {
//         let mut actions = vec![];
//         for folder in state.folders() {
//             actions.extend(
//                 vec![
//                     state::fs::FolderAction::Remove(folder.clone()),
//                     state::fs::FolderAction::Rename {
//                         folder: folder.clone(),
//                         name: utils::random_file_name(rng),
//                     },
//                     // TODO
//                     state::fs::FolderAction::Move {
//                         folder: folder.clone(),
//                         parent: None,
//                     },
//                 ]
//                 .into_iter()
//                 .map(|action| action.into()),
//             );
//         }

//         actions
//     }
// }

// impl Simulator {
//     fn perform_actions(&mut self, actions: Vec<state::fs::Action>) {
//         for action in actions {
//             match action {
//                 Folder(action) => self.perform_actions_folder(action),
//                 File(action) => self.perform_actions_file(action),
//             }
//         }
//     }

//     fn perform_actions_folder(&mut self, action: state::fs::Action) {
//         match action {
//             state::fs::FolderAction::Insert { folder, parent } => {
//                 todo!();
//             }

//             state::fs::FolderAction::Remove(folder) => {
//                 todo!();
//             }

//             state::fs::FolderAction::Rename { folder, name } => {
//                 todo!();
//             }

//             state::fs::FolderAction::Move { folder, parent } => {
//                 todo!();
//             }
//         }
//     }

//     fn perform_actions_file(&mut self, action: state::fs::FileAction) {
//         todo!();
//     }
// }

impl Simulator {
    /// Choose actions to perform.
    ///
    /// # Arguments
    /// #. `num`: Number of actions to select.
    /// #. `state`: Current State to operate on. Used to select valid actions.
    ///
    /// # Returns
    /// Tuple of (actions, final state),
    /// where the final state should be the state of the app after applying all actions.
    fn choose_actions<R>(
        num: u8,
        mut app_state: state::app::State,
        mut fs_state: state::fs::State,
        rng: &mut R,
    ) -> (Vec<state::Action>, state::app::State, state::fs::State)
    where
        R: rand::Rng,
    {
        let num = num as usize;
        let mut actions = Vec::with_capacity(num);
        while actions.len() < num {
            let action = Self::choose_action(&app_state, &fs_state, rng);
            app_state.transition(&action).unwrap();
            fs_state.transition(&action).unwrap();
            actions.push(action);
        }

        (actions, app_state, fs_state)
    }

    fn choose_action<R>(
        app_state: &state::app::State,
        fs_state: &state::fs::State,
        rng: &mut R,
    ) -> state::Action
    where
        R: rand::Rng,
    {
        let mut valid_actions = Self::valid_actions(&state, options, rng);
        let index = rng.gen_range(0..valid_actions.len());
        valid_actions.swap_remove(index)
    }

    /// Returns a list of all valid actions given a state.
    fn valid_actions<R>(
        app_state: &state::app::State,
        fs_state: &state::fs::State,
        rng: &mut R,
    ) -> Vec<state::Action>
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
        state: &state::app::State,
        options: &Options,
        rng: &mut R,
    ) -> Vec<state::actions::AppResource>
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
            state::Resource::NotPresent => actions.push(state::actions::AppResource::UserManifest(
                state::actions::Manifest::Create,
            )),

            state::Resource::Invalid => {
                actions.extend(vec![
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Remove),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Rename(
                        rename_path,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Move(
                        move_path,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Repair),
                ]);
            }

            state::Resource::Valid => {
                actions.extend(vec![
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Remove),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Rename(
                        rename_path,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Move(
                        move_path,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Corrupt),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Add,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Remove,
                    )),
                    state::actions::AppResource::UserManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Alter,
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
            state::Resource::NotPresent => actions.push(
                state::actions::AppResource::ProjectManifest(state::actions::Manifest::Create),
            ),

            state::Resource::Invalid => {
                actions.extend(vec![
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Remove),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Rename(
                        rename_path,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Move(
                        move_path,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Repair),
                ]);
            }

            state::Resource::Valid => {
                actions.extend(vec![
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Remove),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Rename(
                        rename_path,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Move(
                        move_path,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Corrupt),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Add,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Remove,
                    )),
                    state::actions::AppResource::ProjectManifest(state::actions::Manifest::Modify(
                        state::actions::ModifyManifest::Alter,
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
    ) -> Vec<state::actions::ProjectResource>
    where
        R: rand::Rng,
    {
        let mut actions = Self::project_resource_actions(state)
            .into_iter()
            .map(|action| action.into())
            .collect::<Vec<state::actions::ProjectResource>>();

        match &state.graph {
            state::Reference::NotPresent => {
                let path = state.path.join(utils::random_file_name(rng));
                actions.push(
                    state::actions::Project::DataDir(state::actions::Dir::Create(path)).into(),
                )
            }
            state::Reference::Present(graph) => {
                let rename_path = utils::random_file_name(rng);
                let copy_path = utils::random_file_name(rng);
                let move_path = utils::random_move_path(&graph.root_path, &state.path, rng);
                actions.extend(vec![
                    state::actions::Project::DataDir(state::actions::Dir::Remove).into(),
                    state::actions::Project::DataDir(state::actions::Dir::Rename(rename_path))
                        .into(),
                    state::actions::Project::DataDir(state::actions::Dir::Move(move_path)).into(),
                    state::actions::Project::DataDir(state::actions::Dir::Copy(copy_path)).into(),
                ]);

                actions.extend(Self::project_graph_actions(&graph));
            }
        }

        actions
    }

    fn project_resource_actions<R>(
        state: &state::Project,
        rng: &mut R,
    ) -> Vec<state::actions::Project>
    where
        R: rand::Rng,
    {
        let mut actions = vec![];
        match &state.config {
            state::Reference::NotPresent => {
                actions.push(state::actions::Project::ConfigDir(
                    state::actions::StaticDir::Create,
                ));
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
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Remove),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Rename(
                        rename_path,
                    )),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Move(move_path)),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Copy(copy_path)),
                ]);

                actions.extend(Self::project_config_actions(&config));
            }
        }

        match &state.analyses {
            None => {}
            Some(state::Reference::NotPresent) => {
                let filename = utils::random_file_name(rng);
                actions.push(state::actions::Project::AnalysisDir(
                    state::actions::Dir::Create(filename),
                ))
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
                    state::actions::Project::AnalysisDir(state::actions::Dir::Remove),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Rename(rename_path)),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Move(move_path)),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Copy(copy_path)),
                ]);
            }
        }

        actions
    }

    fn project_graph_actions(state: &state::Graph) -> Vec<state::actions::ProjectResource> {
        let mut actions = vec![];
        for node in state.nodes() {
            actions.extend(Self::container_actions(node.borrow().deref()));
        }

        actions
    }

    fn container_actions(state: &state::Container) -> Vec<state::actions::ProjectResource> {
        let mut actions = vec![];
        match &state.config {
            state::Reference::NotPresent => {
                actions.push(state::actions::ProjectResource::Container {
                    container: state.rid().clone(),
                    action: state::actions::Container::ConfigDir(state::actions::StaticDir::Create)
                        .into(),
                })
            }

            state::Reference::Present(config) => actions.extend(
                Self::container_config_actions(&config)
                    .into_iter()
                    .map(|action| state::actions::ProjectResource::Container {
                        container: state.rid().clone(),
                        action,
                    }),
            ),
        }

        for asset in state.assets.iter() {
            actions.extend(Self::asset_actions(asset).into_iter().map(|action| {
                state::actions::ProjectResource::AssetFile {
                    container: state.rid().clone(),
                    asset: asset.rid().clone(),
                    action,
                }
            }));
        }

        actions
    }

    fn asset_actions<R>(state: &state::Asset, rng: &mut R) -> Vec<state::actions::AssetFile>
    where
        R: rand::Rng,
    {
        match state.file {
            state::Reference::NotPresent => {
                let path = utils::random_file_name(rng);
                vec![state::actions::AssetFile::Create(path)]
            }

            state::Reference::Present(_) => {
                let rename_path = utils::random_file_name(rng);
                // let move_path = utils::random_move_path(base_path, root_path, rng);
                vec![
                    state::actions::AssetFile::Remove,
                    state::actions::AssetFile::Rename(rename_path),
                    state::actions::AssetFile::Move,
                    state::actions::AssetFile::Copy,
                    state::actions::AssetFile::Modify,
                ]
            }
        }
    }

    fn container_config_actions(state: &state::ContainerConfig) -> Vec<state::actions::Container> {
        let mut actions = vec![];
        match &state.properties {
            state::Resource::NotPresent => actions.push(state::actions::Container::Properties(
                state::actions::StaticFile::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Container::Properties(state::actions::StaticFile::Remove),
                state::actions::Container::Properties(state::actions::StaticFile::Rename),
                state::actions::Container::Properties(state::actions::StaticFile::Move),
                state::actions::Container::Properties(state::actions::StaticFile::Copy),
                state::actions::Container::Properties(state::actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Container::Properties(state::actions::StaticFile::Remove),
                state::actions::Container::Properties(state::actions::StaticFile::Rename),
                state::actions::Container::Properties(state::actions::StaticFile::Move),
                state::actions::Container::Properties(state::actions::StaticFile::Copy),
                state::actions::Container::Properties(state::actions::StaticFile::Modify),
                state::actions::Container::Properties(state::actions::StaticFile::Corrupt),
            ]),
        }

        match &state.settings {
            state::Resource::NotPresent => actions.push(state::actions::Container::Settings(
                state::actions::StaticFile::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Container::Settings(state::actions::StaticFile::Remove),
                state::actions::Container::Settings(state::actions::StaticFile::Rename),
                state::actions::Container::Settings(state::actions::StaticFile::Move),
                state::actions::Container::Settings(state::actions::StaticFile::Copy),
                state::actions::Container::Settings(state::actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Container::Settings(state::actions::StaticFile::Remove),
                state::actions::Container::Settings(state::actions::StaticFile::Rename),
                state::actions::Container::Settings(state::actions::StaticFile::Move),
                state::actions::Container::Settings(state::actions::StaticFile::Copy),
                state::actions::Container::Settings(state::actions::StaticFile::Modify),
                state::actions::Container::Settings(state::actions::StaticFile::Corrupt),
            ]),
        }

        match &state.assets {
            state::Resource::NotPresent => actions.push(state::actions::Container::Assets(
                state::actions::Manifest::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Container::Assets(state::actions::Manifest::Remove),
                state::actions::Container::Assets(state::actions::Manifest::Rename),
                state::actions::Container::Assets(state::actions::Manifest::Move),
                state::actions::Container::Assets(state::actions::Manifest::Copy),
                state::actions::Container::Assets(state::actions::Manifest::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Container::Assets(state::actions::Manifest::Remove),
                state::actions::Container::Assets(state::actions::Manifest::Rename),
                state::actions::Container::Assets(state::actions::Manifest::Move),
                state::actions::Container::Assets(state::actions::Manifest::Copy),
                state::actions::Container::Assets(state::actions::Manifest::Corrupt),
                state::actions::Container::Assets(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Add,
                )),
                state::actions::Container::Assets(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Remove,
                )),
                state::actions::Container::Assets(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Alter,
                )),
            ]),
        }

        actions
    }

    fn project_config_actions(state: &state::ProjectConfig) -> Vec<state::actions::Project> {
        let mut actions = vec![];
        match state.properties {
            state::Resource::NotPresent => actions.push(state::actions::Project::Properties(
                state::actions::StaticFile::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Project::Properties(state::actions::StaticFile::Remove),
                state::actions::Project::Properties(state::actions::StaticFile::Rename),
                state::actions::Project::Properties(state::actions::StaticFile::Move),
                state::actions::Project::Properties(state::actions::StaticFile::Copy),
                state::actions::Project::Properties(state::actions::StaticFile::Modify),
                state::actions::Project::Properties(state::actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Project::Properties(state::actions::StaticFile::Remove),
                state::actions::Project::Properties(state::actions::StaticFile::Rename),
                state::actions::Project::Properties(state::actions::StaticFile::Move),
                state::actions::Project::Properties(state::actions::StaticFile::Copy),
                state::actions::Project::Properties(state::actions::StaticFile::Modify),
                state::actions::Project::Properties(state::actions::StaticFile::Corrupt),
            ]),
        }

        match state.settings {
            state::Resource::NotPresent => actions.push(state::actions::Project::Settings(
                state::actions::StaticFile::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Project::Settings(state::actions::StaticFile::Remove),
                state::actions::Project::Settings(state::actions::StaticFile::Rename),
                state::actions::Project::Settings(state::actions::StaticFile::Move),
                state::actions::Project::Settings(state::actions::StaticFile::Copy),
                state::actions::Project::Settings(state::actions::StaticFile::Modify),
                state::actions::Project::Settings(state::actions::StaticFile::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Project::Settings(state::actions::StaticFile::Remove),
                state::actions::Project::Settings(state::actions::StaticFile::Rename),
                state::actions::Project::Settings(state::actions::StaticFile::Move),
                state::actions::Project::Settings(state::actions::StaticFile::Copy),
                state::actions::Project::Settings(state::actions::StaticFile::Modify),
                state::actions::Project::Settings(state::actions::StaticFile::Corrupt),
            ]),
        }

        match state.analyses {
            state::Resource::NotPresent => actions.push(state::actions::Project::Analyses(
                state::actions::Manifest::Create,
            )),

            state::Resource::Invalid => actions.extend(vec![
                state::actions::Project::Analyses(state::actions::Manifest::Remove),
                state::actions::Project::Analyses(state::actions::Manifest::Rename),
                state::actions::Project::Analyses(state::actions::Manifest::Move),
                state::actions::Project::Analyses(state::actions::Manifest::Copy),
                state::actions::Project::Analyses(state::actions::Manifest::Repair),
            ]),

            state::Resource::Valid => actions.extend(vec![
                state::actions::Project::Analyses(state::actions::Manifest::Remove),
                state::actions::Project::Analyses(state::actions::Manifest::Rename),
                state::actions::Project::Analyses(state::actions::Manifest::Move),
                state::actions::Project::Analyses(state::actions::Manifest::Copy),
                state::actions::Project::Analyses(state::actions::Manifest::Corrupt),
                state::actions::Project::Analyses(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Add,
                )),
                state::actions::Project::Analyses(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Remove,
                )),
                state::actions::Project::Analyses(state::actions::Manifest::Modify(
                    state::actions::ModifyManifest::Alter,
                )),
            ]),
        }

        actions
    }
}

impl Simulator {
    fn perform_actions(&self, actions: Vec<state::Action>) -> Result<(), io::Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    current_tick: usize,
    pub app: state::app::State,
    pub fs: state::fs::State,
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
