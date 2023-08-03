//! Commands.
use super::{
    asset::AssetCommand, container::ContainerCommand, database::DatabaseCommand,
    graph::GraphCommand, project::ProjectCommand, script::ScriptCommand, user::UserCommand,
};
use serde::{Deserialize, Serialize};

/// Commands that can be issued to the [`Database`](super::Database).
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    AssetCommand(AssetCommand),
    ContainerCommand(ContainerCommand),
    ProjectCommand(ProjectCommand),
    GraphCommand(GraphCommand),
    DatabaseCommand(DatabaseCommand),
    ScriptCommand(ScriptCommand),
    UserCommand(UserCommand),
}

impl From<AssetCommand> for Command {
    fn from(cmd: AssetCommand) -> Self {
        Self::AssetCommand(cmd)
    }
}

impl From<ContainerCommand> for Command {
    fn from(cmd: ContainerCommand) -> Self {
        Self::ContainerCommand(cmd)
    }
}

impl From<ProjectCommand> for Command {
    fn from(cmd: ProjectCommand) -> Self {
        Self::ProjectCommand(cmd)
    }
}

impl From<GraphCommand> for Command {
    fn from(cmd: GraphCommand) -> Self {
        Self::GraphCommand(cmd)
    }
}

impl From<DatabaseCommand> for Command {
    fn from(cmd: DatabaseCommand) -> Self {
        Self::DatabaseCommand(cmd)
    }
}

impl From<ScriptCommand> for Command {
    fn from(cmd: ScriptCommand) -> Self {
        Self::ScriptCommand(cmd)
    }
}

impl From<UserCommand> for Command {
    fn from(cmd: UserCommand) -> Self {
        Self::UserCommand(cmd)
    }
}
