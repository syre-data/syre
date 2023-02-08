//! Top level functions.
use crate::types::DictMap;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashSet;
use thot_core::db::resources::object::StandardObject as StdObject;
use thot_core::runner::common as thot_runner;

/// Returns whether the script is being run in developement mode.
#[pyfunction]
pub fn dev_mode() -> bool {
    thot_runner::dev_mode()
}

// @todo: Allow both Containers and Assets mixed as input and output.
// /// Filters a collection.
// #[pyfunction]
// pub fn filter(
//     py: Python<'_>,
//     search: DictMap,
//     objects: HashSet<dyn StdObject>,
// ) -> PyResult<HashSet<dyn StdObject>> {
//     todo!();
//     let filter = dict_map_to_filter(py, search)?;
// }

#[cfg(test)]
#[path = "./functions_test.rs"]
mod functions_test;
