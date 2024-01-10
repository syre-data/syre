//! Types.
use has_id::HasId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Convenience type to indicate valid port numbers.
pub type PortNumber = u16;

/// Socket types used for hash keys.
///
/// # Note
/// `zmq::SocketType` is not `hash`able, making this enum required.
/// This has been raised as an issue on the [`zmq GitHub`](https://github.com/erickt/rust-zmq/issues/362).
/// If this faeture is implemented, this enum can be depricated.
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Debug)]
pub enum SocketType {
    /// [`zmq::SocketType::REP`]
    REP,
}

pub struct PartialLoadGraph<T: HasId<Id = thot_core::types::ResourceId>> {
    errors: HashMap<PathBuf, thot_local::loader::error::container::Error>,
    graph: Option<thot_core::graph::ResourceTree<T>>,
}
