use serde::{Deserialize, Serialize};
use syre_core::types::UserId;

#[derive(Serialize, Deserialize, Debug, derive_more::From)]
pub enum Query {
    User(User),
    Project(Project),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum User {
    Projects(UserId),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Project {}
