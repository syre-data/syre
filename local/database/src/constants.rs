//! Package constants.
pub use crate::types::PortNumber;

/// Local port for REP-REQ communication.
pub const REQ_REP_PORT: PortNumber = 7047;

/// Local port for PUB-SUB communication.
pub const PUB_SUB_PORT: PortNumber = 4749;

/// Identifier string for the database
pub const DATABASE_ID: &str = "thot local database";
