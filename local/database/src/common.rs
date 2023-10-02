//! Common functions.
use crate::constants::{PUB_SUB_PORT, REQ_REP_PORT};
use std::net::Ipv4Addr;

static LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Returns the URL of the ZMQ socket of the given type.
///
/// # Returns
/// `None` if the given socket type is not supported.
pub fn zmq_url(kind: zmq::SocketType) -> Option<String> {
    let port = match kind {
        zmq::SocketType::REP => REQ_REP_PORT,
        zmq::SocketType::REQ => REQ_REP_PORT,
        zmq::SocketType::PUB => PUB_SUB_PORT,
        zmq::SocketType::SUB => PUB_SUB_PORT,
        _ => return None,
    };

    Some(format!("tcp://{LOCALHOST}:{REQ_REP_PORT}"))
}
