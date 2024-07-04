use crate::{constants, query, server, state, Database};
use serde_json::Value as JsValue;
use std::path::{Path, PathBuf};
use syre_core::{system::User, types::ResourceId};

impl Database {
    pub fn handle_query_config(&self, query: query::Config) -> JsValue {
        match query {
            query::Config::Id => constants::DATABASE_ID.into(),
        }
    }
}

impl Database {
    pub fn handle_query_state(&self, query: query::State) -> JsValue {
        match query {
            query::State::UserManifest => {
                serde_json::to_value(self.state.app().user_manifest()).unwrap()
            }
            query::State::ProjectManifest => {
                serde_json::to_value(self.state.app().project_manifest()).unwrap()
            }
            query::State::LocalConfig => {
                serde_json::to_value(self.state.app().local_config()).unwrap()
            }
            query::State::Projects => {
                let states = self.handle_query_state_projects();
                serde_json::to_value(states).unwrap()
            }
            query::State::Graph(base_path) => {
                let state = self.handle_query_state_graph(base_path);
                serde_json::to_value(state).unwrap()
            }
            query::State::Asset {
                project,
                container,
                asset,
            } => {
                let state = self.handle_query_state_asset(project, container, asset);
                serde_json::to_value(state).unwrap()
            }
        }
    }

    fn handle_query_state_projects(&self) -> Vec<state::Project> {
        self.state
            .projects()
            .iter()
            .map(|project| {
                let data = project.fs_resource().map(|data| data.project_data());
                state::Project {
                    path: project.path().clone(),
                    fs_resource: data,
                }
            })
            .collect()
    }

    fn handle_query_state_graph(&self, base_path: PathBuf) -> Option<state::Graph> {
        let Some(project) = self.state.find_project_by_path(&base_path) else {
            return None;
        };

        let state::FolderResource::Present(project) = project.fs_resource() else {
            return None;
        };

        let state::FolderResource::Present(graph) = project.graph() else {
            return None;
        };

        Some(graph.as_graph())
    }

    // NB: Asset is copied for code cleanliness.
    //      If this becomes a performance issue, this can be changed.
    /// # Arguments
    /// 1. `project`: Path to the project's base folder.
    /// 2. `container`: Absolute path to the container from the graph root.
    /// 3. `asset`: Relative path to the asset file from the container.
    fn handle_query_state_asset(
        &self,
        project: PathBuf,
        container: PathBuf,
        asset: PathBuf,
    ) -> Option<state::Asset> {
        let Some(project) = self.state.find_project_by_path(&project) else {
            return None;
        };

        let state::FolderResource::Present(project) = project.fs_resource() else {
            return None;
        };

        let state::FolderResource::Present(graph) = project.graph() else {
            return None;
        };

        let Some(container) = graph.find(container) else {
            return None;
        };

        let container = container.lock().unwrap();
        let state::DataResource::Ok(ref assets) = container.assets else {
            return None;
        };

        assets.iter().find(|state| state.path == asset).cloned()
    }
}

impl Database {
    pub fn handle_query_user(&self, query: query::User) -> JsValue {
        match query {
            query::User::Info(id) => {
                let state::ManifestState::Ok(ref manifest) = self.state.app().user_manifest()
                else {
                    return serde_json::to_value(Option::<User>::None).unwrap();
                };

                let user = manifest.iter().find(|user| user.rid() == &id);
                serde_json::to_value(user).unwrap()
            }
            query::User::Projects(user) => serde_json::to_value(self.user_projects(&user)).unwrap(),
        }
    }

