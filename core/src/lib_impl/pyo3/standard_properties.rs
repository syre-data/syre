//! [`PyO3`](pyo3) implementation for [`StandardProperties`].
use crate::project::StandardProperties;
use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use serde_json::Value as SerdeValue;
use std::collections::{HashMap, HashSet};

#[pymethods]
impl StandardProperties {
    #[getter(name)]
    fn py_name(&self) -> Option<String> {
        self.name.clone()
    }

    #[getter(kind)]
    fn py_kind(&self) -> Option<String> {
        self.kind.clone()
    }

    #[getter(tags)]
    fn py_tags(&self) -> HashSet<String> {
        self.tags.clone().into_iter().collect()
    }

    #[getter(metadata)]
    fn py_metadata(&self) -> HashMap<String, SerdePyValue> {
        let mut md = HashMap::new();
        for (k, v) in &self.metadata {
            let val = SerdePyValue(v.clone());
            md.insert(k.clone(), val);
        }
        md
    }
}

/// Intermediate struct for converting between serde_json::Value's and pyo3::PyObject's.
#[repr(transparent)]
#[derive(Clone, Debug)]
struct SerdePyValue(serde_json::Value);

/// Convert from serde_json::Value to pyo3::PyObject.
impl ToPyObject for SerdePyValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        serde_value_to_py_object(&self.0, py)
    }
}

impl IntoPy<PyObject> for SerdePyValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

// from https://stackoverflow.com/questions/70193869/exporting-hashmap-of-hashmap-to-python
/// Convert a serde_json::Value to a [`pyo3::PyObject`].
fn serde_value_to_py_object(val: &SerdeValue, py: Python<'_>) -> PyObject {
    match val {
        SerdeValue::Null => py.None(),
        SerdeValue::Bool(b) => b.to_object(py),
        SerdeValue::Number(n) => {
            let oi64 = n.as_i64().map(|i| i.to_object(py));
            let ou64 = n.as_u64().map(|i| i.to_object(py));
            let of64 = n.as_f64().map(|i| i.to_object(py));
            oi64.or(ou64).or(of64).expect("number too large")
        }
        SerdeValue::String(s) => s.to_object(py),
        SerdeValue::Array(v) => {
            let inner: Vec<_> = v.iter().map(|x| serde_value_to_py_object(x, py)).collect();
            inner.to_object(py)
        }
        SerdeValue::Object(m) => {
            let inner: HashMap<_, _> = m
                .iter()
                .map(|(k, v)| (k, serde_value_to_py_object(v, py)))
                .collect();
            inner.to_object(py)
        }
    }
}
