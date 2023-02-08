use clap::{Parser, Subcommand};
use thot_cli::commands::{config, container, project, remote, run, user};

fn main() {
    let cli = Cli::parse();

    let res = match cli.command {
        // top level commands
        Command::New(args) => project::new::main(args, cli.verbose),
        Command::Init(args) => project::init::main(args, cli.verbose),
        Command::Move(args) => project::r#move::main(args, cli.verbose),
        Command::Run(args) => run::main(args, cli.verbose),

        // subcommands
        Command::Config(args) => config::main(args, cli.verbose),
        Command::User(args) => user::main(args, cli.verbose),
        Command::Container(args) => container::main(args, cli.verbose),
        Command::Remote(args) => remote::main(args, cli.verbose),
        Command::Project(args) => project::main(args, cli.verbose),
    };

    match res {
        Ok(()) => {}
        Err(err) => panic!("Something went wrong: {:?}", err),
    };
}

#[derive(Debug, Parser)]
#[clap(name = "Thot CLI")]
#[clap(version)]
#[clap(about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(short, long, global = true)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    // top level commands
    New(project::new::NewArgs),
    Init(project::init::InitArgs),
    Run(run::RunArgs),
    Move(project::r#move::MoveArgs),

    // subcommands
    Config(config::ConfigArgs),
    User(user::UserArgs),
    Remote(remote::RemoteArgs),
    Container(container::ContainerArgs),
    Project(project::ProjectArgs),
}
