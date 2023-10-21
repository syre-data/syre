use crate::types::ResourcePathType;
use crate::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
mod commands;

pub fn main(args: ContainerArgs, verbose: bool) -> Result {
    match args.command {
        Command::Init(init_args) => commands::init(init_args, verbose),
        Command::New(new_args) => commands::new(new_args, verbose),
        Command::AssociateScript(add_args) => commands::associate_script(add_args, verbose),
    }
}

#[derive(Debug, Args)]
pub struct ContainerArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init(InitArgs),
    New(NewArgs),
    AssociateScript(AssociateScriptArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[clap(short, long, parse(from_os_str))]
    path: Option<PathBuf>,

    /// Do not add files as Assets.
    #[clap(long)]
    no_assets: bool,

    /// Do not recurse.
    #[clap(long)]
    no_recurse: bool,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    #[clap(parse(from_os_str))]
    path: PathBuf,
}

#[derive(Debug, Args)]
pub struct AssociateScriptArgs {
    #[clap(parse(from_os_str))]
    path: PathBuf,

    #[clap(short, long, parse(from_os_str))]
    container: Option<PathBuf>,

    #[clap(short, long)]
    priority: Option<i32>,

    #[clap(long)]
    autorun: Option<bool>,

    // force register parameters
    #[clap(long)]
    register: bool,

    // TODO Only valid if register is true
    #[clap(long, value_enum)]
    path_type: Option<ResourcePathType>,
}
