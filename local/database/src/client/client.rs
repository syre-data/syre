//! Client to interact with a [`Database`].
use crate::command::{Command, DatabaseCommand};
use crate::common;
use crate::constants::{DATABASE_ID, REQ_REP_PORT};
use crate::types::PortNumber;
use crate::Result;
use serde_json::Value as JsValue;
use std::net::{Ipv4Addr, TcpListener};

static LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
static CONNECT_TIMEOUT: i32 = 5000;
static RECV_TIMEOUT: i32 = 5000;

pub struct Client {
    zmq_context: zmq::Context,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    #[tracing::instrument(skip(self))]
    pub fn send(&self, cmd: Command) -> Result<JsValue> {
        // TODO: May be able to move creation of `req_socket` to `#new`, but may run into `Sync` issues.
        let req_socket = self
            .zmq_context
            .socket(zmq::REQ)
            .expect("could not create `REQ` socket");

        req_socket
            .set_connect_timeout(CONNECT_TIMEOUT)
            .expect("could not set connection timeout");

        req_socket
            .set_rcvtimeo(RECV_TIMEOUT)
            .expect("could not set socket timeout");

        req_socket
            .connect(&common::zmq_url(zmq::REQ).unwrap())
            .expect("socket could not connect");

        req_socket
            .send(
                &serde_json::to_string(&cmd).expect("could not convert `Command` to JSON"),
                0,
            )
            .expect("socket could not send message");

        let mut msg = zmq::Message::new();
        req_socket.recv(&mut msg, 0)?;

        Ok(serde_json::from_str(
            msg.as_str()
                .expect("could not interpret `Message` as string"),
        )
        .expect("could not convert `Message` to JsValue"))
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
            .set_connect_timeout(CONNECT_TIMEOUT)
            .expect("could not set connection timeout");

        req_socket
            .set_rcvtimeo(RECV_TIMEOUT)
            .expect("could not set socket timeout");

        req_socket
            .connect(&common::zmq_url(zmq::REQ).unwrap())
            .expect("socket could not connect");

        req_socket
            .send(
                &serde_json::to_string(&Command::DatabaseCommand(DatabaseCommand::Id))
                    .expect("could not serialize `Command`"),
                0,
            )
            .expect("could not send `Id` command");

        let mut msg = zmq::Message::new();
        let res = req_socket.recv(&mut msg, 0);
        if res.is_err() {
            // TODO Check error type for timeout.
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
