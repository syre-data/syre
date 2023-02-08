//! Asset
use crate::types::DictMap;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use thot_core::project::{Asset as PrjAsset, StandardProperties as StdProps};
use thot_core::types::{ResourceId, ResourcePath};

// *************
// *** Asset ***
// *************

/// Represents a user defined Asset.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct Asset {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub metadata: Option<DictMap>,
    pub file: Option<PathBuf>,
}

impl Asset {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts self into a [`thot_core::project::Asset`].
    pub fn into_project_asset(self, py: Python<'_>, rid: Option<ResourceId>) -> PyResult<PrjAsset> {
        let rid = rid.unwrap_or_else(|| ResourceId::new());

        // path
        let path = derive_path(&self, rid.clone());
        let Ok(path) = ResourcePath::new(path) else {
            return Err(PyRuntimeError::new_err(
                "Could not convert path to resource path.",
            ));
        };

        // properties
        let mut props = StdProps::default();
        props.name = self.name;
        props.kind = self.kind;
        props.description = self.description;

        props.tags = match self.tags {
            None => Vec::new(),
            Some(t) => t.into_iter().collect(),
        };

        props.metadata = match self.metadata {
            None => HashMap::new(),
            Some(py_md) => {
                let mut md = HashMap::new();
                for (k, v) in py_md {
                    let val = depythonize(v.as_ref(py));
                    if val.is_err() {
                        return Err(PyValueError::new_err(format!(
                            "Could not convert value for key `{}`: {:?}",
                            k, val
                        )));
                    }

                    let val = val.unwrap();
                    md.insert(k, val);
                }

                md
            }
        };

        Ok(PrjAsset {
            rid,
            properties: props,
            path,
        })
    }
}

// *****************
// *** functions ***
// *****************

pub fn dict_map_to_asset(py: Python<'_>, map: Option<DictMap>) -> PyResult<Asset> {
    match map {
        None => Ok(Asset::new()),
        Some(m) => convert_dict_map_to_asset(py, m),
    }
}

// ************************
// *** helper functions ***
// ************************

/// Derives a canonical file path for an [`Asset`].
fn derive_path(asset: &Asset, rid: ResourceId) -> PathBuf {
    if let Some(file) = asset.file.clone() {
        return file;
    }

    if let Some(name) = asset.name.clone() {
        return PathBuf::from(name);
    }

    // default to id
    PathBuf::from(rid.to_string())
}

fn convert_dict_map_to_asset(py: Python<'_>, map: DictMap) -> PyResult<Asset> {
    let mut asset = Asset::new();
    for (k, v) in map {
        match k.as_str() {
            "name" => {
                let name = depythonize(v.as_ref(py));
                if name.is_err() {
                    return Err(PyTypeError::new_err("Invalid value for `name`"));
                }

                asset.name = name.unwrap();
            }
            "kind" => {
                let kind = depythonize(v.as_ref(py));
                if kind.is_err() {
                    return Err(PyTypeError::new_err("Invalid value for `kind`"));
                }

                asset.kind = kind.unwrap();
            }
            "description" => {
                let desc = depythonize(v.as_ref(py));
                if desc.is_err() {
                    return Err(PyTypeError::new_err("Invalid value for `desc`"));
                }

                asset.description = desc.unwrap();
            }
            "tags" => {}
            "metadata" => {}
            "file" => {
                let file = depythonize(v.as_ref(py));
                if file.is_err() {
                    return Err(PyTypeError::new_err("Invalid value for `file`"));
                }

                asset.file = file.unwrap();
            }
            _ => {
                return Err(PyValueError::new_err(format!("Invalid key `{}`", k)));
            }
        }
    }

    Ok(asset)
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
