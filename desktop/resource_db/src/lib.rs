#[cfg(any(feature = "server", feature = "client"))]
mod command;

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "client")]
mod client;

#[cfg(any(feature = "server", feature = "client"))]
pub(crate) use command::Command;

#[cfg(feature = "server")]
pub use server::Builder;

#[cfg(feature = "client")]
pub use client::Client;
