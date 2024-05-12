use crate::{
    // action::{self, Action},
    event_validator::{self, error::Validation},
    state::{self, action},
};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
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
use syre_fs_watcher::{self as watcher};

type Result<T = ()> = std::result::Result<T, error::Error>;

pub struct Simulator {
    options: Options,
    state: State,
    rng: ChaCha8Rng,
    validation_rx: Receiver<event_validator::error::Validation>,
    command_tx: Sender<watcher::Command>,
    event_expect_tx: Sender<Vec<watcher::Event>>,
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
            event_expect_tx,
            watcher_thread,
            validation_thread,
        }
    }
}

impl Simulator {
    pub fn run(&mut self) {
        while self.state.current_tick < self.options.max_ticks() {
            tracing::debug!(?self.state.current_tick);
            let action_count = self.rng.gen_range(self.options.action_count_range());
            let (actions, app_state_final) =
                Self::choose_actions(action_count, self.state.app.clone(), &mut self.rng);

            tracing::debug!(?actions);
            self.perform_actions(actions).unwrap();
            match self.validation_rx.try_recv() {
                Ok(Validation { expected, received }) => {
                    tracing::error!(
                        "event validation failed: expected {expected:?} found {received:?}"
                    );
                    break;
                }

                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    tracing::error!("event validation channel closed. shutting down");
                    break;
                }
            }

            self.state.current_tick += 1;
        }
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

