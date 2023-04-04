//! Thot project database.
use super::search_filter::dict_map_to_filter;
use super::Asset;
use crate::types::DictMap;
use current_platform::CURRENT_PLATFORM;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::PyTypeInfo;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer, Project};
use thot_core::runner::{common as thot_runner, ThotEnv};
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::project::project::project_resource_root_path;
use thot_local_database::{
    AssetCommand, Client as DbClient, ContainerCommand, GraphCommand, ProjectCommand,
    Result as DbResult,
};

type ContainerTree = ResourceTree<CoreContainer>;

// ***************
// *** Project ***
// ***************

/// A Thot Database.
#[pyclass]
pub struct Database {
    root: ResourceId,

    #[pyo3(get)]
    root_path: PathBuf,

    db: DbClient,
}

#[pymethods]
impl Database {
    /// Initialize a new Thot Project.
    #[new]
    fn py_new(py: Python<'_>, dev_root: Option<PathBuf>) -> PyResult<Self> {
        // start database
        if !DbClient::server_available() {
            // create path to database executable
            let mut exe = resources_path(py)?;
            exe.push("package_data");
            exe.push(format!("thot-local-database-{CURRENT_PLATFORM:}"));

            #[cfg(target_os = "windows")]
            exe.set_extension("exe");

            let _server = Command::new(exe)
                .spawn()
                .expect("could not start database server");
        }

        let db = DbClient::new();

        // resolve root
        let root_path = if Self::dev_mode(Self::type_object(py)) {
            let Some(dev_root) = dev_root else {
                return Err(PyValueError::new_err(
                    "`dev_root` must be specified",
                ));
            };

            dev_root
        } else {
            // @todo: Pass Container path instead of id
            let Ok(root_id) = env::var(ThotEnv::container_id_key()) else {
                return Err(PyValueError::new_err(
                    "could not get `THOT_CONTAINER_ID`"
                ));
            };

            let root_id = ResourceId::from_str(&root_id)
                .expect("could not convert `THOT_CONTAINER_ID` to `ResourceId`");

            let root_path = db.send(ContainerCommand::GetPath(root_id).into());
            let root_path: DbResult<Option<PathBuf>> = serde_json::from_value(root_path)
                .expect("could not convert result of `GetPath` to `PathBuf`");

            let Ok(Some(root_path)) = root_path else {
                return Err(PyRuntimeError::new_err("Could not get root `Container` path"));
            };

            PathBuf::from(root_path)
        };

        // get project id
        let Ok(project_path) = project_resource_root_path(&root_path) else {
                return Err(PyRuntimeError::new_err("Root path is not a resource in a Thot project"));
        };

        let project = db.send(ProjectCommand::Load(project_path).into());
        let project: DbResult<Project> = serde_json::from_value(project)
            .expect("could not convert result of `Load` to `Project`");

        let Ok(project) = project else {
            return Err(PyRuntimeError::new_err("Could not load `Project`"));
        };

        // load tree
        let graph = db.send(GraphCommand::Load(project.rid.clone()).into());
        let graph: DbResult<ContainerTree> =
            serde_json::from_value(graph).expect("could not convert result of `Load` to graph");

        let Ok(_graph) = graph else {
            return Err(PyRuntimeError::new_err("Could not load `Container` tree"));
        };

        // get root container
        let root = db.send(ContainerCommand::ByPath(root_path.clone()).into());
        let root: Option<CoreContainer> = serde_json::from_value(root)
            .expect("could not convert result of `ByPath` to `Container`");

        let Some(root) = root else {
            return Err(PyRuntimeError::new_err("Could not get root `Container`"));
        };

        Ok(Self {
            root: root.rid.clone(),
            root_path,
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
            .send(ContainerCommand::Get(self.root.clone()).into());

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
            .send(ContainerCommand::FindWithMetadata(self.root.clone(), filter).into());

        let containers: HashSet<CoreContainer> = serde_json::from_value(containers)
            .expect("could not convert result of `Find` to `HashSet<Container>`");

        Ok(containers)
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
            .send(AssetCommand::FindWithMetadata(self.root.clone(), filter).into());

        let assets: HashSet<CoreAsset> = serde_json::from_value(assets)
            .expect("could not convert result of `Find` to `HashSet<Asset>`");

        Ok(assets)
    }

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
            .send(ContainerCommand::Get(self.root.clone()).into());

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

    // @todo[3]: Move to getter of `Container` and `Asset`.
    fn parent(&self, id: String) -> PyResult<Option<CoreContainer>> {
        // convert string to id
        let Ok(id) = ResourceId::from_str(&id) else {
            return Err(PyValueError::new_err("Invalid id"));
        };

        // try as asset
        let parent = self.db.send(AssetCommand::Parent(id.clone()).into());
        let parent: Option<CoreContainer> = serde_json::from_value(parent)
            .expect("could not convert result of `Parent` to `Container`");

        if parent.is_some() {
            return Ok(parent);
        }

        // try as container
        let parent = self.db.send(ContainerCommand::Parent(id).into());
        let parent: DbResult<Option<CoreContainer>> = serde_json::from_value(parent)
            .expect("could not convert result of `Parent` to `Container`");

        if parent.is_err() {
            // could not get parent as Asset or Container
            return Err(PyRuntimeError::new_err("Could not get parent"));
        }

        // @todo: Check that parent is in current subtree.
        Ok(parent.unwrap())
    }
}

// ***************
// *** helpers ***
// ***************

// for docs see: https://pyo3.rs/v0.18.2/python_from_rust
fn resources_path(py: Python<'_>) -> PyResult<PathBuf> {
    let resources = py.import("importlib.resources")?;
    let files = resources.call_method1("files", ("thot",))?;
    let files = resources.call_method1("as_file", (files,))?;
    let path = files.call_method0("__enter__")?; // enter python context manager
    match path.extract() {
        Ok(path) => {
            let none = py.None();
            files.call_method1("__exit__", (&none, &none, &none))?;
            Ok(path)
        }
        Err(err) => {
            files.call_method1(
                "__exit__",
                (err.get_type(py), err.value(py), err.traceback(py)),
            )?;

            Err(err)
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
