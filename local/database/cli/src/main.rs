use clap::{Args, Parser, Subcommand};
use thot_local_database::constants;

#[derive(Parser, Debug)]
#[clap(name = "Thot Local Database CLI", author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Pub(SendArgs { message }) => publ(message).unwrap(),
        Command::Sub => sub().unwrap(),
        Command::Req(SendArgs { message }) => req(&message).unwrap(),
        Command::Rep => rep().unwrap(),
    }
}

fn publ(message: impl std::fmt::Display) -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::PUB).unwrap();
    zmq_socket
        .bind(&thot_local_database::common::zmq_url(zmq::PUB).unwrap())
        .unwrap();

    zmq_socket.send(&format!("{} {}", constants::PUB_SUB_TOPIC, message), 0)
}

fn sub() -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::SUB).unwrap();
    zmq_socket.connect(&thot_local_database::common::zmq_url(zmq::REQ).unwrap())?;
    zmq_socket
        .set_subscribe(constants::PUB_SUB_TOPIC.as_bytes())
        .unwrap();

    loop {
        let msg = zmq_socket.recv_msg(0)?;
        println!("{:?}", msg.as_str().unwrap());
    }
}

fn req(message: impl zmq::Sendable) -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::REQ)?;
    zmq_socket.connect(&thot_local_database::common::zmq_url(zmq::REQ).unwrap())?;

    zmq_socket.send(message, 0)
}

fn rep() -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::REP).unwrap();
    zmq_socket.bind(&thot_local_database::common::zmq_url(zmq::REP).unwrap())?;

    loop {
        let msg = zmq_socket.recv_msg(0)?;
        println!("{:?}", msg.as_str().unwrap());
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Pub(SendArgs),
    Sub,
    Req(SendArgs),
    Rep,
}

#[derive(Debug, Args)]
struct SendArgs {
    message: String,
}
