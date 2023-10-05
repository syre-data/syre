#![feature(mutex_unlock)]
//! # Thot Local Database
//! Implements a local database for Thot.
pub mod command;
pub mod update;

#[cfg(any(feature = "client", feature = "server"))]
pub mod constants;

#[cfg(any(feature = "client", feature = "server"))]
pub mod common;

#[cfg(any(feature = "client", feature = "server"))]
pub mod error;

#[cfg(any(feature = "client", feature = "server"))]
pub mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

// Re-exports
pub use command::{
    AssetCommand, Command, ContainerCommand, DatabaseCommand, GraphCommand, ProjectCommand,
    ScriptCommand,
};

pub use update::Update;

#[cfg(any(feature = "client", feature = "server"))]
pub use error::{Error, Result};

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub use server::Database;
