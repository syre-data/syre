//! Top level functions.
use crate::resources::search_filter::dict_map_to_filter;
use crate::types::DictMap;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PySet;
use thot_core::db::SearchFilter;
use thot_core::project::{Asset, Container};
use thot_core::runner::common as thot_runner;

/// Returns whether the script is being run in developement mode.
#[pyfunction]
pub fn dev_mode() -> bool {
    thot_runner::dev_mode()
}

// @todo: Improve implementation with typing.
/// Filters a collection.
#[pyfunction]
pub fn filter(py: Python<'_>, search: DictMap, objects: &PySet) -> PyResult<PyObject> {
    let filter = dict_map_to_filter(py, Some(search))?;
    let matches = PySet::empty(py).expect("could not create new `PySet`");

    for obj in objects.iter() {
        if let Ok(asset) = obj.extract::<Asset>() {
            if filter.matches(&asset) {
                matches.add(obj)?;
            }
        } else if let Ok(container) = obj.extract::<Container>() {
            if filter.matches(&container) {
                matches.add(obj)?;
            }
        } else {
            return Err(PyTypeError::new_err(format!("Invalid element {obj:}")));
        }
    }

    Ok(matches.to_object(py))
}

#[cfg(test)]
#[path = "./functions_test.rs"]
mod functions_test;
