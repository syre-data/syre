//! Package constants.
pub use crate::types::PortNumber;
use std::net::Ipv4Addr;

pub const LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Local port for REP-REQ communication.
pub const REQ_REP_PORT: PortNumber = 7047;

/// Local port for PUB-SUB communication.
pub const PUB_SUB_PORT: PortNumber = 7048;

/// Local port for PUB-SUB communication.
pub const DATASTORE_PORT: PortNumber = 7049;

/// PUB-SUB topic
pub const PUB_SUB_TOPIC: &str = "syre://local-database";

/// Identifier string for the database
pub const DATABASE_ID: &str = "syre local database";
