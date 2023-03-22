//! Client to interact with a [`Database`].
use crate::command::Command;
use crate::command::{Command as DbCommand, DatabaseCommand};
use crate::constants::{DATABASE_ID, REQ_REP_PORT};
use crate::types::PortNumber;
use serde_json::Value as JsValue;
use std::net::TcpListener;

#[cfg(target_os = "windows")]
static LOOPBACK_ADDR: &str = "localhost";

#[cfg(not(target_os = "windows"))]
static LOOPBACK_ADDR: &str = "0.0.0.0";

pub struct Client {
    zmq_context: zmq::Context,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&self, cmd: Command) -> JsValue {
        let req_socket = self
            .zmq_context
            .socket(zmq::SocketType::REQ)
            .expect("could not create `REQ` socket");

        req_socket
            .connect(&format!("tcp://{LOOPBACK_ADDR}:{REQ_REP_PORT}"))
            .expect("socket could not connect");

        req_socket
            .send(
                &serde_json::to_string(&cmd).expect("could not convert `Command` to JSON"),
                0,
            )
            .expect("socket could not send message");

        let mut msg = zmq::Message::new();
        req_socket
            .recv(&mut msg, 0)
            .expect("socket could not recieve `Message`");

        serde_json::from_str(
            msg.as_str()
                .expect("could not interpret `Message` as string"),
        )
        .expect("could not convert `Message` to JsValue")
    }

    /// Checks if a database is running.
    pub fn server_available() -> bool {
        // check if port is occupied
        // if dbg!(port_is_free(REQ_REP_PORT)) {
        //     // port is open, no chance of a listener
        //     return false;
        // }

        let ctx = zmq::Context::new();
        let req_socket = ctx
            .socket(zmq::SocketType::REQ)
            .expect("could not create socket");

        req_socket
            .connect(&format!("tcp://{LOOPBACK_ADDR}:{REQ_REP_PORT}"))
            .expect("socket could not connect");

        req_socket
            .send(
                &serde_json::to_string(&DbCommand::DatabaseCommand(DatabaseCommand::Id))
                    .expect("could not serialize `Command`"),
                0,
            )
            .expect("could not send `Id` command");

        let mut msg = zmq::Message::new();
        req_socket
            .recv(&mut msg, 0)
            .expect("could not recieve `Message`");

        let Some(id_str) = msg.as_str() else {
            panic!("invalid response");
        };

        let id_str: &str =
            serde_json::from_str(id_str).expect("could not convert `Message` to `String`");

        return id_str == DATABASE_ID;
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            zmq_context: zmq::Context::new(),
        }
    }
}

/// Checks if a given port on the loopback address is free.
fn port_is_free(port: PortNumber) -> bool {
    TcpListener::bind(format!("{LOOPBACK_ADDR}:{port}")).is_ok()
}

#[cfg(test)]
#[path = "./client_test.rs"]
mod client_test;
