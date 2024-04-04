#![feature(io_error_more)]
//! # Syre Local Database
//! Implements a local database for Syre.
pub mod command;
pub mod event;

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

// Re-exports
pub use command::{
    AnalysisCommand, AssetCommand, Command, ContainerCommand, DatabaseCommand, GraphCommand,
    ProjectCommand,
};

pub use event::Update;

#[cfg(any(feature = "client", feature = "server", feature = "error"))]
pub use error::{Error, Result};

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub use server::Database;
