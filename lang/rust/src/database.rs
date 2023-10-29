//! Thot project database.
use crate::{Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset, Container, Project};
use thot_core::runner::{common as thot_runner, CONTAINER_ID_KEY};
use thot_core::types::ResourceId;
use thot_local::project::project::project_resource_root_path;
use thot_local_database::{
    AssetCommand, Client as DbClient, ContainerCommand, GraphCommand, ProjectCommand,
    Result as DbResult,
};

pub type ContainerTree = ResourceTree<Container>;

/// A Thot Database.
pub struct Database {
    root: ResourceId,
    root_path: PathBuf,
    db: DbClient,
}

impl Database {
    /// Initialize a new Thot Project.
    ///
    /// # Arguments
    /// 1. Path to the root `Container` for use in dev mode.
    /// 2. Path to the local database server executable.
    ///     Used to instantiate a local database server if needed.
    pub fn new(dev_root: Option<PathBuf>, db_server_path: &Path) -> Result<Self> {
        if !DbClient::server_available() {
            let _server = Command::new(db_server_path)
                .spawn()
                .expect("could not start database server");
        }

        let db = DbClient::new();

        // resolve root
        let root_path = if thot_runner::dev_mode() {
            let Some(dev_root) = dev_root else {
                return Err(Error::Value("`dev_root` must be specified".into()));
            };

            dev_root
        } else {
            // TODO: Pass Container path instead of id?
            let Ok(root_id) = env::var(CONTAINER_ID_KEY) else {
                return Err(Error::Runtime(format!(
                    "could not get `{CONTAINER_ID_KEY}`"
                )));
            };

            let root_id = ResourceId::from_str(&root_id)
                .expect("could not convert `THOT_CONTAINER_ID` to `ResourceId`");

            let root_path = db.send(ContainerCommand::GetPath(root_id).into())?;
            let root_path: Option<PathBuf> = serde_json::from_value(root_path)
                .expect("could not convert result of `GetPath` to `PathBuf`");

            let Some(root_path) = root_path else {
                return Err(Error::Runtime("Could not get root `Container` path".into()));
            };

            PathBuf::from(root_path)
        };

        // get project id
        let Ok(project_path) = project_resource_root_path(&root_path) else {
            return Err(Error::Runtime(
                "Root path is not a resource in a Thot project".into(),
            ));
        };

        let project = db.send(ProjectCommand::Load(project_path).into())?;
        let project: DbResult<Project> = serde_json::from_value(project)
            .expect("could not convert result of `Load` to `Project`");

        let Ok(project) = project else {
            return Err(Error::Runtime("Could not load `Project`".into()));
        };

        // load tree
        let graph = db.send(GraphCommand::Load(project.rid.clone()).into())?;
        let graph: DbResult<ContainerTree> =
            serde_json::from_value(graph).expect("could not convert result of `Load` to graph");

        let Ok(_graph) = graph else {
            return Err(Error::Runtime("Could not load `Container` tree".into()));
        };

        // get root container
        let root = db.send(ContainerCommand::ByPath(root_path.clone()).into())?;
        let root: Option<Container> = serde_json::from_value(root)
            .expect("could not convert result of `ByPath` to `Container`");

        let Some(root) = root else {
            return Err(Error::Runtime("Could not get root `Container`".into()));
        };

        Ok(Self {
            root: root.rid.clone(),
            root_path,
            db,
        })
    }

    /// Returns the root Container of the project.
    pub fn root(&self) -> Result<Container> {
        let root = self
            .db
            .send(ContainerCommand::Get(self.root.clone()).into())?;

        let root: Option<Container> = serde_json::from_value(root)
            .expect("could not convert result of `GetContainer` to `Container`");

        let Some(root) = root else {
            return Err(Error::Runtime("Could not find root Container".into()));
        };

        Ok(root.into())
    }

    /// Finds a single Container matching the search fitler.
    pub fn find_container(&self, filter: StdFilter) -> Result<Option<Container>> {
        let containers = self.find_containers(filter)?;
        Ok(containers.into_iter().next())
    }

    /// Finds all Containers matching th1 search filter.
    pub fn find_containers(&self, filter: StdFilter) -> Result<HashSet<Container>> {
        let containers = self
            .db
            .send(ContainerCommand::FindWithMetadata(self.root.clone(), filter).into())?;

        let containers: HashSet<Container> = serde_json::from_value(containers)
            .expect("could not convert result of `Find` to `HashSet<Container>`");

        Ok(containers)
    }

    /// Finds a single Asset matching the search filter.
    pub fn find_asset(&self, filter: StdFilter) -> Result<Option<Asset>> {
        let assets = self.find_assets(filter)?;
        Ok(assets.into_iter().next())
    }

    /// Finds all Assets matching the search filter.
    pub fn find_assets(&self, filter: StdFilter) -> Result<HashSet<Asset>> {
        let assets = self
            .db
            .send(AssetCommand::FindWithMetadata(self.root.clone(), filter).into())?;

        let assets: HashSet<Asset> = serde_json::from_value(assets)
            .expect("could not convert result of `Find` to `HashSet<Asset>`");

        Ok(assets)
    }

    // @todo: Allow either an Asset object or dictionary.
    /// Adds an Asset to the database.
    ///
    /// # Arguments
    /// 1. Dictionary of properties for the Asset.
    /// 2. Whether the Asset can be overwritten if it already exists.
    ///
    /// # Returns
    /// The Asset's file path.
    pub fn add_asset(&self, asset: Asset) -> Result<PathBuf> {
        let root = self
            .db
            .send(ContainerCommand::Get(self.root.clone()).into())?;

        let root: Container =
            serde_json::from_value(root).expect("could not convert result of `Get` to `Container`");

        let asset_path = asset.path.clone();
        let bucket = asset.bucket();
        let res = self
            .db
            .send(AssetCommand::Add(asset, root.rid.clone()).into())?;

        let res: DbResult<Option<Asset>> = serde_json::from_value(res)
            .expect("could not convert result of `Add` to `Option<Asset>`");

        if res.is_err() {
            return Err(Error::Value("Could not create `Asset`".into()));
        }

        // ensure bucket exists
        if let Some(bucket) = bucket {
            let mut path = self.root_path.clone();
            path.push(bucket);

            let res = fs::create_dir_all(&path);
            if res.is_err() {
                return Err(Error::Runtime("Could not create directory `{path}`".into()));
            }
        }

        let mut path = self.root_path.clone();
        path.push(asset_path);
        Ok(path.into())
    }
}
