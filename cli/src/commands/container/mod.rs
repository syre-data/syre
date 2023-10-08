use crate::result::Result;
use crate::types::ResourcePathType;
use clap::{Args, Subcommand};
use std::path::PathBuf;
mod commands;

pub fn main(args: ContainerArgs, verbose: bool) -> Result {
    match args.command {
        Command::Init(init_args) => commands::init(init_args, verbose),
        Command::New(new_args) => commands::new(new_args, verbose),
        Command::AddChild(add_args) => commands::add_child(add_args, verbose),
        Command::NewChild(new_args) => commands::new_child(new_args, verbose),
        Command::AddAsset(add_args) => commands::add_asset(add_args, verbose),
        Command::AddScript(add_args) => commands::add_script(add_args, verbose),
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
    AddChild(AddChildArgs),
    NewChild(NewChildArgs),
    AddAsset(AddAssetArgs),
    AddScript(AddScriptArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[clap(short, long, parse(from_os_str))]
    path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct NewArgs {
    #[clap(parse(from_os_str))]
    name: PathBuf,
}

#[derive(Debug, Args)]
pub struct AddChildArgs {
    #[clap(parse(from_os_str))]
    path: PathBuf,

    #[clap(short, long, parse(from_os_str))]
    parent: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct NewChildArgs {
    #[clap(parse(from_os_str))]
    path: PathBuf,

    #[clap(short, long, parse(from_os_str))]
    parent: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct AddAssetArgs {
    #[clap(parse(from_os_str))]
    path: PathBuf,

    #[clap(short, long, parse(from_os_str))]
    parent: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct AddScriptArgs {
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
