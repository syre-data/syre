//! Thot Project.
use super::search_filter::dict_map_to_filter;
use super::Asset;
use crate::types::DictMap;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::PyTypeInfo;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer};
use thot_core::runner::common as thot_runner;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local_database::{AssetCommand, Client as DbClient, ContainerCommand, Result as DbResult};

// ***************
// *** Project ***
// ***************

// @todo: Inject database so loading in init is not required.
/// A Thot Database.
#[pyclass]
pub struct Database {
    #[pyo3(get)]
    root_path: PathBuf,
    root_id: ResourceId,
    db: DbClient,
}

#[pymethods]
impl Database {
    /// Initialize a new Thot Project.
    #[new]
    fn py_new(py: Python<'_>, root: Option<PathBuf>, dev_root: Option<PathBuf>) -> PyResult<Self> {
        // resolve root
        let root_path = if Self::dev_mode(Self::type_object(py)) {
            dev_root
        } else {
            root
        };

        let Some(root_path) = root_path else {
            return Err(PyValueError::new_err(
                "Root `Container` not passed and `THOT_CONTAINER_ID` not set.",
            ));
        };

        // start database
        if !DbClient::server_available() {
            let _server =
                Command::new("./thot.data/data/thot-local-database-x86_64-unknown-linux-gnu")
                    .spawn()
                    .expect("could not start database server");
        }

        let db = DbClient::new();

        // load tree
        let root_container = db.send(ContainerCommand::LoadTree(root_path.clone()).into());
        let root_container: DbResult<CoreContainer> = serde_json::from_value(root_container)
            .expect("could not convert result of `LoadContainerTree` to `Container`");

        let Ok(root_container) = root_container else {
            return Err(PyRuntimeError::new_err("Could not load `Container` tree"));
        };

        Ok(Self {
            root_path,
            root_id: root_container.rid.clone(),
            db,
        })
    }

    /// Returns whether the script is being run in developement mode.
    #[classmethod]
    fn dev_mode(_cls: &PyType) -> bool {
        thot_runner::dev_mode()
    }

    /// Returns the root Container of the project.
    #[getter]
    fn root(&self) -> PyResult<CoreContainer> {
        let root = self
            .db
            .send(ContainerCommand::Get(self.root_id.clone()).into());

        let root: Option<CoreContainer> = serde_json::from_value(root)
            .expect("could not convert result of `GetContainer` to `Container`");

        let Some(root) = root else {
            return Err(PyRuntimeError::new_err("Could not find root Container"));
        };

        Ok(root.into())
    }

    /// Finds a single Container matching the search fitler.
    fn find_container(
        &self,
        py: Python<'_>,
        search: Option<DictMap>,
    ) -> PyResult<Option<CoreContainer>> {
        let containers = self.find_containers(py, search)?;
        Ok(containers.into_iter().next())
    }

    /// Finds all Containers matching th1 search filter.
    fn find_containers(
        &self,
        py: Python<'_>,
        search: Option<DictMap>,
    ) -> PyResult<HashSet<CoreContainer>> {
        let filter = dict_map_to_filter(py, search)?;
        let containers = self
            .db
            .send(ContainerCommand::Find(self.root_id.clone(), filter).into());

        let containers: HashSet<CoreContainer> = serde_json::from_value(containers)
            .expect("could not convert result of `Find` to `HashSet<Container>`");

        return Ok(containers);
    }

    /// Finds a single Asset matching the search filter.
    fn find_asset(&self, py: Python<'_>, search: Option<DictMap>) -> PyResult<Option<CoreAsset>> {
        let assets = self.find_assets(py, search)?;
        Ok(assets.into_iter().next())
    }

    /// Finds all Assets matching the search filter.
    fn find_assets(&self, py: Python<'_>, search: Option<DictMap>) -> PyResult<HashSet<CoreAsset>> {
        let filter = dict_map_to_filter(py, search)?;
        let assets = self
            .db
            .send(AssetCommand::Find(self.root_id.clone(), filter).into());

        let assets: HashSet<CoreAsset> = serde_json::from_value(assets)
            .expect("could not convert result of `Find` to `HashSet<Asset>`");

        return Ok(assets);
    }

    // @todo: fix
    // @todo: Allow either an Asset object or dictionary.
    /// Adds an Asset to the database.
    ///
    /// # Arguments
    /// + `asset`: Dictionary of properties for the Asset.
    /// + `overwrite`: Whether the Asset can be overwritten if it already exists.
    ///
    /// # Returns
    /// The Asset's file path.
    fn add_asset(
        &mut self,
        py: Python<'_>,
        asset: Option<DictMap>,
        overwrite: Option<bool>,
    ) -> PyResult<PathBuf> {
        let asset = match asset {
            None => Asset::new(),
            Some(map) => Asset::from_dict_map(py, map)?,
        };

        let root = self
            .db
            .send(ContainerCommand::Get(self.root_id.clone()).into());

        let root: CoreContainer =
            serde_json::from_value(root).expect("could not convert result of `Get` to `Container`");

        // check if asset already exists
        let Some(path) =  asset.path.as_ref() else {
                return Err(PyValueError::new_err(
                    "If `overwrite` is `False` the Asset's `file` must be set",
                ));
            };

        let Ok(path) = ResourcePath::new(path.clone()) else {
                return Err(PyValueError::new_err(
                    "Invalid file {path}, could not convert to `ResourcePath`",
                ));
            };

        let mut asset_id = None;
        for c_asset in root.assets.values() {
            if c_asset.path == path {
                if overwrite == Some(false) {
                    return Err(PyRuntimeError::new_err(
                        "Asset with file `{path}` already exists",
                    ));
                }

                asset_id = Some(c_asset.rid.clone());
                break;
            }
        }

        let asset = asset.into_core_asset(py, asset_id)?;
        let asset_path = asset.path.clone();
        let bucket = asset.bucket();
        let res = self
            .db
            .send(AssetCommand::Add(asset, root.rid.clone()).into());

        let res: DbResult<Option<CoreAsset>> = serde_json::from_value(res)
            .expect("could not convert result of `Add` to `Option<Asset>`");

        if res.is_err() {
            return Err(PyRuntimeError::new_err("Could not create `Asset`"));
        }

        // ensure bucket exists
        if let Some(bucket) = bucket {
            let mut path = self.root_path.clone();
            path.push(bucket);

            let res = fs::create_dir_all(&path);
            if res.is_err() {
                return Err(PyRuntimeError::new_err(
                    "Could not create directory `{path}`",
                ));
            }
        }

        let mut path = self.root_path.clone();
        path.push(asset_path);
        Ok(path.into())
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
