//! Client to interact with a [`Database`].
use crate::command::Command;
use crate::command::{Command as DbCommand, DatabaseCommand};
use crate::constants::{DATABASE_ID, REQ_REP_PORT};
use crate::types::PortNumber;
use serde_json::Value as JsValue;
use std::net::{Ipv4Addr, TcpListener};

static LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

pub struct Client {
    zmq_context: zmq::Context,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    #[tracing::instrument(skip(self))]
    pub fn send(&self, cmd: Command) -> JsValue {
        // TODO: May be able to move creation of `req_socket` to `#new`, but may run into `Sync` issues.
        let req_socket = self
            .zmq_context
            .socket(zmq::SocketType::REQ)
            .expect("could not create `REQ` socket");

        req_socket
            .set_connect_timeout(1000)
            .expect("could not set connection timeout");

        req_socket
            .set_rcvtimeo(5_000)
            .expect("could not set socket timeout");

        req_socket
            .connect(&format!("tcp://{LOCALHOST}:{REQ_REP_PORT}"))
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
        if port_is_free(REQ_REP_PORT) {
            // port is open, no chance of a listener
            return false;
        }

        let ctx = zmq::Context::new();
        let req_socket = ctx
            .socket(zmq::SocketType::REQ)
            .expect("could not create socket");

        req_socket
            .set_connect_timeout(1000)
            .expect("could not set connection timeout");

        req_socket
            .set_rcvtimeo(1000)
            .expect("could not set socket timeout");

        req_socket
            .connect(&format!("tcp://{LOCALHOST}:{REQ_REP_PORT}"))
            .expect("socket could not connect");

        req_socket
            .send(
                &serde_json::to_string(&DbCommand::DatabaseCommand(DatabaseCommand::Id))
                    .expect("could not serialize `Command`"),
                0,
            )
            .expect("could not send `Id` command");

        let mut msg = zmq::Message::new();
        let res = req_socket.recv(&mut msg, 0);
        if res.is_err() {
            // @todo: Check error type for timeout.
            return false;
        }

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
    TcpListener::bind(format!("{LOCALHOST}:{port}")).is_ok()
}

#[cfg(test)]
#[path = "./client_test.rs"]
mod client_test;
