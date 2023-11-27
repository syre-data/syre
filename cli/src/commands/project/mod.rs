use crate::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
mod commands;

// **************************
// *** top level commands ***
// **************************
pub mod init;
pub mod r#move;
pub mod new;

// ********************
// *** sub commands ***
// ********************

pub fn main(args: ProjectArgs, verbose: bool) -> Result {
    match args.command {
        Command::AddScript(add_args) => commands::add_script(add_args, verbose),
    }
}

#[derive(Debug, Args)]
pub struct ProjectArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    AddScript(AddScriptArgs),
}

#[derive(Debug, Args)]
pub struct AddScriptArgs {
    path: PathBuf,

    #[clap(short, long)]
    project: Option<PathBuf>,
    // TODO Allow path type to be specified
    // #[clap(long, value_enum)]
    // path_type: Option<ResourcePathType>,
}
