//! Common types.
use pyo3::prelude::*;
use std::collections::HashMap;

// ***************
// *** DictMap ***
// ***************

/// Type representing a Python dictionary keyed by strings with arbitrary values.
pub type DictMap = HashMap<String, PyObject>;

#[cfg(test)]
#[path = "./types_test.rs"]
mod types_test;
