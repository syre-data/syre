use clap::{Parser, Subcommand};
use std::io;
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
        Command::Pub => publ().unwrap(),
        Command::Sub => sub().unwrap(),
        Command::Req => req().unwrap(),
        Command::Rep => rep().unwrap(),
    }
}

fn publ() -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::PUB).unwrap();
    zmq_socket
        .bind(&thot_local_database::common::zmq_url(zmq::PUB).unwrap())
        .unwrap();

    let stdin = io::stdin();
    let mut message = String::new();
    loop {
        stdin.read_line(&mut message).unwrap();
        zmq_socket.send(&constants::PUB_SUB_TOPIC, zmq::SNDMORE)?;
        zmq_socket.send(&message, 0)?;
        message.clear();
    }
}

fn sub() -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::SUB).unwrap();
    zmq_socket.connect(&thot_local_database::common::zmq_url(zmq::SUB).unwrap())?;
    zmq_socket
        .set_subscribe(constants::PUB_SUB_TOPIC.as_bytes())
        .unwrap();

    loop {
        let messages = match zmq_socket.recv_multipart(0) {
            Ok(msg) => msg,
            Err(err) => {
                tracing::debug!(?err);
                continue;
            }
        };

        let messages = messages
            .into_iter()
            .map(|msg| zmq::Message::try_from(msg).unwrap())
            .collect::<Vec<_>>();

        let topic = messages.get(0).unwrap().as_str().unwrap();
        let mut message = String::new();
        for msg in messages.iter().skip(1) {
            message.push_str(msg.as_str().unwrap());
        }

        match serde_json::from_str::<thot_local_database::event::Update>(&message) {
            Ok(message) => println!(
                "{topic}\n{}\n",
                serde_json::to_string_pretty(&message).unwrap()
            ),
            Err(err) => println!("[could not decode: {err:?}]\n{topic}\n{message:?}\n"),
        }
    }
}

fn req() -> zmq::Result<()> {
    let zmq_context = zmq::Context::new();
    let zmq_socket = zmq_context.socket(zmq::REQ)?;
    zmq_socket.connect(&thot_local_database::common::zmq_url(zmq::REQ).unwrap())?;

    let stdin = io::stdin();
    let mut message = String::new();
    loop {
        stdin.read_line(&mut message).unwrap();
        zmq_socket.send(&message, 0)?;
        message.clear();
    }
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
    Pub,
    Sub,
    Req,
    Rep,
}
