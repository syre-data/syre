//! Container
use crate::types::DictMap;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use std::collections::{HashMap, HashSet};
use thot_core::project::{
    container::AssetMap, container::ScriptMap, Container as CoreContainer,
    StandardProperties as StdProps,
};
use thot_core::types::ResourceId;

/// Represents a user defined Container.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct Container {
    // db: DbClient,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub metadata: Option<DictMap>,
    pub assets: AssetMap,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts self into a [`thot_core::project::Asset`].
    pub fn into_core_container(
        self,
        py: Python<'_>,
        rid: Option<ResourceId>,
    ) -> PyResult<CoreContainer> {
        let rid = rid.unwrap_or_else(|| ResourceId::new());

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
                    let Ok(val) = depythonize(v.as_ref(py)) else {
                        return Err(PyValueError::new_err(format!(
                            "Could not convert value for key `{}`: {:?}",
                            k, v
                        )));
                    };

                    md.insert(k, val);
                }

                md
            }
        };

        Ok(CoreContainer {
            rid,
            properties: props,
            assets: self.assets,
            scripts: ScriptMap::new(),
        })
    }

    pub fn from_dict_map(py: Python<'_>, map: DictMap) -> PyResult<Container> {
        let mut container = Container::new();
        for (k, v) in map {
            match k.as_str() {
                "name" => {
                    let name = depythonize(v.as_ref(py));
                    if name.is_err() {
                        return Err(PyTypeError::new_err("Invalid value for `name`"));
                    }

                    container.name = name.unwrap();
                }
                "type" => {
                    let kind = depythonize(v.as_ref(py));
                    if kind.is_err() {
                        return Err(PyTypeError::new_err("Invalid value for `type`"));
                    }

                    container.kind = kind.unwrap();
                }
                "description" => {
                    let desc = depythonize(v.as_ref(py));
                    if desc.is_err() {
                        return Err(PyTypeError::new_err("Invalid value for `desc`"));
                    }

                    container.description = desc.unwrap();
                }
                "tags" => {}
                "metadata" => {}
                _ => {}
            }
        }

        Ok(container)
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
