//! [`PyO3`](pyo3) implementation for types:
//! + [`ResourceId`]
use crate::types::ResourceId;
use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use pyo3::types::PyString;

impl ToPyObject for ResourceId {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let py_str = PyString::new(py, &self.to_string());
        PyObject::from(py_str)
    }
}

// impl IntoPy<PyObject> for ResourceId {
//     fn into_py(self, py: Python<'_>) -> PyObject {
//         self.to_object(py)
//     }
// }

#[cfg(test)]
#[path = "./types_test.rs"]
mod types_test;
