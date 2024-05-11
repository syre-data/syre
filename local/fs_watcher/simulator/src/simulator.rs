use crate::{
    action::{self, Action},
    event_validator,
    state::{self, actions::Manifest},
};
use crossbeam::channel::{Receiver, Sender};
use options::Options;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{
    assert_matches::assert_matches,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    thread,
};
use syre_core::types::ResourceId;
use syre_fs_watcher::{self as watcher, config::AppConfig};

pub struct Simulator {
    options: Options,
    state: State,
    rng: ChaCha8Rng,
    validation_rx: Receiver<event_validator::error::Validation>,
    command_tx: Sender<watcher::Command>,
    watcher_thread: thread::JoinHandle<()>,
    validation_thread: thread::JoinHandle<()>,
}

impl Simulator {
    pub fn new(options: Options) -> Self {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();
        let (event_expect_tx, event_expect_rx) = crossbeam::channel::unbounded();
        let (validation_tx, validation_rx) = crossbeam::channel::unbounded();

        let rng = ChaCha8Rng::seed_from_u64(options.seed());
        let watcher = watcher::FsWatcher::new(command_rx, event_tx, options.app_config().clone());
        let watcher_thread = thread::Builder::new()
            .name("syre fs watcher simulator watcher".into())
            .spawn(move || {
                watcher.run().unwrap();
            })
            .unwrap();

        let mut validator =
            event_validator::EventValidator::new(event_rx, event_expect_rx, validation_tx);
        let validation_thread = thread::Builder::new()
            .name("syre fs watcher simulator event validation".into())
            .spawn(move || {
                validator.run().unwrap();
            })
            .unwrap();

        Self {
            options,
            state: State::default(),
            rng,
            command_tx,
            validation_rx,
            watcher_thread,
            validation_thread,
        }
    }
}

impl Simulator {
    pub fn run(&mut self) {
        self.init();
        while self.state.current_tick < self.options.max_ticks() {
            tracing::debug!(?self.state.current_tick);
            let action_count = self.rng.gen_range(self.options.action_count_range());
            let (app_actions, app_state_final) =
                Self::choose_actions(action_count, self.state.app.clone(), &mut self.rng);

            tracing::debug!(?app_actions);
            let sim_actions = Self::convert_actions_app_to_simulator(
                &app_actions,
                self.options.app_config(),
                &self.state.app,
                &mut self.rng,
            );

            tracing::debug!(?sim_actions);
            self.perform_simulator_actions(sim_actions).unwrap();
            // let events = self.event_rx.recv().unwrap();
            // match events {
            //     Ok(events) => {
            //         if let Err(err) = Self::verify_app_action_events(&events, &app_actions) {
            //             panic!("{err:?}");
            //         }
            //     }

            //     Err(errors) => {
            //         tracing::debug!(?errors);
            //     }
            // }

            self.state.current_tick += 1;
            self.state.app = app_state_final;
        }
    }

