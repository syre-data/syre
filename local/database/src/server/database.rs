//! Database for storing resources.
#[path = "./handler/mod.rs"]
mod handler;

use super::store::Datastore;
use crate::command::Command;
use crate::constants::{DATABASE_ID, REQ_REP_PORT};
use crate::Result;
use serde_json::Value as JsValue;

/// Database.
pub struct Database {
    store: Datastore,

    /// Kill flag indicating the database should stop listening and return.
    kill: bool,

    /// ZMQ context.
    zmq_context: zmq::Context,
}

impl Database {
    pub fn new() -> Self {
        Database {
            store: Datastore::new(),
            kill: false,
            zmq_context: zmq::Context::new(),
        }
    }

    pub fn listen_for_commands(&mut self) -> Result {
        let rep_socket = self.zmq_context.socket(zmq::SocketType::REP)?;
        rep_socket.bind(&format!("tcp://*:{REQ_REP_PORT}"))?;

        loop {
            if self.kill {
                break;
            }

            let mut msg = zmq::Message::new();
            rep_socket
                .recv(&mut msg, 0)
                .expect("could not recieve request");

            let msg_str = msg.as_str().expect("could not convert `Message` to srting");
            let cmd: Command =
                serde_json::from_str(msg_str).expect("could not convert `Message` to JSON");

            let res = self.handle_command(cmd);
            rep_socket
                .send(&res.to_string(), 0)
                .expect("could not send response");
        }

        Ok(())
    }

    // @todo: Handle errors.
    /// Handles a given command, returning the correct data.
    pub fn handle_command(&mut self, command: Command) -> JsValue {
        match command {
            Command::AssetCommand(cmd) => self.handle_command_asset(cmd),
            Command::ContainerCommand(cmd) => self.handle_command_container(cmd),
            Command::DatabaseCommand(cmd) => self.handle_command_database(cmd),
            Command::ProjectCommand(cmd) => self.handle_command_project(cmd),
            Command::GraphCommand(cmd) => self.handle_command_graph(cmd),
            Command::ScriptCommand(cmd) => self.handle_command_script(cmd),
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
