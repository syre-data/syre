//! Top level functions.
use extendr_api::prelude::*;
use thot_core::runner::common as thot_runner;

/// Return whether Thot is running in development mode.
/// @export
#[extendr]
pub fn dev_mode() -> bool {
    thot_runner::dev_mode()
}

extendr_module! {
    mod functions;
    fn dev_mode;
}

#[cfg(test)]
#[path = "./functions_test.rs"]
mod functions_test;
