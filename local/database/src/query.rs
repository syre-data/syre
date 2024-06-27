use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::types::ResourceId;

#[derive(Serialize, Deserialize, Debug, derive_more::From)]
pub enum Query {
    Config(Config),
    State(State),
    User(User),
    Project(Project),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Config {
    Id,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    /// Retrieve the state of the user manifest.
    UserManifest,

    /// Retrieve the state of the project manifest.
    ProjectManifest,

    /// Retrieve the state of the local config.
    LocalConfig,

    /// Retrieve the state of all projects.
    Projects,

    /// Retrieve the entire graph of a project.
    Graph(
        /// Base path of the project.
        PathBuf,
    ),

    /// Retrieve the state of a container.
    Container {
        /// Base path of the project.
        project: PathBuf,

        /// Relative path to the container from the data root.
        container: PathBuf,
    },

    /// Retrieve the state of a container.
    Asset {
        /// Base path of the project.
        project: PathBuf,

        /// Relative path to the container from the data root.
        container: PathBuf,

        /// Relative path to the asset from the container.
        asset: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum User {
    /// Return info on the user.
    Info(ResourceId),

    /// Get all the user's projects.
    Projects(ResourceId),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Project {
    /// Retrieve the state of the project at the given path.
    Get(PathBuf),

    /// Retrieve the state of the projects at the given paths.
    ///
    /// # Notes
    /// If a path is not associated with a state, it is excluded from the
    /// result. It is up to the client application to diff the request and response.
    GetMany(Vec<PathBuf>),

    /// Retrieve the project's data graph.
    Graph(ResourceId),
}
