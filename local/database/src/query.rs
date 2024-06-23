use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::types::UserId;

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
    UserManifest,
    ProjectManifest,
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
    Projects(UserId),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Project {}