    fn valid_actions_app(state: &state::app::State) -> Vec<action::AppResource> {
        use crate::state::action::{AppResource, Manifest, ModifyManifest};

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

    fn valid_actions_project<R>(state: &state::Project, rng: &mut R) -> Vec<action::ProjectResource>
    where
        R: rand::Rng,
    {
        use crate::state::action::{Dir, Project, ProjectResource, ResourceDir};

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
            state::Reference::NotPresent => actions.push(ProjectResource::CreateDataDir {
                id: ResourceId::new(),
                path: utils::random_file_name(rng),
            }),
            state::Reference::Present(graph) => {
                actions.extend(vec![
                    Project::DataDir(ResourceDir::Remove).into(),
                    Project::DataDir(ResourceDir::Rename {
                        to: utils::random_file_name(rng),
                    })
                    .into(),
                    Project::DataDir(ResourceDir::Move {
                        to: utils::random_file_name(rng),
                    })
                    .into(),
                    Project::DataDir(ResourceDir::Copy {
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
    ) -> Vec<state::action::Project>
    where
        R: rand::Rng,
    {
        use crate::state::{
            action::{Dir, Project, StaticDir},
            Reference,
        };

        let mut actions = vec![];
        match &state.config {
            Reference::NotPresent => {
                actions.push(Project::ConfigDir(StaticDir::Create));
            }

            Reference::Present(config) => {
                actions.extend(vec![
                    Project::ConfigDir(StaticDir::Remove),
                    Project::ConfigDir(StaticDir::Rename),
                    Project::ConfigDir(StaticDir::Move),
                    Project::ConfigDir(StaticDir::Copy),
                ]);

                actions.extend(Self::valid_actions_project_config(&config));
            }
        }

        match &state.analyses {
            None => {}
            Some(Reference::NotPresent) => actions.push(Project::AnalysisDir(Dir::Create {
                path: utils::random_file_name(rng),
            })),

            Some(Reference::Present(path)) => {
                actions.extend(vec![
                    Project::AnalysisDir(Dir::Remove),
                    Project::AnalysisDir(Dir::Rename {
                        to: utils::random_file_name(rng),
                    }),
                    Project::AnalysisDir(Dir::Move {
                        to: utils::random_move_path(path, &state.path, rng),
                    }),
                    Project::AnalysisDir(Dir::Copy {
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
    ) -> Vec<state::action::ProjectResource>
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
    ) -> Vec<state::action::ProjectResource>
    where
        R: rand::Rng,
    {
        use crate::state::{
            action::{Container, ProjectResource, StaticDir},
            app::{Reference, Resource},
        };

        let mut actions = vec![
            ProjectResource::CreateContainer {
                parent: state.rid().clone(),
                id: ResourceId::new(),
                name: utils::random_file_name(rng),
            },
            ProjectResource::CreateAssetFile {
                container: state.rid().clone(),
                id: ResourceId::new(),
                name: utils::random_file_name(rng),
            },
        ];

        match &state.config {
            Reference::NotPresent => actions.push(ProjectResource::Container {
                container: state.rid().clone(),
                action: Container::ConfigDir(StaticDir::Create).into(),
            }),

            Reference::Present(config) => actions.extend(
                Self::valid_actions_container_config(&config)
                    .into_iter()
                    .map(|action| ProjectResource::Container {
                        container: state.rid().clone(),
                        action,
                    }),
            ),
        }

        if let Reference::Present(config) = &state.config {
            if let Resource::Valid(assets) = &config.assets {
                for asset in assets.iter() {
                    actions.extend(Self::valid_actions_asset(asset, rng).into_iter().map(
                        |action| ProjectResource::AssetFile {
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

    fn valid_actions_asset<R>(state: &state::Asset, rng: &mut R) -> Vec<state::action::AssetFile>
    where
        R: rand::Rng,
    {
        use crate::state::action::AssetFile;

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
    ) -> Vec<state::action::Container> {
        use crate::state::{
            action::{Container, Manifest, ModifyManifest, StaticFile},
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

    fn valid_actions_project_config(state: &state::ProjectConfig) -> Vec<state::action::Project> {
        use crate::state::{
            action::{Manifest, ModifyManifest, Project, StaticFile},
            Resource,
        };

        let mut actions = vec![];
        match state.properties {
            Resource::NotPresent => actions.push(Project::Properties(StaticFile::Create)),

            Resource::Invalid => actions.extend(vec![
                Project::Properties(StaticFile::Remove),
                Project::Properties(StaticFile::Rename),
                Project::Properties(StaticFile::Move),
                Project::Properties(StaticFile::Copy),
                Project::Properties(StaticFile::Modify),
                Project::Properties(StaticFile::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Project::Properties(StaticFile::Remove),
                Project::Properties(StaticFile::Rename),
                Project::Properties(StaticFile::Move),
                Project::Properties(StaticFile::Copy),
                Project::Properties(StaticFile::Modify),
                Project::Properties(StaticFile::Corrupt),
            ]),
        }

        match state.settings {
            Resource::NotPresent => actions.push(Project::Settings(StaticFile::Create)),

            Resource::Invalid => actions.extend(vec![
                Project::Settings(StaticFile::Remove),
                Project::Settings(StaticFile::Rename),
                Project::Settings(StaticFile::Move),
                Project::Settings(StaticFile::Copy),
                Project::Settings(StaticFile::Modify),
                Project::Settings(StaticFile::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Project::Settings(StaticFile::Remove),
                Project::Settings(StaticFile::Rename),
                Project::Settings(StaticFile::Move),
                Project::Settings(StaticFile::Copy),
                Project::Settings(StaticFile::Modify),
                Project::Settings(StaticFile::Corrupt),
            ]),
        }

        match state.analyses {
            Resource::NotPresent => actions.push(Project::Analyses(Manifest::Create)),

            Resource::Invalid => actions.extend(vec![
                Project::Analyses(Manifest::Remove),
                Project::Analyses(Manifest::Rename),
                Project::Analyses(Manifest::Move),
                Project::Analyses(Manifest::Copy),
                Project::Analyses(Manifest::Repair),
            ]),

            Resource::Valid(_) => actions.extend(vec![
                Project::Analyses(Manifest::Remove),
                Project::Analyses(Manifest::Rename),
                Project::Analyses(Manifest::Move),
                Project::Analyses(Manifest::Copy),
                Project::Analyses(Manifest::Corrupt),
                Project::Analyses(Manifest::Modify(ModifyManifest::Add)),
                Project::Analyses(Manifest::Modify(ModifyManifest::Remove)),
                Project::Analyses(Manifest::Modify(ModifyManifest::Alter)),
            ]),
        }

        actions
    }
}

impl Simulator {
    fn perform_actions(&mut self, actions: Vec<action::Action>) -> Result {
        actions
            .iter()
            .map(|action| {
                let res = self.perform_action(action);
                self.state.app.transition(&action).unwrap();
                res
            })
            .collect()
    }

    fn perform_action(&mut self, action: &state::Action) -> Result {
        use crate::state::Action;

        tracing::debug!(?action);
        match action {
            Action::App(action) => self.perform_action_app(action),
            Action::CreateProject { id, path } => {
                let path = self.options.base_path().join(path);
                let mut project =
                    syre_local::project::resources::Project::new(path.clone()).unwrap();
                project.rid = id.clone();
                project.save().unwrap();

                self.watch(path)?;
                Ok(())
            }

            Action::Project { project, action } => {
                self.perform_action_project_resource(project, action)
            }

            Action::Watch(path) => {
                let path = self.options.base_path().join(path);
                self.watch(path)?;
                Ok(())
            }

            Action::Unwatch(path) => {
                let path = self.options.base_path().join(path);
                self.unwatch(path)?;
                Ok(())
            }
        }
    }

    fn perform_action_app(&mut self, action: &state::action::AppResource) -> Result {
        use state::action::{AppResource, Manifest};

        match action {
            AppResource::UserManifest(action) => match action {
                Manifest::Create => {
                    self.create_file(self.options.app_config().user_manifest())?;
                    self.watch(self.options.app_config().user_manifest())?;
                }

                Manifest::Remove => self.remove_file(self.options.app_config().user_manifest())?,
                Manifest::Rename => {
                    let to = utils::random_file_name(&mut self.rng);
                    self.rename_file(self.options.app_config().user_manifest(), to)?;
                }

                Manifest::Move => {
                    let path = self.options.app_config().user_manifest();
                    let new_dir = utils::random_file_name(&mut self.rng);
                    self.create_folder(&new_dir)?;
                    self.move_file(&path, new_dir.join(path.file_name().unwrap()))?;
                }

                Manifest::Copy => {
                    let to = utils::random_file_name(&mut self.rng);
                    self.create_folder(&to)?;
                    self.copy_file(self.options.app_config().user_manifest(), to)?
                }

                Manifest::Corrupt => {}
                Manifest::Repair => {}
                Manifest::Modify(kind) => {}
            },

            AppResource::ProjectManifest(action) => match action {
                Manifest::Create => {
                    self.create_file(self.options.app_config().project_manifest())?;
                    self.watch(self.options.app_config().project_manifest())?;
                }

                Manifest::Remove => {
                    self.remove_file(self.options.app_config().project_manifest())?
                }
                Manifest::Rename => {
                    let to = utils::random_file_name(&mut self.rng);
                    self.rename_file(self.options.app_config().project_manifest(), to)?;
                }

                Manifest::Move => {
                    let path = self.options.app_config().project_manifest();
                    let new_dir = utils::random_file_name(&mut self.rng);
                    self.create_folder(&new_dir)?;
                    self.move_file(&path, new_dir.join(path.file_name().unwrap()))?;
                }

                Manifest::Copy => {
                    let to = utils::random_file_name(&mut self.rng);
                    self.create_folder(&to)?;
                    self.copy_file(self.options.app_config().project_manifest(), to)?
                }

                Manifest::Corrupt => {}
                Manifest::Repair => {}
                Manifest::Modify(kind) => {}
            },
        }

        Ok(())
    }

    fn perform_action_project_resource(
        &mut self,
        project: &ResourceId,
        action: &state::action::ProjectResource,
    ) -> Result {
        use crate::state::{action::ProjectResource, Reference};
        use syre_local::project::resources::Container;

        match action {
            ProjectResource::Project(action) => self.perform_action_project(project, action),
            ProjectResource::CreateDataDir { id, path } => {
                let project = self.state.app.find_project_mut(project).unwrap();
                assert_matches!(project.data, Reference::NotPresent);

                let path = self.options.base_path().join(&project.path).join(path);
                let mut container = Container::new(path);
                container.rid = id.clone();
                container.save()?;
                Ok(())
            }

            ProjectResource::CreateContainer { parent, id, name } => {
                let project = self.state.app.find_project_mut(project).unwrap();
                let Reference::Present(data) = &mut project.data else {
                    unreachable!();
                };

                let parent = data.find(parent).unwrap();
                let parent_path =
                    data.graph
                        .ancestors(parent)
                        .iter()
                        .fold(PathBuf::new(), |path, container| {
                            let container = container.borrow();
                            path.join(&container.path)
                        });

                let path = self
                    .options
                    .base_path()
                    .join(&project.path)
                    .join(data.root_path())
                    .join(parent_path)
                    .join(name);

                let mut container = Container::new(path);
                container.rid = id.clone();
                container.save()?;
                Ok(())
            }

            ProjectResource::CreateAssetFile {
                container,
                id,
                name,
            } => {
                let project = self.state.app.find_project_mut(project).unwrap();
                let Reference::Present(data) = &mut project.data else {
                    unreachable!();
                };

                let container = data.find(container).unwrap();
                let container_path = data.graph.ancestors(container).iter().fold(
                    PathBuf::new(),
                    |path, container| {
                        let container = container.borrow();
                        path.join(&container.path)
                    },
                );

                let path = self
                    .options
                    .base_path()
                    .join(&project.path)
                    .join(data.root_path())
                    .join(container_path)
                    .join(name);

                self.create_file(path)?;
                Ok(())
            }

            ProjectResource::Container { container, action } => {
                self.perform_action_container(project, container, action)
            }

            ProjectResource::AssetFile {
                container,
                asset,
                action,
            } => self.perform_action_asset_file(project, container, asset, action),
        }
    }

    fn perform_action_project(
        &mut self,
        project: &ResourceId,
        action: &state::action::Project,
    ) -> Result {
        use crate::state::{
            action::{Dir, Manifest, Project, ResourceDir, StaticDir, StaticFile},
            app::{ProjectConfig, Reference, Resource},
        };
        use syre_local::{common, project::resources::Container};

        let project = self.state.app.find_project(&project).unwrap();
        match action {
            Project::Project(action) => match action {
                ResourceDir::Remove => {
                    self.remove_folder(&project.path)?;
                    self.unwatch(project.path.clone())?;
                }

                ResourceDir::Rename { to } => {
                    self.rename_folder(&project.path, &to)?;
                    self.unwatch(project.path.clone())?;
                    self.watch(to)?;
                }

                ResourceDir::Move { to } => {
                    self.create_folder(&to)?;
                    self.move_folder(&project.path, &to)?;
                    self.unwatch(project.path.clone())?;
                    self.watch(to)?;
                }

                ResourceDir::Copy { to } => {
                    self.copy_folder(&project.path, to)?;
                    // TODO: Not sure if should watch new project.
                }
            },

            Project::ConfigDir(action) => {
                let path = common::app_dir_of(&project.path);
                match action {
                    StaticDir::Create => {
                        self.create_folder(path)?;
                    }

                    StaticDir::Remove => {
                        assert_matches!(project.config, Reference::Present(_));
                        self.remove_folder(path)?;
                    }

                    StaticDir::Rename => {
                        assert_matches!(project.config, Reference::Present(_));
                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_folder(path, to)?;
                    }

                    StaticDir::Move => {
                        assert_matches!(project.config, Reference::Present(_));
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_folder(path, to)?;
                    }

                    StaticDir::Copy => {
                        assert_matches!(project.config, Reference::Present(_));
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.copy_folder(path, to)?;
                    }
                }
            }

            Project::AnalysisDir(action) => match action {
                Dir::Create { path } => {
                    self.create_folder(project.path.join(path))?;
                }

                Dir::Remove => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    self.remove_folder(project.path.join(path))?;
                }

                Dir::Rename { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    self.rename_folder(project.path.join(path), to)?;
                }

                Dir::Move { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    self.create_folder(&to)?;
                    self.move_folder(project.path.join(path), to)?;
                }

                Dir::Copy { to } => {
                    let Some(Reference::Present(path)) = project.analyses.as_ref() else {
                        unreachable!();
                    };

                    self.copy_folder(project.path.join(path), to)?;
                }
            },

            Project::DataDir(action) => match action {
                ResourceDir::Remove => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    self.remove_folder(project.path.join(data.root_path()))?;
                }

                ResourceDir::Rename { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    self.rename_folder(project.path.join(data.root_path()), to)?;
                }

                ResourceDir::Move { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    self.create_folder(&to)?;
                    self.move_folder(project.path.join(data.root_path()), to)?;
                }

                ResourceDir::Copy { to } => {
                    let Reference::Present(data) = &project.data else {
                        unreachable!();
                    };

                    self.copy_folder(project.path.join(data.root_path()), to.clone())?;
                }
            },

            Project::Properties(action) => {
                let path = common::project_file_of(&project.path);
                match action {
                    StaticFile::Create => {
                        self.create_file(path)?;
                    }

                    StaticFile::Remove => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    StaticFile::Rename => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    StaticFile::Move => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    StaticFile::Copy => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    StaticFile::Corrupt => {}
                    StaticFile::Repair => {}
                    StaticFile::Modify => {}
                }
            }

            Project::Settings(action) => {
                let path = common::project_settings_file_of(&project.path);
                match action {
                    StaticFile::Create => {
                        self.create_file(path)?;
                    }

                    StaticFile::Remove => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    StaticFile::Rename => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );
                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    StaticFile::Move => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    StaticFile::Copy => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    StaticFile::Corrupt => {}
                    StaticFile::Repair => {}
                    StaticFile::Modify => {}
                }
            }

            Project::Analyses(action) => {
                let path = common::analyses_file_of(&project.path);
                match action {
                    Manifest::Create => {
                        self.create_file(path)?;
                    }

                    Manifest::Remove => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                analyses: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    Manifest::Rename => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                analyses: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    Manifest::Move => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                analyses: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        // TODO: May not want to move into other part of project.
                        // e.g. data dir
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    Manifest::Copy => {
                        assert_matches!(
                            project.config,
                            Reference::Present(ProjectConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    Manifest::Corrupt => {}
                    Manifest::Repair => {}
                    Manifest::Modify(kind) => {}
                }
            }
        }

        Ok(())
    }

    fn perform_action_container(
        &mut self,
        project: &ResourceId,
        container: &ResourceId,
        action: &state::action::Container,
    ) -> Result {
        use crate::state::{
            action::{Container, Manifest, ResourceDir, StaticDir, StaticFile},
            app::{ContainerConfig, Reference, Resource},
        };
        use syre_local::common;

        let project = self.state.app.find_project(project).unwrap();
        let Reference::Present(data) = &project.data else {
            unreachable!();
        };

        let container = data.graph.find(container).unwrap();
        let container = container.borrow();
        let path = project.path.join(data.root_path()).join(&container.path);
        match action {
            Container::Container(action) => match action {
                ResourceDir::Remove => self.remove_folder(path)?,
                ResourceDir::Rename { to } => self.rename_folder(path, to)?,
                ResourceDir::Move { to } => {
                    self.create_folder(&to)?;
                    self.move_folder(path, to)?;
                }

                ResourceDir::Copy { to } => {
                    self.copy_folder(path, to)?;
                }
            },

            Container::ConfigDir(action) => {
                let path = common::app_dir_of(path);
                match action {
                    StaticDir::Create => self.create_folder(path)?,
                    StaticDir::Remove => {
                        assert_matches!(container.config, Reference::Present(_));
                        self.remove_folder(path)?;
                    }

                    StaticDir::Rename => {
                        assert_matches!(container.config, Reference::Present(_));
                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_folder(path, to)?;
                    }

                    StaticDir::Move => {
                        assert_matches!(container.config, Reference::Present(_));
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_folder(path, to)?;
                    }

                    StaticDir::Copy => {
                        assert_matches!(container.config, Reference::Present(_));
                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.copy_folder(path, to)?;
                    }
                }
            }

            Container::Properties(action) => {
                let path = common::container_file_of(path);
                match action {
                    StaticFile::Create => self.create_file(path)?,
                    StaticFile::Remove => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    StaticFile::Rename => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    StaticFile::Move => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    StaticFile::Copy => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                properties: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    StaticFile::Corrupt => {}
                    StaticFile::Repair => {}
                    StaticFile::Modify => {}
                }
            }

            Container::Settings(action) => {
                let path = common::container_settings_file_of(path);
                match action {
                    StaticFile::Create => self.create_file(path)?,
                    StaticFile::Remove => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    StaticFile::Rename => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    StaticFile::Move => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    StaticFile::Copy => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                settings: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    StaticFile::Corrupt => {}
                    StaticFile::Repair => {}
                    StaticFile::Modify => {}
                }
            }

            Container::Assets(action) => {
                let path = common::assets_file_of(&container.path);
                match action {
                    Manifest::Create => self.create_file(path)?,
                    Manifest::Remove => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                assets: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        self.remove_file(path)?;
                    }

                    Manifest::Rename => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                assets: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_file_name(&mut self.rng);
                        self.rename_file(path, to)?;
                    }

                    Manifest::Move => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                assets: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.move_file(path, to)?;
                    }

                    Manifest::Copy => {
                        assert_matches!(
                            container.config,
                            Reference::Present(ContainerConfig {
                                assets: Resource::Valid(_) | Resource::Invalid,
                                ..
                            })
                        );

                        let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                        self.create_folder(&to)?;
                        self.copy_file(path, to)?;
                    }

                    Manifest::Corrupt => {}
                    Manifest::Repair => {}
                    Manifest::Modify(kind) => {}
                }
            }
        }

        Ok(())
    }

    fn perform_action_asset_file(
        &mut self,
        project: &ResourceId,
        container: &ResourceId,
        asset: &ResourceId,
        action: &state::action::AssetFile,
    ) -> Result {
        use crate::state::{action::AssetFile, app::Reference};

        let project = self.state.app.find_project(&project).unwrap();
        let Reference::Present(data) = &project.data else {
            unreachable!();
        };

        let container = data.graph.find(&container).unwrap();
        let container = container.borrow();
        let asset = container.find_asset(&asset).unwrap();
        let path = project
            .path
            .join(data.root_path())
            .join(&container.path)
            .join(&asset.path);
        match action {
            AssetFile::Remove => self.remove_file(path)?,
            AssetFile::Rename => {
                let to = utils::random_file_name(&mut self.rng);
                self.rename_file(path, to)?;
            }

            AssetFile::Move => {
                let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                self.create_folder(&to)?;
                self.move_file(path, to)?;
            }

            AssetFile::Copy => {
                let to = utils::random_move_path(&path, &project.path, &mut self.rng);
                self.create_folder(&to)?;
                self.copy_file(path, to)?;
            }

            AssetFile::Modify => {
                todo!()
            }
        }

        Ok(())
    }
}

impl Simulator {
    fn create_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.options.base_path().join(path);
        fs::File::create(path)?;
        Ok(())
    }

    fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.options.base_path().join(path);
        fs::remove_file(path)
    }

    fn rename_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = from.parent().unwrap().join(to);
        fs::rename(from, to)
    }

    fn move_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = self.options.base_path().join(to);
        fs::rename(from, to)
    }

    fn copy_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = from.parent().unwrap().join(to);
        fs::copy(from, to)?;
        Ok(())
    }

    fn create_folder(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.options.base_path().join(path);
        fs::create_dir_all(path)
    }

    fn remove_folder(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.options.base_path().join(path);
        fs::remove_dir_all(path)
    }

    fn rename_folder(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = from.parent().unwrap().join(to);
        fs::rename(from, to)
    }

    fn move_folder(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = self.options.base_path().join(to);
        fs::rename(from, to)
    }

    fn copy_folder(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = self.options.base_path().join(to);
        utils::copy_dir(from, to)
    }

    fn watch(
        &self,
        path: impl AsRef<Path>,
    ) -> std::result::Result<(), crossbeam::channel::SendError<watcher::Command>> {
        let path = self.options.base_path().join(path);
        self.command_tx.send(watcher::Command::Watch(path))
    }

    fn unwatch(
        &self,
        path: impl AsRef<Path>,
    ) -> std::result::Result<(), crossbeam::channel::SendError<watcher::Command>> {
        let path = self.options.base_path().join(path);
        self.command_tx.send(watcher::Command::Unwatch(path))
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

        // let kind: action::MoveKind = rng.sample(distributions::Standard);
        // match kind {
        //     action::MoveKind::Ancestor => {
        //         if let Some(parent) = base_path.parent() {
        //             let mut parent = parent.to_path_buf();
        //             parent.set_file_name(base_path.file_name().unwrap());
        //             parent
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     action::MoveKind::Descendant => {
        //         if let Some(parent) = base_path.parent() {
        //             let filename = base_path.file_name().unwrap();
        //             parent
        //                 .join(distributions::Alphanumeric.sample_string(rng, 16))
        //                 .join(filename)
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     action::MoveKind::Sibling => {
        //         if let Some(parent) = base_path.parent() {
        //         } else {
        //             PathBuf::from(distributions::Alphanumeric.sample_string(rng, 16))
        //         }
        //     }

        //     action::MoveKind::OutOfResource => {
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
    type Result<T = ()> = std::result::Result<T, Error>;

    #[derive(Debug, derive_more::From)]
    pub enum Error {
        Fs(std::io::Error),
        IoSerde(syre_local::error::IoSerde),
        Channel,
    }

    impl From<crossbeam::channel::RecvError> for Error {
        fn from(_value: crossbeam::channel::RecvError) -> Self {
            Self::Channel
        }
    }

    impl<T> From<crossbeam::channel::SendError<T>> for Error {
        fn from(_value: crossbeam::channel::SendError<T>) -> Self {
            Self::Channel
        }
    }
}

#[cfg(test)]
#[path = "simulator_test.rs"]
mod simulator_test;
