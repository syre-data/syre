//! Thot Project.
use super::search_filter::dict_map_to_filter;
use crate::types::DictMap;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::PyTypeInfo;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use thot_core::db::resources::{
    Asset as DbAsset, Container as DbContainer, StandardSearchFilter as StdPropsFilter,
};
use thot_core::project::Container as CoreContainer;
use thot_core::runner::common as thot_runner;
use thot_core::types::ResourceId;
use thot_local_database::{Client as DbClient, ContainerCommand, Result as DbResult};

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
        let root_container = db.send(ContainerCommand::LoadContainerTree(root_path.clone()).into());
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
    fn root(&self) -> PyResult<DbContainer> {
        let root = self
            .db
            .send(ContainerCommand::GetContainer(self.root_id.clone()).into());

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
    ) -> PyResult<Option<DbContainer>> {
        let filter = dict_map_to_filter(py, search)?;
        Ok(self.db.containers.find_one(&filter))
    }

    // /// Finds all Containers matching the search filter.
    // fn find_containers(
    //     &self,
    //     py: Python<'_>,
    //     search: Option<DictMap>,
    // ) -> PyResult<HashSet<DbContainer>> {
    //     let filter = dict_map_to_filter(py, search)?;
    //     Ok(self.db.containers.find(&filter))
    // }

    // /// Finds a single Asset matching the search filter.
    // fn find_asset(&self, py: Python<'_>, search: Option<DictMap>) -> PyResult<Option<DbAsset>> {
    //     let filter = dict_map_to_filter(py, search)?;
    //     Ok(self.db.assets.find_one(&filter))
    // }

    // /// Finds all Assets matching the search filter.
    // fn find_assets(&self, py: Python<'_>, search: Option<DictMap>) -> PyResult<HashSet<DbAsset>> {
    //     let filter = dict_map_to_filter(py, search)?;
    //     Ok(self.db.assets.find(&filter))
    // }

    // @todo: fix
    // @todo: Allow either an Asset object or dictionary.
    ///// Adds an Asset to the database.
    /////
    ///// # Arguments
    ///// + `asset`: Dictionary of properties for the Asset.
    ///// + `_id`: Id for the Asset.
    ///// + `overwrite`: Whether the Asset can be overwritten if it already exists.
    /////
    ///// # Returns
    ///// The Asset's file path.
    //fn add_asset(
    //    &mut self,
    //    py: Python<'_>,
    //    asset: Option<DictMap>,
    //    _id: Option<String>,
    //    overwrite: Option<bool>,
    //) -> PyResult<PathBuf> {
    //    // -- validation
    //    // load root container
    //    let c_root = PrjContainer::load(&self.root_path);
    //    if c_root.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not load root Container to register Asset {:?}",
    //            c_root
    //        )));
    //    }

    //    let mut c_root = c_root.unwrap();

    //    // load assets
    //    let prj_assets = PrjAssets::load(&self.root_path);
    //    if prj_assets.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not load Assets {:?}",
    //            prj_assets
    //        )));
    //    }

    //    let mut prj_assets = prj_assets.unwrap();

    //    // check if asset is already registered if `overwrite` is false.
    //    if overwrite == Some(false) {
    //        if _id.is_none() {
    //            return Err(PyValueError::new_err(
    //                "If `overwrite` is `False` an `_id` must be set.",
    //            ));
    //        }

    //        let lid = _id.clone().unwrap();
    //        if prj_assets.contains_lid(&lid) {
    //            return Err(PyRuntimeError::new_err(format!(
    //                "Asset with id `{}` already exists",
    //                lid
    //            )));
    //        }
    //    }

    //    // create project asset
    //    let asset = dict_map_to_asset(py, asset)?;
    //    let mut prj_asset = asset.into_project_asset(py, _id.clone())?;
    //    let path = prj_asset.path.clone();
    //    if path.is_none() {
    //        return Err(PyRuntimeError::new_err(
    //            "Could not get path to Asset's file",
    //        ));
    //    }
    //    let path = path.unwrap();

    //    // ensure path is relative
    //    if !matches!(path, ResourcePath::Relative(_)) {
    //        return Err(PyValueError::new_err("Asset's file path must be relative"));
    //    }

    //    // --- creation
    //    // save asset
    //    let found = match &_id {
    //        None => None,
    //        Some(id) => prj_assets.index_by_lid(id),
    //    };

    //    match found {
    //        None => prj_assets.push(prj_asset.clone()),
    //        Some(ind) => {
    //            let o_asset = prj_assets.assets.get(ind).unwrap();
    //            prj_asset.properties.rid = o_asset.properties.rid.clone();
    //            prj_assets.assets[ind] = prj_asset.clone();
    //        }
    //    }

    //    let save_res = prj_assets.save();
    //    if save_res.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not save Assets {:?}",
    //            save_res
    //        )));
    //    }

    //    // register asset with root container, if needed
    //    let aid = prj_asset.properties.rid.clone();
    //    let prev_reg = c_root.register_asset(aid);
    //    if prev_reg.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not register Asset {:?}",
    //            prev_reg
    //        )));
    //    }

    //    let save_res = c_root.save();
    //    if save_res.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not save Container {:?}",
    //            save_res
    //        )));
    //    }

    //    // insert into or update db
    //    let db_asset = DbAsset::from(prj_asset, self.root_id.clone());
    //    let bucket = db_asset.bucket();
    //    let ins_res = match &_id {
    //        None => self.db.assets.insert_one(db_asset),
    //        Some(lid) => {
    //            let mut search = RidFilter::new();
    //            search.lid = Some(Some(lid.clone()));
    //            match self.db.assets.update_one(&search, db_asset) {
    //                Ok(_) => Ok(()),
    //                Err(err) => Err(err),
    //            }
    //        }
    //    };

    //    if ins_res.is_err() {
    //        return Err(PyRuntimeError::new_err(format!(
    //            "Could not insert Asset into the database: {:?}",
    //            ins_res
    //        )));
    //    }

    //    // create bucket if needed
    //    if bucket.is_err() {
    //        // @unreachable
    //        return Err(PyRuntimeError::new_err("Asset's path not set"));
    //    }

    //    let bucket = bucket.unwrap();
    //    if let Some(bp) = bucket {
    //        let b_path = self.root_path.join(bp);
    //        if !b_path.exists() {
    //            let dir_res = fs::create_dir_all(b_path);
    //            if dir_res.is_err() {
    //                return Err(PyRuntimeError::new_err(format!(
    //                    "Could not create bucket: {:?}",
    //                    dir_res
    //                )));
    //            }
    //        } else {
    //            if !b_path.is_dir() {
    //                return Err(PyRuntimeError::new_err("Bucket is not a directory"));
    //            }
    //        }
    //    }

    //    // return asset file path to user
    //    let path = path.as_path().to_path_buf();
    //    let path = self.root_path.join(path);
    //    Ok(path)
    //}
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
