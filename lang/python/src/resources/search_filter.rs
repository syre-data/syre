//! Serach filter functionality.
use crate::types::DictMap;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::collections::HashSet;
use std::str::FromStr;
use thot_core::db::resources::search_filter::StandardSearchFilter as StdFilter;
use thot_core::types::ResourceId;

/// Convert a raw search map to a StandardPropertiesSearchFilter.
pub fn dict_map_to_filter(py: Python<'_>, search: Option<DictMap>) -> PyResult<StdFilter> {
    match search {
        None => Ok(StdFilter::new()),
        Some(m) => convert_dict_map_to_search_filter(py, m),
    }
}

/// Convert a HashMap to a StandardPropertiesSearchFilter.
///
/// # Errors
/// + If an invalid key is encountered.
/// + If a valid key has an invalid type or value.
fn convert_dict_map_to_search_filter(py: Python<'_>, map: DictMap) -> PyResult<StdFilter> {
    let mut filter = StdFilter::new();
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
                    let name = v.extract::<String>(py);
                    if name.is_err() {
                        return Err(PyTypeError::new_err("Invalid value for `name`"));
                    }

                    Some(name.unwrap())
                };

                filter.name = Some(name);
            }
            "type" => {
                let kind = if v.is_none(py) {
                    None
                } else {
                    let kind = v.extract::<String>(py);
                    if kind.is_err() {
                        return Err(PyTypeError::new_err("Invalid value for `kind`"));
                    }

                    Some(kind.unwrap())
                };

                filter.kind = Some(kind);
            }
            "tags" => {
                let tags = v.extract::<HashSet<String>>(py);
                if tags.is_err() {
                    return Err(PyTypeError::new_err("Invalid value for `tags`"));
                }

                let tags = tags.unwrap();
                filter.tags = Some(tags);
            }
            "metadata" => {
                //                let md = v.extract::<HashMap<String, SerdeValue>>(py);
                //                if !md.is_err() {
                //                    return Err(PyTypeError::new_err("Invalid value for `metadata`"));
                //                }
                //
                //                let md = md.unwrap();
                //                filter.metadata = Some(md);
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
