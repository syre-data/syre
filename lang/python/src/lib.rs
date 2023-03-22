//! Python bindings for Thot.
mod functions;
mod resources;
pub mod types;

use crate::functions as fcn;
use crate::resources::Database;
use pyo3::prelude::*;

/// Thot's language bindings for Python.
#[pymodule]
fn thot(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", get_version())?;
    m.add_class::<Database>()?;
    m.add_function(wrap_pyfunction!(fcn::dev_mode, m)?)?;
    m.add_function(wrap_pyfunction!(fcn::filter, m)?)?;
    Ok(())
}

/// Gets the package version.
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
