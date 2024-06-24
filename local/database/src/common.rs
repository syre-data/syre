//! Common functions.
use crate::constants::{PortNumber, DATASTORE_PORT, LOCALHOST, PUB_SUB_PORT, REQ_REP_PORT};
use std::path::{Path, PathBuf, StripPrefixError};

/// Returns a localhost tcp address with the given port.
pub fn localhost_with_port(port: PortNumber) -> String {
    format!("tcp://{LOCALHOST}:{port}")
}

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

    Some(localhost_with_port(port))
}

pub fn datastore_url() -> String {
    format!("{LOCALHOST}:{DATASTORE_PORT}")
}

/// Creates the absolute path from the data root to the container.
pub fn container_graph_path(
    data_root: impl AsRef<Path>,
    container: impl AsRef<Path>,
) -> Result<PathBuf, StripPrefixError> {
    let path = container.as_ref().strip_prefix(data_root.as_ref())?;
    Ok(Path::new("/").join(path))
}
