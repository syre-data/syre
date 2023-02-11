//! PyO3 implementation for [`Asset`].
use crate::project::{Asset, StandardProperties};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pythonize::pythonize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// @todo: Unify implementations of property getters with Container.
#[pymethods]
impl Asset {
    // properties
    #[getter(_id)]
    fn py_id(&self) -> String {
        self.rid.to_string()
    }

    #[getter(_properties)]
    fn py_properties(&self) -> StandardProperties {
        self.properties.clone()
    }

    #[getter(name)]
    fn py_name(&self) -> Option<String> {
        self.properties.name.clone()
    }

    #[getter(type)]
    fn py_kind(&self) -> Option<String> {
        self.properties.kind.clone()
    }

    #[getter(tags)]
    fn py_tags(&self) -> HashSet<String> {
        self.properties.tags.clone().into_iter().collect()
    }

    // @todo: Provide inherited and native metadata separated.
    /// Returns all metadata.
    #[getter(metadata)]
    fn py_metadata(&self, py: Python<'_>) -> PyResult<HashMap<String, PyObject>> {
        let mut md = HashMap::with_capacity(self.properties.metadata.len());
        for (k, v) in self.properties.metadata.clone() {
            let val = pythonize(py, &v);
            if val.is_err() {
                return Err(PyRuntimeError::new_err(format!(
                    "Could not convert metadata of key `{}`: {:?}",
                    k, val
                )));
            }

            let val = val.unwrap();
            md.insert(k, val);
        }

        Ok(md)
    }

    // others
    #[getter(path)]
    fn py_path(&self) -> PathBuf {
        PathBuf::from(self.path.as_path())
    }

    // @todo: Unclear if needed.
    // #[getter(parent)]
    // fn py_parent(&self) -> Option<ResourceId> {
    //     self.parent.clone()
    // }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
