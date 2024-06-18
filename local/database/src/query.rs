use serde::{Deserialize, Serialize};
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum User {
    Projects(UserId),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Project {}