    fn init(&self) {
        let user_manifest = self
            .options
            .base_path()
            .join(self.options.app_config().user_manifest());
        let project_manifest = self
            .options
            .base_path()
            .join(self.options.app_config().project_manifest());

        self.command_tx
            .send(watcher::Command::Watch(user_manifest))
            .unwrap();
        self.command_tx
            .send(watcher::Command::Watch(project_manifest))
            .unwrap();
    }
}

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
        mut state: state::app::State,
        rng: &mut R,
    ) -> (Vec<state::Action>, state::app::State)
    where
        R: rand::Rng,
    {
        let num = num as usize;
        let mut actions = Vec::with_capacity(num);
        while actions.len() < num {
            let action = Self::choose_action(&state, rng);
            state.transition(&action).unwrap();
            // fs_state.transition(&action).unwrap();
            actions.push(action);
        }

        (actions, state)
    }

    fn choose_action<R>(state: &state::app::State, rng: &mut R) -> state::Action
    where
        R: rand::Rng,
    {
        let mut valid_actions = Self::valid_actions(&state, rng);
        let index = rng.gen_range(0..valid_actions.len());
        valid_actions.swap_remove(index)
    }

    /// Returns a list of all valid actions given a state.
    fn valid_actions<R>(state: &state::app::State, rng: &mut R) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        let mut actions = Self::valid_actions_app(state)
            .into_iter()
            .map(|action| action.into())
            .collect::<Vec<state::Action>>();

        if matches!(state.project_manifest, state::Resource::Valid(_)) {
            actions.push(state::Action::CreateProject {
                id: ResourceId::new(),
                path: utils::random_file_name(rng),
            });
        }

        for project in state.projects.iter() {
            actions.extend(
                Self::valid_actions_project(project, rng)
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

    fn valid_actions_app(state: &state::app::State) -> Vec<state::actions::AppResource> {
        use crate::state::actions::{AppResource, Manifest, ModifyManifest};

        let mut actions = vec![];
        match state.user_manifest {
            state::Resource::NotPresent => {
                actions.push(AppResource::UserManifest(Manifest::Create))
            }

            state::Resource::Invalid => {
                actions.extend(vec![
                    AppResource::UserManifest(Manifest::Remove),
                    AppResource::UserManifest(Manifest::Rename),
                    AppResource::UserManifest(Manifest::Move),
                    AppResource::UserManifest(Manifest::Repair),
                ]);
            }

            state::Resource::Valid(_) => {
                actions.extend(vec![
                    AppResource::UserManifest(Manifest::Remove),
                    AppResource::UserManifest(Manifest::Rename),
                    AppResource::UserManifest(Manifest::Move),
                    AppResource::UserManifest(Manifest::Corrupt),
                    AppResource::UserManifest(Manifest::Modify(ModifyManifest::Add)),
                    AppResource::UserManifest(Manifest::Modify(ModifyManifest::Remove)),
                    AppResource::UserManifest(Manifest::Modify(ModifyManifest::Alter)),
                ]);
            }
        }

        match state.project_manifest {
            state::Resource::NotPresent => {
                actions.push(AppResource::ProjectManifest(Manifest::Create))
            }

            state::Resource::Invalid => {
                actions.extend(vec![
                    AppResource::ProjectManifest(Manifest::Remove),
                    AppResource::ProjectManifest(Manifest::Rename),
                    AppResource::ProjectManifest(Manifest::Move),
                    AppResource::ProjectManifest(Manifest::Repair),
                ]);
            }

            state::Resource::Valid(_) => {
                actions.extend(vec![
                    AppResource::ProjectManifest(Manifest::Remove),
                    AppResource::ProjectManifest(Manifest::Rename),
                    AppResource::ProjectManifest(Manifest::Move),
                    AppResource::ProjectManifest(Manifest::Corrupt),
                    AppResource::ProjectManifest(Manifest::Modify(ModifyManifest::Add)),
                    AppResource::ProjectManifest(Manifest::Modify(ModifyManifest::Remove)),
                    AppResource::ProjectManifest(Manifest::Modify(ModifyManifest::Alter)),
                ]);
            }
        }

        actions
    }

    fn valid_actions_project<R>(
        state: &state::Project,
        rng: &mut R,
    ) -> Vec<state::actions::ProjectResource>
    where
        R: rand::Rng,
    {
        use crate::state::actions::{Dir, Project, ProjectResource, ResourceDir};

        let mut actions = vec![
            Project::Project(ResourceDir::Remove).into(),
            Project::Project(ResourceDir::Rename {
                to: utils::random_file_name(rng),
            })
            .into(),
            Project::Project(ResourceDir::Move {
                to: utils::random_file_name(rng),
            })
            .into(),
            Project::Project(ResourceDir::Copy {
                to: utils::random_file_name(rng),
            })
            .into(),
        ];

        actions.extend(
            Self::valid_actions_project_resource(state, rng)
                .into_iter()
                .map(|action| action.into())
                .collect::<Vec<ProjectResource>>(),
        );

        match &state.data {
            state::Reference::NotPresent => actions.push(
                Project::DataDir(Dir::Create {
                    path: utils::random_file_name(rng),
                })
                .into(),
            ),
            state::Reference::Present(graph) => {
                actions.extend(vec![
                    Project::DataDir(Dir::Remove).into(),
                    Project::DataDir(Dir::Rename {
                        to: utils::random_file_name(rng),
                    })
                    .into(),
                    Project::DataDir(Dir::Move {
                        to: utils::random_file_name(rng),
                    })
                    .into(),
                    Project::DataDir(Dir::Copy {
                        to: utils::random_file_name(rng),
                    })
                    .into(),
                ]);

                actions.extend(Self::valid_actions_project_data(&graph, rng));
            }
        }

        actions
    }

    fn valid_actions_project_resource<R>(
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
                actions.extend(vec![
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Remove),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Rename),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Move),
                    state::actions::Project::ConfigDir(state::actions::StaticDir::Copy),
                ]);

                actions.extend(Self::valid_actions_project_config(&config));
            }
        }

        match &state.analyses {
            None => {}
            Some(state::Reference::NotPresent) => actions.push(
                state::actions::Project::AnalysisDir(state::actions::Dir::Create {
                    path: utils::random_file_name(rng),
                }),
            ),

            Some(state::Reference::Present(path)) => {
                actions.extend(vec![
                    state::actions::Project::AnalysisDir(state::actions::Dir::Remove),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Rename {
                        to: utils::random_file_name(rng),
                    }),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Move {
                        to: utils::random_move_path(path, &state.path, rng),
                    }),
                    state::actions::Project::AnalysisDir(state::actions::Dir::Copy {
                        to: utils::random_move_path(path, &state.path, rng),
                    }),
                ]);
            }
        }

        actions
    }

    fn valid_actions_project_data<R>(
        state: &state::Data,
        rng: &mut R,
    ) -> Vec<state::actions::ProjectResource>
    where
        R: rand::Rng,
    {
        let mut actions = vec![];

        for node in state.nodes() {
            actions.extend(Self::valid_actions_container(node.borrow().deref(), rng));
        }

        actions
    }

    fn valid_actions_container<R>(
        state: &state::Container,
        rng: &mut R,
    ) -> Vec<state::actions::ProjectResource>
    where
        R: rand::Rng,
    {
        use crate::state::app::{Reference, Resource};

        let mut actions = vec![
            state::actions::ProjectResource::CreateAssetFile {
                container: state.rid().clone(),
                name: utils::random_file_name(rng),
            },
            state::actions::ProjectResource::CreateContainer {
                parent: state.rid().clone(),
                name: utils::random_file_name(rng),
            },
        ];

        match &state.config {
            state::Reference::NotPresent => {
                actions.push(state::actions::ProjectResource::Container {
                    container: state.rid().clone(),
                    action: state::actions::Container::ConfigDir(state::actions::StaticDir::Create)
                        .into(),
                })
            }

            state::Reference::Present(config) => actions.extend(
                Self::valid_actions_container_config(&config)
                    .into_iter()
                    .map(|action| state::actions::ProjectResource::Container {
                        container: state.rid().clone(),
                        action,
                    }),
            ),
        }

        if let Reference::Present(config) = &state.config {
            if let Resource::Valid(assets) = &config.assets {
                for asset in assets.iter() {
                    actions.extend(Self::valid_actions_asset(asset, rng).into_iter().map(
                        |action| state::actions::ProjectResource::AssetFile {
                            container: state.rid().clone(),
                            asset: asset.rid().clone(),
                            action,
                        },
                    ));
                }
            }
        }

        actions
    }

    fn valid_actions_asset<R>(state: &state::Asset, rng: &mut R) -> Vec<state::actions::AssetFile>
    where
        R: rand::Rng,
    {
        use crate::state::actions::{AssetFile, ProjectResource};

        match state.file {
            state::Reference::NotPresent => {
                todo!();
            }

            state::Reference::Present(_) => {
                vec![
                    AssetFile::Remove,
                    AssetFile::Rename,
                    AssetFile::Move,
                    AssetFile::Copy,
                    AssetFile::Modify,
                ]
            }
        }
    }

    fn valid_actions_container_config(
        state: &state::ContainerConfig,
    ) -> Vec<state::actions::Container> {
        use crate::state::{
            actions::{Container, ModifyManifest, StaticFile},
            Resource,
        };

        let mut actions = vec![];
        match &state.properties {
            Resource::NotPresent => actions.push(Container::Properties(StaticFile::Create)),

            Resource::Invalid => actions.extend(vec![
                Container::Properties(StaticFile::Remove),
                Container::Properties(StaticFile::Rename),
                Container::Properties(StaticFile::Move),
                Container::Properties(StaticFile::Copy),
                Container::Properties(StaticFile::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Container::Properties(StaticFile::Remove),
                Container::Properties(StaticFile::Rename),
                Container::Properties(StaticFile::Move),
                Container::Properties(StaticFile::Copy),
                Container::Properties(StaticFile::Modify),
                Container::Properties(StaticFile::Corrupt),
            ]),
        }

        match &state.settings {
            Resource::NotPresent => actions.push(Container::Settings(StaticFile::Create)),

            Resource::Invalid => actions.extend(vec![
                Container::Settings(StaticFile::Remove),
                Container::Settings(StaticFile::Rename),
                Container::Settings(StaticFile::Move),
                Container::Settings(StaticFile::Copy),
                Container::Settings(StaticFile::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Container::Settings(StaticFile::Remove),
                Container::Settings(StaticFile::Rename),
                Container::Settings(StaticFile::Move),
                Container::Settings(StaticFile::Copy),
                Container::Settings(StaticFile::Modify),
                Container::Settings(StaticFile::Corrupt),
            ]),
        }

        match &state.assets {
            Resource::NotPresent => actions.push(Container::Assets(Manifest::Create)),

            Resource::Invalid => actions.extend(vec![
                Container::Assets(Manifest::Remove),
                Container::Assets(Manifest::Rename),
                Container::Assets(Manifest::Move),
                Container::Assets(Manifest::Copy),
                Container::Assets(Manifest::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Container::Assets(Manifest::Remove),
                Container::Assets(Manifest::Rename),
                Container::Assets(Manifest::Move),
                Container::Assets(Manifest::Copy),
                Container::Assets(Manifest::Corrupt),
                Container::Assets(Manifest::Modify(ModifyManifest::Add)),
                Container::Assets(Manifest::Modify(ModifyManifest::Remove)),
                Container::Assets(Manifest::Modify(ModifyManifest::Alter)),
            ]),
        }

        actions
    }

    fn valid_actions_project_config(state: &state::ProjectConfig) -> Vec<state::actions::Project> {
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

            state::Resource::Valid(_) => actions.extend(vec![
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

            state::Resource::Valid(_) => actions.extend(vec![
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

            state::Resource::Valid(_) => actions.extend(vec![
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
    fn convert_actions_app_to_simulator<R>(
        app_actions: &Vec<state::Action>,
        app_config: &AppConfig,
        app_state: &state::app::State,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        app_actions
            .iter()
            .flat_map(|app_action| {
                Self::convert_action_app_to_simulator(app_action, app_config, app_state, rng)
            })
            .collect()
    }

    fn convert_action_app_to_simulator<R>(
        action: &state::Action,
        app_config: &AppConfig,
        app_state: &state::app::State,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        match action {
            state::Action::App(action) => {
                Self::convert_action_app_to_simulator_app(action, app_config, rng)
            }

            state::Action::CreateProject { id, path } => {
                vec![
                    Action::App(action::app::Action::Project(action::app::Project::Create {
                        id: id.clone(),
                        path: path.clone(),
                    })),
                    Action::Watcher(action::watcher::Action::Watch(path.clone())),
                ]
            }

            state::Action::Project { project, action } => {
                let project = app_state.find_project(project).unwrap();
                Self::convert_action_app_to_simulator_project_resource(project, action, rng)
            }

            _ => todo!(),
        }
    }

    fn convert_action_app_to_simulator_app<R>(
        action: &state::actions::AppResource,
        config: &AppConfig,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        use crate::action::{fs, watcher};
        use state::actions::{AppResource, Manifest, ModifyManifest};
        match action {
            AppResource::UserManifest(action) => match action {
                Manifest::Create => {
                    vec![
                        fs::Action::file_create(config.user_manifest()).into(),
                        watcher::Action::Watch(config.user_manifest().to_path_buf()).into(),
                    ]
                }

                Manifest::Remove => {
                    vec![fs::Action::file_remove(config.user_manifest()).into()]
                }

                Manifest::Rename => vec![fs::Action::file_rename(
                    config.user_manifest(),
                    utils::random_file_name(rng),
                )
                .into()],

                Manifest::Move => {
                    let new_dir = utils::random_file_name(rng);
                    vec![
                        fs::Action::folder_create(new_dir.clone()).into(),
                        fs::Action::file_move(
                            config.user_manifest(),
                            new_dir.join(config.user_manifest()),
                        )
                        .into(),
                    ]
                }

                Manifest::Copy => {
                    let new_dir = utils::random_file_name(rng);
                    vec![
                        fs::Action::folder_create(new_dir.clone()).into(),
                        fs::Action::file_move(
                            config.user_manifest(),
                            new_dir.join(config.user_manifest()),
                        )
                        .into(),
                    ]
                }

                Manifest::Corrupt => {
                    vec![]
                }

                Manifest::Repair => {
                    vec![]
                }

                Manifest::Modify(kind) => {
                    vec![]
                }
            },

            AppResource::ProjectManifest(action) => match action {
                Manifest::Create => vec![
                    fs::Action::file_create(config.project_manifest()).into(),
                    watcher::Action::Watch(config.project_manifest().to_path_buf()).into(),
                ],

                Manifest::Remove => {
                    vec![fs::Action::file_remove(config.user_manifest()).into()]
                }

                Manifest::Rename => vec![fs::Action::file_rename(
                    config.project_manifest(),
                    utils::random_file_name(rng),
                )
                .into()],

                Manifest::Move => {
                    let new_dir = utils::random_file_name(rng);
                    vec![
                        fs::Action::folder_create(new_dir.clone()).into(),
                        fs::Action::file_move(
                            config.project_manifest(),
                            new_dir.join(config.project_manifest()),
                        )
                        .into(),
                    ]
                }

                Manifest::Copy => {
                    let new_dir = utils::random_file_name(rng);
                    vec![
                        fs::Action::folder_create(new_dir.clone()).into(),
                        fs::Action::file_move(
                            config.user_manifest(),
                            new_dir.join(config.user_manifest()),
                        )
                        .into(),
                    ]
                }

                Manifest::Corrupt => {
                    vec![]
                }
                Manifest::Repair => {
                    vec![]
                }
                Manifest::Modify(kind) => {
                    vec![]
                }
            },
        }
    }

    fn convert_action_app_to_simulator_project_resource<R>(
        project: &state::Project,
        action: &state::actions::ProjectResource,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        use state::actions::ProjectResource;

        match action {
            ProjectResource::Project(action) => {
                Self::convert_action_app_to_simulator_project(project, action, rng)
            }

            ProjectResource::CreateContainer { parent, name } => {
                todo!();
            }

            ProjectResource::Container { container, action } => {
                Self::convert_action_app_to_simulator_container(project, container, action, rng)
            }

            ProjectResource::CreateAssetFile { container, name } => {
                todo!();
            }

            ProjectResource::AssetFile {
                container,
                asset,
                action,
            } => vec![Self::convert_action_app_to_simulator_asset_file(
                project, container, asset, action, rng,
            )
            .into()],
        }
    }

    fn convert_action_app_to_simulator_project<R>(
        project: &state::app::Project,
        action: &state::actions::Project,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        use crate::{
            action::{app, fs, watcher},
            state::{
                actions::{Dir, Project, ResourceDir, StaticDir, StaticFile},
                app::Reference,
            },
        };
        use syre_local::common;

        match action {
            Project::Project(action) => match action {
                ResourceDir::Remove => vec![
                    fs::Action::folder_remove(project.path.clone()).into(),
                    watcher::Action::Unwatch(project.path.clone()).into(),
                ],

                ResourceDir::Rename { to } => vec![
                    fs::Action::folder_rename(project.path.clone(), to).into(),
                    app::Action::Project(app::Project::Move {
                        id: project.rid().clone(),
                        to: project.path.clone(),
                    })
                    .into(),
                ],

                ResourceDir::Move { to } => vec![
                    fs::Action::folder_rename(project.path.clone(), to).into(),
                    app::Action::Project(app::Project::Move {
                        id: project.rid().clone(),
                        to: project.path.clone(),
                    })
                    .into(),
                ],

                ResourceDir::Copy { to } => {
                    vec![fs::Action::folder_copy(project.path.clone(), to).into()]
                }
            },

            Project::ConfigDir(action) => {
                let path = common::app_dir_of(&project.path);
                match action {
                    StaticDir::Create => {
                        vec![fs::Action::folder_create(path).into()]
                    }

                    StaticDir::Remove => {
                        vec![fs::Action::folder_remove(path).into()]
                    }

                    StaticDir::Rename => {
                        vec![fs::Action::folder_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticDir::Move => vec![fs::Action::folder_move(
                        path.clone(),
                        utils::random_move_path(&path, &project.path, rng),
                    )
                    .into()],

                    StaticDir::Copy => vec![fs::Action::folder_copy(
                        path.clone(),
                        utils::random_move_path(&path, &project.path, rng),
                    )
                    .into()],
                }
            }

            Project::AnalysisDir(action) => match action {
                Dir::Create { path } => {
                    vec![fs::Action::folder_create(project.path.join(path)).into()]
                }

                Dir::Remove => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    vec![fs::Action::folder_remove(project.path.join(path)).into()]
                }

                Dir::Rename { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    vec![fs::Action::folder_rename(project.path.join(path), to.clone()).into()]
                }

                Dir::Move { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    vec![fs::Action::folder_move(project.path.join(path), to.clone()).into()]
                }

                Dir::Copy { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    vec![fs::Action::folder_copy(project.path.join(path), to.clone()).into()]
                }
            },

            Project::DataDir(action) => match action {
                Dir::Create { path } => {
                    vec![fs::Action::folder_create(project.path.join(path)).into()]
                }

                Dir::Remove => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    vec![fs::Action::folder_remove(project.path.join(data.root_path())).into()]
                }

                Dir::Rename { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    vec![
                        fs::Action::folder_rename(project.path.join(data.root_path()), to.clone())
                            .into(),
                    ]
                }

                Dir::Move { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    vec![
                        fs::Action::folder_move(project.path.join(data.root_path()), to.clone())
                            .into(),
                    ]
                }

                Dir::Copy { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    vec![
                        fs::Action::folder_copy(project.path.join(data.root_path()), to.clone())
                            .into(),
                    ]
                }
            },

            Project::Properties(action) => {
                let path = common::project_file_of(&project.path);
                match action {
                    StaticFile::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    StaticFile::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    StaticFile::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Move => vec![
                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into(),
                    ],

                    StaticFile::Copy => {
                        vec![fs::Action::file_copy(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Corrupt => {
                        vec![]
                    }

                    StaticFile::Repair => {
                        vec![]
                    }

                    StaticFile::Modify => {
                        vec![]
                    }
                }
            }

            Project::Settings(action) => {
                let path = common::project_settings_file_of(&project.path);
                match action {
                    StaticFile::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    StaticFile::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    StaticFile::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Move => vec![
                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into(),
                    ],

                    StaticFile::Copy => {
                        vec![fs::Action::file_copy(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Corrupt => {
                        vec![]
                    }

                    StaticFile::Repair => {
                        vec![]
                    }

                    StaticFile::Modify => {
                        vec![]
                    }
                }
            }

            Project::Analyses(action) => {
                let path = common::analyses_file_of(&project.path);
                match action {
                    Manifest::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    Manifest::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    Manifest::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    Manifest::Move => vec![
                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into(),
                    ],

                    Manifest::Copy => {
                        vec![fs::Action::file_copy(path, utils::random_file_name(rng)).into()]
                    }

                    Manifest::Corrupt => {
                        vec![]
                    }

                    Manifest::Repair => {
                        vec![]
                    }

                    Manifest::Modify(kind) => {
                        vec![]
                    }
                }
            }
        }
    }

    fn convert_action_app_to_simulator_container<R>(
        project: &state::Project,
        container: &ResourceId,
        action: &state::actions::Container,
        rng: &mut R,
    ) -> Vec<Action>
    where
        R: rand::Rng,
    {
        use crate::{
            action::fs,
            state::{
                actions::{Container, Manifest, ResourceDir, StaticDir, StaticFile},
                app::Reference,
            },
        };
        use syre_local::common;

        let Reference::Present(data) = &project.data else {
            unreachable!();
        };

        let container = data.graph.find(container).unwrap();
        let container = container.borrow();
        match action {
            Container::Container(action) => {
                let path = project.path.join(&container.path);
                match action {
                    ResourceDir::Remove => {
                        vec![fs::Action::folder_remove(path).into()]
                    }

                    ResourceDir::Rename { to } => {
                        vec![fs::Action::folder_rename(path, utils::random_file_name(rng)).into()]
                    }

                    ResourceDir::Move { to } => {
                        vec![fs::Action::folder_move(path, to.clone()).into()]
                    }

                    ResourceDir::Copy { to } => {
                        vec![fs::Action::folder_copy(path, to.clone()).into()]
                    }
                }
            }

            Container::ConfigDir(action) => {
                let path = common::app_dir_of(&container.path);
                match action {
                    StaticDir::Create => {
                        vec![fs::Action::folder_create(path).into()]
                    }

                    StaticDir::Remove => {
                        vec![fs::Action::folder_remove(path).into()]
                    }

                    StaticDir::Rename => {
                        vec![fs::Action::folder_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticDir::Move => {
                        vec![fs::Action::folder_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    StaticDir::Copy => {
                        vec![fs::Action::folder_copy(
                            path.clone(),
                            utils::random_move_path(path, &project.path, rng),
                        )
                        .into()]
                    }
                }
            }

            Container::Properties(action) => {
                let path = common::container_file_of(&container.path);
                match action {
                    StaticFile::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    StaticFile::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    StaticFile::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Move => {
                        vec![fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    StaticFile::Copy => {
                        vec![fs::Action::file_copy(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    StaticFile::Corrupt => {
                        vec![]
                    }

                    StaticFile::Repair => {
                        vec![]
                    }

                    StaticFile::Modify => {
                        vec![]
                    }
                }
            }

            Container::Settings(action) => {
                let path = common::container_settings_file_of(&container.path);
                match action {
                    StaticFile::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    StaticFile::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    StaticFile::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    StaticFile::Move => {
                        vec![fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    StaticFile::Copy => {
                        vec![fs::Action::file_copy(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    StaticFile::Corrupt => {
                        vec![]
                    }

                    StaticFile::Repair => {
                        vec![]
                    }

                    StaticFile::Modify => {
                        vec![]
                    }
                }
            }

            Container::Assets(action) => {
                let path = common::assets_file_of(&container.path);
                match action {
                    Manifest::Create => {
                        vec![fs::Action::file_create(path).into()]
                    }

                    Manifest::Remove => {
                        vec![fs::Action::file_remove(path).into()]
                    }

                    Manifest::Rename => {
                        vec![fs::Action::file_rename(path, utils::random_file_name(rng)).into()]
                    }

                    Manifest::Move => {
                        vec![fs::Action::file_move(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    Manifest::Copy => {
                        vec![fs::Action::file_copy(
                            path.clone(),
                            utils::random_move_path(&path, &project.path, rng),
                        )
                        .into()]
                    }

                    Manifest::Corrupt => {
                        vec![]
                    }

                    Manifest::Repair => {
                        vec![]
                    }

                    Manifest::Modify(kind) => {
                        vec![]
                    }
                }
            }
        }
    }

    fn convert_action_app_to_simulator_asset_file<R>(
        project: &state::Project,
        container: &ResourceId,
        asset: &ResourceId,
        action: &state::actions::AssetFile,
        rng: &mut R,
    ) -> action::fs::Action
    where
        R: rand::Rng,
    {
        use crate::{
            action::fs,
            state::{actions::AssetFile, app::Reference},
        };

        let Reference::Present(data) = &project.data else {
            unreachable!();
        };

        let container = data.graph.find(container).unwrap();
        let container = container.borrow();
        let asset = container.find_asset(asset).unwrap();
        let path = project.path.join(&container.path).join(&asset.path);
        match action {
            AssetFile::Remove => fs::Action::file_remove(path),
            AssetFile::Rename => fs::Action::file_rename(path, utils::random_file_name(rng)),
            AssetFile::Move => fs::Action::file_move(
                path.clone(),
                utils::random_move_path(&path, &project.path, rng),
            ),

            AssetFile::Modify => {
                todo!()
            }

            AssetFile::Copy => fs::Action::file_copy(
                path.clone(),
                utils::random_move_path(&path, &project.path, rng),
            ),
        }
    }
}

impl Simulator {
    fn perform_simulator_actions(&self, actions: Vec<Action>) -> io::Result<()> {
        for action in actions {
            match action {
                Action::Watcher(action) => self.perform_watcher_action(action),
                Action::Fs(action) => Self::perform_fs_action(action, self.options.base_path())?,
                Action::App(action) => Self::perform_app_action(action, self.options.base_path())?,
            }
        }

        Ok(())
    }
}

impl Simulator {
    fn perform_watcher_action(&self, action: action::watcher::Action) {
        match action {
            action::watcher::Action::Watch(path) => self
                .command_tx
                .send(watcher::Command::Watch(self.options.base_path().join(path)))
                .unwrap(),

            action::watcher::Action::Unwatch(path) => self
                .command_tx
                .send(watcher::Command::Unwatch(
                    self.options.base_path().join(path),
                ))
                .unwrap(),
        }
    }

    fn perform_app_action(
        action: action::app::Action,
        base_path: impl AsRef<Path>,
    ) -> io::Result<()> {
        use action::app::{Action, Project};
        match action {
            Action::Project(Project::Create { id, path }) => {
                let path = base_path.as_ref().join(path);
                let mut project = syre_local::project::resources::Project::new(path).unwrap();
                project.rid = id;
                project.save().unwrap();

                Ok(())
            }

            Action::Project(Project::Move { id, to }) => {
                let path = base_path.as_ref().join(to);
                Ok(())
            }
        }
    }
}

impl Simulator {
    fn perform_fs_action(
        action: action::fs::Action,
        base_path: impl AsRef<Path>,
    ) -> io::Result<()> {
        match action.resource() {
            action::fs::Resource::File => Self::perform_fs_action_file(action.action(), base_path),
            action::fs::Resource::Folder => {
                Self::perform_fs_action_folder(action.action(), base_path)
            }
        }
    }

    fn perform_fs_action_file(
        action: &action::fs::ResourceAction,
        base_path: impl AsRef<Path>,
    ) -> io::Result<()> {
        let base_path = base_path.as_ref();
        match action {
            action::fs::ResourceAction::Create(path) => {
                fs::File::create(base_path.join(path))?;
            }

            action::fs::ResourceAction::Remove(path) => {
                fs::remove_file(base_path.join(path))?;
            }

            action::fs::ResourceAction::Rename { from, to } => {
                let to = from.parent().unwrap().join(to);
                fs::rename(base_path.join(from), base_path.join(to))?;
            }

            action::fs::ResourceAction::Move { from, to } => {
                fs::rename(base_path.join(from), base_path.join(to))?;
            }

            action::fs::ResourceAction::Copy { from, to } => {
                fs::copy(base_path.join(from), base_path.join(to))?;
            }
        }

        Ok(())
    }

    fn perform_fs_action_folder(
        action: &action::fs::ResourceAction,
        base_path: impl AsRef<Path>,
    ) -> io::Result<()> {
        let base_path = base_path.as_ref();
        match action {
            action::fs::ResourceAction::Create(path) => {
                fs::create_dir(base_path.join(path))?;
            }

            action::fs::ResourceAction::Remove(path) => {
                fs::remove_dir(base_path.join(path))?;
            }

            action::fs::ResourceAction::Rename { from, to } => {
                let to = from.parent().unwrap().join(to);
                fs::rename(base_path.join(from), base_path.join(to))?;
            }

            action::fs::ResourceAction::Move { from, to } => {
                fs::rename(base_path.join(from), base_path.join(to))?;
            }

            action::fs::ResourceAction::Copy { from, to } => {
                utils::copy_dir(base_path.join(from), base_path.join(to))?;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    current_tick: usize,
    pub app: state::app::State,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

mod utils {
    use rand::distributions::{self, DistString, Distribution};
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };
    use walkdir::WalkDir;

    pub fn random_file_name<R>(rng: &mut R) -> PathBuf
    where
        R: rand::Rng,
    {
        PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
    }

    /// Gets a random path within the root path.
    /// Weights the likelihood to select a path based on the distance between
    /// each path and the base path.
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
    pub fn path_distance(a: impl AsRef<Path>, b: impl AsRef<Path>) -> usize {
        let mut a = a.as_ref().components();
        let mut b = b.as_ref().components();

        while a.next() == b.next() {}
        a.count() + b.count()
    }

    /// Copy the contents of a directory to a new location.
    /// Ignores symlinks and files or folders that already exist.
    pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = from.as_ref();
        for entry in WalkDir::new(from)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let origin = entry.path();
            let destination = to.as_ref().join(origin.strip_prefix(from).unwrap());
            if entry.file_type().is_dir() {
                if let Err(err) = fs::create_dir(&destination) {
                    match err.kind() {
                        io::ErrorKind::AlreadyExists => {}
                        _ => return Err(err),
                    }
                }
            } else if entry.file_type().is_file() {
                fs::copy(origin, &destination)?;
            }
        }

        Ok(())
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

mod error {
    use crate::state;
    use syre_fs_watcher::Event;

    #[derive(Debug)]
    pub struct ActionEventDiscrepency {
        action: state::Action,
        event: Event,
    }
}

#[cfg(test)]
#[path = "simulator_test.rs"]
mod simulator_test;