    fn user_projects(&self, user: &ResourceId) -> Vec<(PathBuf, state::ProjectData)> {
        self.state
            .projects()
            .iter()
            .filter_map(|project| {
                let state::FolderResource::Present(project_state) = project.fs_resource() else {
                    return None;
                };

                let state::DataResource::Ok(settings) = project_state.settings() else {
                    return None;
                };

                let Some(permissions) =
                    settings.permissions.iter().find_map(|(uid, permissions)| {
                        if uid == user {
                            Some(permissions)
                        } else {
                            None
                        }
                    })
                else {
                    return None;
                };

                if permissions.any() {
                    Some((project.path().clone(), project_state.project_data()))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Database {
    pub fn handle_query_project(&self, query: query::Project) -> JsValue {
        match query {
            query::Project::Get(project) => {
                let state = self.handle_query_project_get(&project);
                serde_json::to_value(state).unwrap()
            }
            query::Project::GetById(project) => {
                let state = self.handle_query_project_get_by_id(&project);
                serde_json::to_value(state).unwrap()
            }
            query::Project::GetMany(projects) => {
                let states = self.handle_query_project_get_many(&projects);
                serde_json::to_value(states).unwrap()
            }
            query::Project::Resources(project) => {
                let state = self.handle_query_state_project_resources(&project);
                serde_json::to_value(state).unwrap()
            }
        }
    }

    fn handle_query_project_get(&self, project: impl AsRef<Path>) -> Option<state::Project> {
        self.state.projects().iter().find_map(|state| {
            if state.path() == project.as_ref() {
                Some(state::Project {
                    path: state.path().clone(),
                    fs_resource: state.fs_resource().map(|data| data.project_data()),
                })
            } else {
                None
            }
        })
    }

    /// # Returns
    /// Project with the given id.
    /// `None` if a state is not associated with the project.
    fn handle_query_project_get_by_id(
        &self,
        project: &ResourceId,
    ) -> Option<(PathBuf, state::ProjectData)> {
        self.state.projects().iter().find_map(|state| {
            let state::FolderResource::Present(project_state) = state.fs_resource() else {
                return None;
            };

            let state::DataResource::Ok(properties) = project_state.properties() else {
                return None;
            };

            if properties.rid() == project {
                Some((state.path().clone(), project_state.project_data()))
            } else {
                None
            }
        })
    }

    /// # Returns
    /// State of the projects at the given paths.
    /// Paths without an associated state are ommitted from the result.
    fn handle_query_project_get_many(&self, projects: &Vec<PathBuf>) -> Vec<state::Project> {
        projects
            .iter()
            .filter_map(|project| {
                self.state.projects().iter().find_map(|state| {
                    if state.path() == project {
                        Some(state::Project {
                            path: state.path().clone(),
                            fs_resource: state.fs_resource().map(|data| data.project_data()),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// # Returns
    /// Project's data and graph.
    /// `None` if a state is not associated with the project.
    fn handle_query_state_project_resources(
        &self,
        project: &ResourceId,
    ) -> Option<(state::ProjectData, state::FolderResource<state::Graph>)> {
        self.state.projects().iter().find_map(|state| {
            let state::FolderResource::Present(state) = state.fs_resource() else {
                return None;
            };

            let state::DataResource::Ok(properties) = state.properties() else {
                return None;
            };

            if properties.rid() == project {
                Some((
                    state.project_data(),
                    state.graph().map(|graph| graph.as_graph()),
                ))
            } else {
                None
            }
        })
    }
}

impl Database {
    pub fn handle_query_container(&self, query: query::Container) -> JsValue {
        match query {
            query::Container::Get { project, container } => {
                let state = self
                    .handle_query_container_get(&project, container)
                    .map(|state| {
                        let container = state.lock().unwrap();
                        (*container).clone()
                    });

                serde_json::to_value(state).unwrap()
            }
        }
    }

    /// # Arguments
    /// 1. `project`: Path to the project's base folder.
    /// 2. `container`: Absolute path to the container from the graph root.
    /// The root container has the root path.
    fn handle_query_container_get(
        &self,
        project: &ResourceId,
        container: PathBuf,
    ) -> Option<&server::state::project::graph::Node> {
        let Some(project) = self.state.find_project_by_id(&project) else {
            return None;
        };

        let state::FolderResource::Present(project) = project.fs_resource() else {
            return None;
        };

        let state::FolderResource::Present(graph) = project.graph() else {
            return None;
        };

        graph.find(container)
    }
}
