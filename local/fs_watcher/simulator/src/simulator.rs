use crate::{
    event_validator::{self, error::Validation},
    state::{self, Ptr, Reducible},
};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use options::Options;
use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::*,
};
use rand_chacha::ChaCha8Rng;
use std::{ffi::OsString, fs, io, path::Path, thread};
use syre_fs_watcher::{self as watcher};
use watcher::config::AppConfig;

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

        let base_path = options.base_path().clone();
        let state = State::new(base_path, options.app_config());
        Self {
            options,
            state,
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

        if self.state.current_tick == self.options.max_ticks() {
            tracing::debug!("simulation complete");
        }
    }
}

// TODO: Allow copy in same folder.
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
        mut state: state::State,
        rng: &mut R,
    ) -> (Vec<state::Action>, state::State)
    where
        R: rand::Rng,
    {
        let num = num as usize;
        let mut actions = Vec::with_capacity(num);
        while actions.len() < num {
            let action = Self::choose_action(&state, rng);
            state.reduce(&action).unwrap();
            actions.push(action);
        }

        (actions, state)
    }

    fn choose_action<R>(state: &state::State, rng: &mut R) -> state::Action
    where
        R: rand::Rng,
    {
        let mut valid_actions = Self::valid_actions(&state, rng);
        let index = rng.gen_range(0..valid_actions.len());
        valid_actions.swap_remove(index)
    }

    /// Returns a list of all valid actions given a state.
    fn valid_actions<R>(state: &state::State, rng: &mut R) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        let all_folders = state.fs().all_folders();
        let mut actions = Self::valid_actions_app(state, &all_folders, rng);
        for project in state.app().projects() {
            actions.extend(Self::valid_actions_project(
                project,
                state.fs(),
                &all_folders,
                rng,
            ));
        }

        actions
    }

    fn valid_actions_app<R>(
        state: &state::State,
        folders: &Vec<Ptr<state::fs::Folder>>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        let mut actions = Vec::with_capacity(16);
        let folder = &folders[rng.gen_range(0..folders.len())];
        let user_manifest = state.app().app_state().user_manifest();
        actions.extend(Self::valid_actions_app_manifest(
            user_manifest,
            state.fs(),
            folder,
            rng,
        ));

        let folder = &folders[rng.gen_range(0..folders.len())];
        let project_manifest = state.app().app_state().project_manifest();
        actions.extend(Self::valid_actions_app_manifest(
            project_manifest,
            state.fs(),
            folder,
            rng,
        ));

        actions
    }

    fn valid_actions_project<R>(
        project: &Ptr<state::app::Project>,
        fs_state: &state::fs::State,
        folders: &Vec<Ptr<state::fs::Folder>>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        use crate::state::{
            action,
            app::{FsResource, Resource},
            Action,
        };
        use syre_local::constants;

        let mut actions = Vec::with_capacity(50);
        let project = project.borrow();
        match project.fs_resource() {
            FsResource::NotPresent => actions.push(Action::CreateFolderAt {
                path: project.path().clone(),
                with_parents: true,
            }),
            FsResource::Present(folder) => {
                let parent = &folders[rng.gen_range(0..folders.len())];
                let project_ptr = folder.upgrade().unwrap();
                let project_folder = action::FsResource::Folder(project_ptr.clone());
                actions.extend([
                    Action::Remove(project_folder.clone()),
                    Action::Rename {
                        resource: project_folder.clone(),
                        to: utils::random_file_name(rng),
                    },
                ]);

                if !Ptr::ptr_eq(&project_ptr, parent) {
                    actions.extend([
                        Action::Move {
                            resource: project_folder.clone(),
                            parent: parent.clone(),
                        },
                        Action::Copy {
                            resource: project_folder.clone(),
                            parent: parent.clone(),
                        },
                    ]);
                }

                match project.config() {
                    Resource::NotPresent => actions.push(Action::CreateFolder {
                        parent: project_ptr.clone(),
                        name: constants::APP_DIR.into(),
                    }),
                    Resource::Present(config_ptr) => {
                        let config = config_ptr.borrow();
                        let config_folder_ptr = config.fs_resource().upgrade().unwrap();

                        let config_folder = action::FsResource::Folder(config_folder_ptr.clone());
                        let folder = &folders[rng.gen_range(0..folders.len())];
                        actions.extend([
                            Action::Remove(config_folder.clone()),
                            Action::Rename {
                                resource: config_folder.clone(),
                                to: utils::random_file_name(rng),
                            },
                        ]);

                        if !Ptr::ptr_eq(&config_folder_ptr, folder) {
                            actions.extend([
                                Action::Move {
                                    resource: config_folder.clone(),
                                    parent: folder.clone(),
                                },
                                Action::Copy {
                                    resource: config_folder.clone(),
                                    parent: folder.clone(),
                                },
                            ]);
                        }

                        let folder = &folders[rng.gen_range(0..folders.len())];
                        actions.extend(Self::valid_actions_project_config_resource(
                            config.properties(),
                            constants::PROJECT_FILE,
                            &config_folder_ptr,
                            &folder,
                            rng,
                        ));

                        let folder = &folders[rng.gen_range(0..folders.len())];
                        actions.extend(Self::valid_actions_project_config_resource(
                            config.settings(),
                            constants::PROJECT_FILE,
                            &config_folder_ptr,
                            &folder,
                            rng,
                        ));

                        let folder = &folders[rng.gen_range(0..folders.len())];
                        actions.extend(Self::valid_actions_project_resource_manifest(
                            config.analyses(),
                            constants::ANALYSES_FILE,
                            &config_folder_ptr,
                            &folder,
                            rng,
                        ));
                    }
                }

                if let Some(analyses) = project.analyses() {
                    match analyses.borrow().fs_resource() {
                        FsResource::NotPresent => actions.push(Action::CreateFolderAt {
                            path: analyses.borrow().path().clone(),
                            with_parents: true,
                        }),
                        FsResource::Present(analyses_ptr) => {
                            let analyses_ptr = analyses_ptr.upgrade().unwrap();
                            let analyses_folder = action::FsResource::Folder(analyses_ptr.clone());
                            let folder = &folders[rng.gen_range(0..folders.len())];
                            actions.extend([
                                Action::Remove(analyses_folder.clone()),
                                Action::Rename {
                                    resource: analyses_folder.clone(),
                                    to: utils::random_file_name(rng),
                                },
                            ]);

                            if !Ptr::ptr_eq(&analyses_ptr, folder) {
                                actions.extend([
                                    Action::Move {
                                        resource: analyses_folder.clone(),
                                        parent: folder.clone(),
                                    },
                                    Action::Copy {
                                        resource: analyses_folder.clone(),
                                        parent: folder.clone(),
                                    },
                                ]);
                            }
                        }
                    }
                }

                match project.data().borrow().graph() {
                    None => actions.push(Action::CreateFolderAt {
                        path: project.data().borrow().path().clone(),
                        with_parents: true,
                    }),
                    Some(graph) => {
                        for container in graph.nodes() {
                            actions.extend(Self::valid_actions_container(
                                container, fs_state, &folders, rng,
                            ));
                        }
                    }
                }
            }
        }

        actions
    }

    fn valid_actions_app_manifest<M, R>(
        manifest: &Ptr<M>,
        fs_state: &state::fs::State,
        folder: &Ptr<state::fs::Folder>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        M: state::app::HasPath
            + state::app::HasFsDataResource<Resource = state::fs::File>
            + state::app::Manifest,
        R: rand::Rng,
    {
        use crate::state::{
            action::{FsResource, ModifyKind},
            app::{DataResourceState, FsDataResource},
            Action,
        };

        let manifest = manifest.borrow();
        let mut actions = Vec::with_capacity(10);
        match manifest.fs_resource() {
            FsDataResource::NotPresent => {
                actions.push(Action::CreateFileAt {
                    path: manifest.path().clone(),
                    with_parents: true,
                });
            }

            FsDataResource::Present {
                resource,
                state: resource_state,
            } => {
                let file_ptr = resource.upgrade().unwrap();
                let file = FsResource::File(file_ptr.clone());
                actions.extend([
                    Action::Remove(file.clone()),
                    Action::Rename {
                        resource: file.clone(),
                        to: utils::random_file_name(rng),
                    },
                    Action::Modify {
                        file: file_ptr.clone(),
                        kind: ModifyKind::Other,
                    },
                ]);

                let parent = fs_state.find_file_folder_by_ptr(&file_ptr).unwrap();
                if Ptr::ptr_eq(parent, folder) {
                    actions.extend([
                        Action::Move {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                        Action::Copy {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                    ]);
                }

                match resource_state {
                    DataResourceState::Invalid => actions.push(Action::Modify {
                        file: file_ptr,
                        kind: ModifyKind::Repair,
                    }),

                    DataResourceState::Valid => {
                        actions.extend([
                            Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::Corrupt,
                            },
                            Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::ManifestAdd(Alphanumeric.sample_string(rng, 16)),
                            },
                        ]);

                        let manifest_len = manifest.manifest().len();
                        if manifest_len > 0 {
                            let remove_index = rng.gen_range(0..manifest_len);
                            actions.push(Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::ManifestRemove(remove_index),
                            });
                        }
                    }
                }
            }
        }

        actions
    }

    /// # Arguments
    /// #. `manifest`: Project resource for which to get valid actions.
    /// #. `name`: File name for the resource.
    /// #. `parent`: Associated config folder.
    /// #. `folder`: Folder used for move and copy.
    /// #. `rng`: Random number generator.
    fn valid_actions_project_resource_manifest<M, R>(
        manifest: &Ptr<M>,
        name: impl Into<OsString>,
        parent: &Ptr<state::fs::Folder>,
        folder: &Ptr<state::fs::Folder>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        M: state::app::HasFsDataResource<Resource = state::fs::File> + state::app::Manifest,
        R: rand::Rng,
    {
        use crate::state::{
            action::{FsResource, ModifyKind},
            app::{DataResourceState, FsDataResource},
            Action,
        };

        let manifest = manifest.borrow();
        let mut actions = Vec::with_capacity(10);
        match manifest.fs_resource() {
            FsDataResource::NotPresent => {
                actions.push(Action::CreateFile {
                    parent: parent.clone(),
                    name: name.into(),
                });
            }

            FsDataResource::Present {
                resource,
                state: resource_state,
            } => {
                let file_ptr = resource.upgrade().unwrap();
                let file = FsResource::File(file_ptr.clone());
                actions.extend([
                    Action::Remove(file.clone()),
                    Action::Rename {
                        resource: file.clone(),
                        to: utils::random_file_name(rng),
                    },
                    Action::Modify {
                        file: file_ptr.clone(),
                        kind: ModifyKind::Other,
                    },
                ]);

                if !Ptr::ptr_eq(parent, folder) {
                    actions.extend([
                        Action::Move {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                        Action::Copy {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                    ]);
                }

                match resource_state {
                    DataResourceState::Invalid => actions.push(Action::Modify {
                        file: file_ptr,
                        kind: ModifyKind::Repair,
                    }),

                    DataResourceState::Valid => {
                        actions.extend([
                            Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::Corrupt,
                            },
                            Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::ManifestAdd(Alphanumeric.sample_string(rng, 16)),
                            },
                        ]);

                        let manifest_len = manifest.manifest().len();
                        if manifest_len > 0 {
                            let remove_index = rng.gen_range(0..manifest_len);
                            actions.push(Action::Modify {
                                file: file_ptr.clone(),
                                kind: ModifyKind::ManifestRemove(remove_index),
                            });
                        }
                    }
                }
            }
        }

        actions
    }

    /// # Arguments
    /// #. `resource`: Project config resource for which to get valid actions.
    /// #. `name`: File name for the resource.
    /// #. `parent`: Associated config folder.
    /// #. `folder`: Folder used for move and copy.
    /// #. `rng`: Random number generator.
    fn valid_actions_project_config_resource<M, R>(
        resource: &Ptr<M>,
        name: impl Into<OsString>,
        parent: &Ptr<state::fs::Folder>,
        folder: &Ptr<state::fs::Folder>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        M: state::app::HasFsDataResource<Resource = state::fs::File>,
        R: rand::Rng,
    {
        use crate::state::{
            action::{FsResource, ModifyKind},
            app::{DataResourceState, FsDataResource},
            Action,
        };

        let mut actions = Vec::with_capacity(10);
        match resource.borrow().fs_resource() {
            FsDataResource::NotPresent => actions.push(Action::CreateFile {
                parent: parent.clone(),
                name: name.into(),
            }),
            FsDataResource::Present { resource, state } => {
                let file_ptr = resource.upgrade().unwrap();
                let file = FsResource::File(file_ptr.clone());
                actions.extend([
                    Action::Remove(file.clone()),
                    Action::Rename {
                        resource: file.clone(),
                        to: utils::random_file_name(rng),
                    },
                    Action::Modify {
                        file: file_ptr.clone(),
                        kind: ModifyKind::Other,
                    },
                ]);

                if !Ptr::ptr_eq(parent, folder) {
                    actions.extend([
                        Action::Move {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                        Action::Copy {
                            resource: file.clone(),
                            parent: folder.clone(),
                        },
                    ]);
                }

                match state {
                    DataResourceState::Invalid => actions.push(Action::Modify {
                        file: file_ptr.clone(),
                        kind: ModifyKind::Repair,
                    }),
                    DataResourceState::Valid => actions.push(Action::Modify {
                        file: file_ptr.clone(),
                        kind: ModifyKind::Corrupt,
                    }),
                }
            }
        }

        actions
    }

    /// # Arguments
    /// #. `container`: Container for which to get valid actions.
    /// #. `fs_state`: File system state.
    /// #. `folders`: List of all folders in the file system.
    /// #. `rng`: Random number generator.
    fn valid_actions_container<R>(
        container: &Ptr<state::app::Container>,
        fs_state: &state::fs::State,
        folders: &Vec<Ptr<state::fs::Folder>>,
        rng: &mut R,
    ) -> Vec<state::Action>
    where
        R: rand::Rng,
    {
        use crate::state::{
            action,
            app::{DataResource, DataResourceState, Resource},
            Action,
        };
        use syre_local::constants;

        let mut actions = Vec::with_capacity(10);
        let container_ptr = container.borrow().fs_resource().upgrade().unwrap();
        let container_folder = action::FsResource::Folder(container_ptr.clone());
        let folder = &folders[rng.gen_range(0..folders.len())];

        actions.extend([
            Action::Remove(container_folder.clone()),
            Action::Rename {
                resource: container_folder.clone(),
                to: utils::random_file_name(rng),
            },
            Action::Move {
                resource: container_folder.clone(),
                parent: folder.clone(),
            },
        ]);

        match container.borrow().data() {
            Resource::NotPresent => {
                actions.push(Action::CreateFolder {
                    parent: container_ptr.clone(),
                    name: constants::APP_DIR.into(),
                });
            }

            Resource::Present(data) => {
                let data = data.borrow();
                let config = data.config().borrow();
                let config_folder_ptr = config.fs_resource().upgrade().unwrap().clone();
                actions.extend([
                    Action::CreateFolder {
                        parent: config_folder_ptr.clone(),
                        name: utils::random_file_name(rng).into(),
                    },
                    Action::CreateFile {
                        parent: config_folder_ptr.clone(),
                        name: utils::random_file_name(rng).into(),
                    },
                    Action::Remove(folder.clone().into()),
                    Action::Rename {
                        resource: config_folder_ptr.clone().into(),
                        to: utils::random_file_name(rng),
                    },
                ]);

                let folder = &folders[rng.gen_range(0..folders.len())];
                actions.extend(Self::valid_actions_project_config_resource(
                    config.properties(),
                    constants::PROJECT_FILE,
                    &config_folder_ptr,
                    &folder,
                    rng,
                ));

                let folder = &folders[rng.gen_range(0..folders.len())];
                actions.extend(Self::valid_actions_project_config_resource(
                    config.settings(),
                    constants::PROJECT_FILE,
                    &config_folder_ptr,
                    &folder,
                    rng,
                ));

                let folder = &folders[rng.gen_range(0..folders.len())];
                actions.extend(Self::valid_actions_project_resource_manifest(
                    config.assets(),
                    constants::ANALYSES_FILE,
                    &config_folder_ptr,
                    &folder,
                    rng,
                ));
            }
        }

        actions
    }
}

impl Simulator {
    fn perform_actions(&mut self, actions: Vec<state::Action>) -> Result {
        actions
            .iter()
            .map(|action| {
                let res = self.perform_action(action);
                self.state.app.reduce(action).unwrap();
                res
            })
            .collect()
    }

    fn perform_action(&mut self, action: &state::Action) -> Result {
        use crate::state::{action::FsResource, Action};

        tracing::debug!(?action);
        let fs_state = self.state.app.fs();
        match action {
            Action::CreateFolder { parent, name } => {
                assert!(!fs_state.name_exists(parent, name).unwrap());
                let path = fs_state.graph().path(parent).unwrap();
                let path = fs_state.base_path().join(path).join(name);
                fs::create_dir(path).unwrap();
            }
            Action::CreateFolderAt { path, with_parents } => {
                if *with_parents {
                    let path = fs_state.base_path().join(path);
                    fs::create_dir_all(path)?;
                } else {
                    let parent = fs_state
                        .graph()
                        .find_by_path(path.parent().unwrap())
                        .unwrap();

                    assert!(!fs_state
                        .name_exists(&parent, path.file_name().unwrap())
                        .unwrap());

                    let path = fs_state.base_path().join(path);
                    fs::create_dir(path)?;
                }
            }
            Action::CreateFile { parent, name } => {
                assert!(!fs_state.name_exists(parent, name).unwrap());
                let path = fs_state.graph().path(parent).unwrap();
                let path = fs_state.base_path().join(path).join(name);
                fs::File::create(path).unwrap();
            }
            Action::CreateFileAt { path, with_parents } => {
                let path = fs_state.base_path().join(path);
                if *with_parents {
                    fs::create_dir_all(path.parent().unwrap())?;
                }

                fs::File::create(path)?;
            }

            Action::Remove(resource) => {
                match resource {
                    FsResource::File(file) => {
                        let path = fs_state.file_path(file).unwrap();
                        let path = fs_state.base_path().join(path);
                        fs::remove_file(path)?;
                    }
                    FsResource::Folder(folder) => {
                        let path = fs_state.graph().path(folder).unwrap();
                        let path = fs_state.base_path().join(path);
                        fs::remove_dir_all(path)?;
                    }
                };
            }
            Action::Rename { resource, to } => {
                let from = match resource {
                    FsResource::File(file) => fs_state.file_path(file).unwrap(),
                    FsResource::Folder(folder) => fs_state.graph().path(folder).unwrap(),
                };

                let from = fs_state.base_path().join(from);
                let mut to_path = from.clone();
                to_path.set_file_name(to);
                fs::rename(from, to_path)?;
            }
            Action::Move { resource, parent } => {
                let from = match resource {
                    FsResource::File(file) => fs_state.file_path(file).unwrap(),
                    FsResource::Folder(folder) => fs_state.graph().path(folder).unwrap(),
                };
                assert!(!fs_state
                    .name_exists(parent, from.file_name().unwrap())
                    .unwrap());

                let to = fs_state.graph().path(parent).unwrap();
                let to = to.join(from.file_name().unwrap());
                assert!(!fs_state
                    .name_exists(parent, to.file_name().unwrap())
                    .unwrap());

                let from = fs_state.base_path().join(from);
                let to = fs_state.base_path().join(to);
                fs::rename(from, to)?;
            }
            Action::Copy { resource, parent } => {
                let from = match resource {
                    FsResource::File(file) => fs_state.file_path(file).unwrap(),
                    FsResource::Folder(folder) => fs_state.graph().path(folder).unwrap(),
                };
                assert!(!fs_state
                    .name_exists(parent, from.file_name().unwrap())
                    .unwrap());

                let to = fs_state.graph().path(parent).unwrap();
                let to = to.join(from.file_name().unwrap());
                assert!(!fs_state
                    .name_exists(parent, to.file_name().unwrap())
                    .unwrap());

                let from = fs_state.base_path().join(from);
                let to = fs_state.base_path().join(to);

                match resource {
                    FsResource::File(_) => {
                        fs::copy(from, to)?;
                    }
                    FsResource::Folder(_) => {
                        utils::copy_dir(from, to)?;
                    }
                }
            }
            Action::Modify { file, kind } => {
                let path = fs_state.file_path(file).unwrap();
                let path = fs_state.base_path().join(path);
                match kind {
                    state::action::ModifyKind::ManifestAdd(item) => {
                        fs::write(path, item)?;
                    }
                    state::action::ModifyKind::ManifestRemove(index) => {
                        fs::write(path, format!("remove {index}"))?;
                    }
                    state::action::ModifyKind::Corrupt => {
                        fs::write(path, "corrupt")?;
                    }
                    state::action::ModifyKind::Repair => {
                        fs::write(path, "repair")?;
                    }
                    state::action::ModifyKind::Other => {
                        fs::write(path, "modified")?;
                    }
                }
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
        let mut to_path = from.clone();
        to_path.set_file_name(to.as_ref());
        fs::rename(from, to_path)
    }

    fn move_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = self.options.base_path().join(to);
        fs::rename(from, to)
    }

    fn copy_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from = self.options.base_path().join(from);
        let to = self.options.base_path().join(to);
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
        let mut to_path = from.clone();
        to_path.set_file_name(to.as_ref());
        fs::rename(from, to_path)
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

pub struct State {
    current_tick: usize,
    pub app: state::State,
}

impl State {
    pub fn new(path: impl AsRef<Path>, app_config: &AppConfig) -> Self {
        Self {
            current_tick: 0,
            app: state::State::new(
                path,
                app_config.user_manifest().clone(),
                app_config.project_manifest().clone(),
            ),
        }
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
    ///
    /// # Arguments
    /// #. `base_path`: Path to calculate distances from.
    /// #. `paths`: Paths to choose from.
    pub fn random_move_path<R>(base_path: &PathBuf, paths: &Vec<PathBuf>, rng: &mut R) -> PathBuf
    where
        R: rand::Rng,
    {
        let (paths, distances): (Vec<_>, Vec<_>) = paths
            .iter()
            .filter_map(|path| {
                let distance = path_distance(base_path, path);
                if distance == 0 {
                    None
                } else {
                    Some((path, distance))
                }
            })
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
    /// + Assumes the paths are relative to the same root.
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
        /// Creates a new Option with seed `0`.
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
