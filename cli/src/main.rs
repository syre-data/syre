use clap::{Parser, Subcommand};
use thot_cli::commands::{check, config, container, project, run, user};

fn main() {
    let cli = Cli::parse();

    let res = match cli.command {
        // top level commands
        Command::New(args) => project::new::main(args, cli.verbose),
        Command::Init(args) => project::init_from::main(args, cli.verbose),
        Command::Move(args) => project::r#move::main(args, cli.verbose),
        Command::Run(args) => run::main(args, cli.verbose),
        Command::Check(args) => check::main(args, cli.verbose),

        // subcommands
        Command::Config(args) => config::main(args, cli.verbose),
        Command::User(args) => user::main(args, cli.verbose),
        Command::Container(args) => container::main(args, cli.verbose),
        Command::Project(args) => project::main(args, cli.verbose),
    };

    match res {
        Ok(()) => {}
        Err(err) => panic!("Something went wrong: {:?}", err),
    };
}

#[derive(Debug, Parser)]
#[clap(name = "Thot CLI")]
#[clap(version, about, long_about = None)]
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
    Init(project::init_from::InitFromArgs),
    Run(run::RunArgs),
    Move(project::r#move::MoveArgs),
    Check(check::CheckArgs),

    // subcommands
    Config(config::ConfigArgs),
    User(user::UserArgs),
    Container(container::ContainerArgs),
    Project(project::ProjectArgs),
}
