//! Serach filter functionality.
use crate::types::DictMap;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::depythonize;
use std::collections::HashSet;
use std::str::FromStr;
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::project::Metadata;
use thot_core::types::ResourceId;

/// Convert a raw search map to a StandardPropertiesSearchFilter.
pub fn dict_map_to_filter(py: Python<'_>, search: Option<DictMap>) -> PyResult<StdFilter> {
    match search {
        None => Ok(StdFilter::default()),
        Some(m) => convert_dict_map_to_search_filter(py, m),
    }
}

/// Convert a HashMap to a StandardPropertiesSearchFilter.
///
/// # Errors
/// + If an invalid key is encountered.
/// + If a valid key has an invalid type or value.
fn convert_dict_map_to_search_filter(py: Python<'_>, map: DictMap) -> PyResult<StdFilter> {
    let mut filter = StdFilter::default();
    for (k, v) in map {
        match k.as_str() {
            "_id" => {
                let id = v.extract::<String>(py)?;
                let Ok(id) = ResourceId::from_str(&id) else {
                    return Err(PyValueError::new_err("Invalid value for `_id`"));
                };

                filter.rid = Some(id);
            }
            "name" => {
                let name = if v.is_none(py) {
                    None
                } else {
                    let name = v.extract::<String>(py)?;
                    Some(name)
                };

                filter.name = Some(name);
            }
            "type" => {
                let kind = if v.is_none(py) {
                    None
                } else {
                    let kind = v.extract::<String>(py)?;
                    Some(kind)
                };

                filter.kind = Some(kind);
            }
            "tags" => {
                let tags = v.extract::<HashSet<String>>(py)?;
                filter.tags = Some(tags);
            }
            "metadata" => {
                let md = depythonize(v.as_ref(py))?;
                filter.metadata = Some(md);
            }
            _ => {
                return Err(PyValueError::new_err(format!("Invalid search key `{}`", k)));
            }
        }
    }

    Ok(filter)
}

#[cfg(test)]
#[path = "./search_filter_test.rs"]
mod search_filter_test;
