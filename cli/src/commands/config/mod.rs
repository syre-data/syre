use crate::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use thot_core::types::UserId;
mod commands;

/// Configures a Thot project.
pub fn main(args: ConfigArgs, verbose: bool) -> Result {
    // run command
    match args.command {
        Command::SetUser(user) => commands::set_active_user(&user),
    }
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    command: Command,
    root: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    SetUser(UserId),
}