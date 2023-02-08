//! Commands.
use super::{
    asset::AssetCommand, container::ContainerCommand, database::DatabaseCommand,
    project::ProjectCommand, script::ScriptCommand,
};
use serde::{Deserialize, Serialize};

/// Commands that can be issued to the [`Database`](super::Database).
#[derive(Serialize, Deserialize)]
pub enum Command {
    AssetCommand(AssetCommand),
    ContainerCommand(ContainerCommand),
    ProjectCommand(ProjectCommand),
    DatabaseCommand(DatabaseCommand),
    ScriptCommand(ScriptCommand),
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

#[cfg(test)]
#[path = "./command_test.rs"]
mod command_test;
