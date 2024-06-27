//! Events for the desktop.
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use syre_core::system::User;
use syre_local_database as db;
use uuid::Uuid;

pub use topic::topic;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    id: Uuid,

    /// Id of the parent event.
    parent: Uuid,
    kind: EventKind,
}

impl Event {
    pub fn new(kind: EventKind, parent: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn parent(&self) -> &Uuid {
        &self.parent
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
pub enum EventKind {
    User(Option<User>),

    #[from(ignore)]
    App(App),
    ProjectManifest(ProjectManifest),
}

impl<T> From<T> for EventKind
where
    T: Into<App>,
{
    fn from(value: T) -> Self {
        Self::App(value.into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
pub enum App {
    UserManifest(UserManifest),
    LocalConfig(LocalConfig),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserManifest {
    Corrupted,
    Repaired,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProjectManifest {
    Added(Vec<(PathBuf, db::state::ProjectData)>),
    Removed(Vec<PathBuf>),
    Corrupted,
    Repaired,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocalConfig {
    Corrupted,
    Repaired,
}

pub mod topic {
    pub const PREFIX: &str = "syre";
    pub const USER: &str = "syre:user";
    pub const PROJECT_MANIFEST: &str = "syre:project_manifest";

    pub fn topic(topic: impl AsRef<str>) -> String {
        format!("{}:{}", PREFIX, topic.as_ref())
    }
}
