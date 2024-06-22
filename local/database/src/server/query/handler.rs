use crate::{constants, query, server, state, Database};
use serde_json::Value as JsValue;
use std::path::PathBuf;

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
            query::State::Projects => {
                let states = self.handle_query_state_projects();
                serde_json::to_value(states).unwrap()
            }
            query::State::Graph(base_path) => {
                let state = self.handle_query_state_graph(base_path);
                serde_json::to_value(state).unwrap()
            }
            query::State::Container { project, container } => {
                let state = self
                    .handle_query_state_container(project, container)
                    .map(|state| {
                        let container = state.lock().unwrap();
                        (*container).clone()
                    });

                serde_json::to_value(state).unwrap()
            }
        }
    }

    fn handle_query_state_projects(&self) -> Vec<state::Project> {
        self.state
            .projects()
            .iter()
            .map(|project| {
                let data = project.fs_resource().map(|data| state::ProjectData {
                    properties: data.properties().cloned(),
                    settings: data.settings().cloned(),
                    analyses: data.analyses().cloned(),
                });

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

    /// # Arguments
    /// 1. `project`: Path to the project's base folder.
    /// 2. `container`: Absolute path to the container from the graph root.
    /// The root container has the root path.
    fn handle_query_state_container(
        &self,
        project: PathBuf,
        container: PathBuf,
    ) -> Option<&server::state::project::graph::Node> {
        let Some(project) = self.state.find_project_by_path(&project) else {
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

impl Database {
    pub fn handle_query_user(&self, query: query::User) -> JsValue {
        todo!();
    }
}

impl Database {
    pub fn handle_query_project(&self, query: query::Project) -> JsValue {
        todo!();
    }
}
