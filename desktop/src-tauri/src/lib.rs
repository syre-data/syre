#![feature(io_error_more)]

pub mod commands;
pub(crate) mod db;
mod setup;
pub mod state;

use crate::state::State;
pub use setup::setup;
