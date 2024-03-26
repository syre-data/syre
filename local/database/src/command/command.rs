//! Commands.
use super::{
    analysis::AnalysisCommand, asset::AssetCommand, container::ContainerCommand,
    database::DatabaseCommand, graph::GraphCommand, project::ProjectCommand, runner::RunnerCommand,
    search::SearchCommand, user::UserCommand,
};
use serde::{Deserialize, Serialize};

/// Commands that can be issued to the [`Database`](super::Database).
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Asset(AssetCommand),
    Container(ContainerCommand),
    Project(ProjectCommand),
    Graph(GraphCommand),
    Database(DatabaseCommand),
    Analysis(AnalysisCommand),
    User(UserCommand),
    Runner(RunnerCommand),
    Search(SearchCommand),
}

impl From<AssetCommand> for Command {
    fn from(cmd: AssetCommand) -> Self {
        Self::Asset(cmd)
    }
}

impl From<ContainerCommand> for Command {
    fn from(cmd: ContainerCommand) -> Self {
        Self::Container(cmd)
    }
}

impl From<ProjectCommand> for Command {
    fn from(cmd: ProjectCommand) -> Self {
        Self::Project(cmd)
    }
}

impl From<GraphCommand> for Command {
    fn from(cmd: GraphCommand) -> Self {
        Self::Graph(cmd)
    }
}

impl From<DatabaseCommand> for Command {
    fn from(cmd: DatabaseCommand) -> Self {
        Self::Database(cmd)
    }
}

impl From<AnalysisCommand> for Command {
    fn from(cmd: AnalysisCommand) -> Self {
        Self::Analysis(cmd)
    }
}

impl From<UserCommand> for Command {
    fn from(cmd: UserCommand) -> Self {
        Self::User(cmd)
    }
}

impl From<RunnerCommand> for Command {
    fn from(cmd: RunnerCommand) -> Self {
        Self::Runner(cmd)
    }
}

impl From<SearchCommand> for Command {
    fn from(cmd: SearchCommand) -> Self {
        Self::Search(cmd)
    }
}
