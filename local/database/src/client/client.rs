//! Client to interact with a [`Database`].
use crate::{
    common,
    constants::{DATABASE_ID, LOCALHOST, REQ_REP_PORT},
    types::PortNumber,
    Query,
};
use serde_json::Value as JsValue;
use std::net::TcpListener;

static CONNECT_TIMEOUT: i32 = 5000;
static RECV_TIMEOUT: i32 = 5000;

pub type CmdResult<T, E> = zmq::Result<Result<T, E>>;

/// Checks if a given port on the loopback address is free.
fn port_is_free(port: PortNumber) -> bool {
    TcpListener::bind(format!("{LOCALHOST}:{port}")).is_ok()
}

pub struct Client {
    zmq_context: zmq::Context,
}

impl Client {
    pub fn new() -> Self {
        let ctx = zmq::Context::new();
        Self {
            zmq_context: ctx.clone(),
        }
    }

    pub fn send(&self, query: Query) -> zmq::Result<JsValue> {
        // TODO: May be able to move creation of `socket` to `#new`, but may run into `Sync` issues.
        let socket = Self::socket(&self.zmq_context);
        socket.send(&serde_json::to_string(&query).unwrap(), 0)?;

        let mut msg = zmq::Message::new();
        socket.recv(&mut msg, 0)?;

        Ok(serde_json::from_str(msg.as_str().unwrap()).unwrap())
    }

    fn socket(ctx: &zmq::Context) -> zmq::Socket {
        const SOCKET_KIND: zmq::SocketType = zmq::REQ;
        let socket = ctx.socket(SOCKET_KIND).unwrap();
        socket.set_connect_timeout(CONNECT_TIMEOUT).unwrap();
        socket.set_rcvtimeo(RECV_TIMEOUT).unwrap();
        socket
            .connect(&common::zmq_url(SOCKET_KIND).unwrap())
            .unwrap();

        socket
    }
}
