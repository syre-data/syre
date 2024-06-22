#![feature(io_error_more)]
#![feature(assert_matches)]
//! # Syre Local Database
//! Implements a local database for Syre.
pub mod event;
pub mod query;
pub mod state;

#[cfg(any(feature = "client", feature = "server"))]
pub mod constants;

#[cfg(any(feature = "client", feature = "server"))]
pub mod common;

#[cfg(any(feature = "client", feature = "server", feature = "error"))]
pub mod error;

#[cfg(any(feature = "client", feature = "server"))]
pub mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub use event::Update;
pub use query::Query;

#[cfg(any(feature = "client", feature = "server", feature = "error"))]
pub use error::{Error, Result};

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub use server::Database;
