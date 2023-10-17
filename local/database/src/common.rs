//! Common functions.
use crate::constants::{PUB_SUB_PORT, REQ_REP_PORT};
use std::net::Ipv4Addr;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use thot_local::constants::WINDOWS_UNC_PREFIX;

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

    Some(format!("tcp://{LOCALHOST}:{port}"))
}

/// Prefixes the path with the [Windows UNC](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats#unc-paths) path if it is not already there.
#[cfg(target_os = "windows")]
pub fn ensure_windows_unc(path: impl Into<PathBuf>) -> PathBuf {
    let path: PathBuf = path.into();
    if path.starts_with(WINDOWS_UNC_PREFIX) {
        path
    } else {
        // Must prefix UNC path as `str` because using `Path`s strips it.
        let mut p = WINDOWS_UNC_PREFIX.to_string();
        p.push_str(path.to_str().unwrap());
        PathBuf::from(p)
    }
}
