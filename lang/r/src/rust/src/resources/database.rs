//! Database functionality.
use current_platform::CURRENT_PLATFORM;
use extendr_api::prelude::*;
use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::lib_impl::extendr::{asset, container, functions};
use thot_core::project::asset::{Asset, Builder as AssetBuilder};
use thot_core::project::{Container, Metadata};
use thot_core::types::{ResourceId, ResourcePath};
use thot_lang::{Database as BaseDb, Error, Result};

// ****************
// *** Database ***
// ****************

/// A Thot Database.
/// @export
pub struct Database(BaseDb);

impl Database {
    pub fn new(dev_root: Option<PathBuf>) -> Result<Self> {
        let db = BaseDb::new(dev_root, &db_server_path()?)?;
        Ok(Self(db))
    }
}

#[extendr]
impl Database {
    pub fn root(&self) -> Container {
        self.0
            .root()
            .expect("could not get database root `Container`")
    }
}

impl Deref for Database {
    type Target = BaseDb;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Return whether Thot is running in development mode.
/// @export
#[extendr]
fn database(#[default = "NULL"] dev_root: Nullable<String>) -> Database {
    let dev_root = match dev_root {
        Nullable::Null => None,
        Nullable::NotNull(path) => Some(PathBuf::from(path)),
    };

    let db = Database::new(dev_root).expect("could not create database");
    db
}

// *************
// *** find_ ***
// *************

#[extendr]
fn find_container(
    db: &mut Database,
    #[default = "NULL"] _id: Nullable<String>,
    #[default = "NULL"] name: Nullable<String>,
    #[default = "NULL"] r#type: Nullable<String>,
    #[default = "NULL"] tags: Nullable<List>,
    #[default = "NULL"] metadata: Nullable<List>,
) -> Nullable<Container> {
    let filter = FilterArgs::default()
        .set_rid(_id)
        .set_name(name)
        .set_kind(r#type)
        .set_tags(tags)
        .set_metadata(metadata);

    match db
        .find_container(filter.into())
        .expect("could not retrieve `Container`s")
    {
        None => Nullable::Null,
        Some(container) => Nullable::NotNull(container),
    }
}

#[extendr]
fn find_containers(
    db: &mut Database,
    #[default = "NULL"] _id: Nullable<String>,
    #[default = "NULL"] name: Nullable<String>,
    #[default = "NULL"] r#type: Nullable<String>,
    #[default = "NULL"] tags: Nullable<List>,
    #[default = "NULL"] metadata: Nullable<List>,
) -> List {
    let filter = FilterArgs::default()
        .set_rid(_id)
        .set_name(name)
        .set_kind(r#type)
        .set_tags(tags)
        .set_metadata(metadata);

    let containers = db
        .find_containers(filter.into())
        .expect("could not retrieve `Container`s");

    containers.into_iter().collect()
}

#[extendr]
fn find_asset(
    db: &mut Database,
    #[default = "NULL"] _id: Nullable<String>,
    #[default = "NULL"] name: Nullable<String>,
    #[default = "NULL"] r#type: Nullable<String>,
    #[default = "NULL"] tags: Nullable<List>,
    #[default = "NULL"] metadata: Nullable<List>,
) -> Nullable<Asset> {
    let filter = FilterArgs::default()
        .set_rid(_id)
        .set_name(name)
        .set_kind(r#type)
        .set_tags(tags)
        .set_metadata(metadata);

    match db
        .find_asset(filter.into())
        .expect("could not retrieve `Asset`s")
    {
        None => Nullable::Null,
        Some(asset) => Nullable::NotNull(asset),
    }
}

#[extendr]
fn find_assets(
    db: &mut Database,
    #[default = "NULL"] _id: Nullable<String>,
    #[default = "NULL"] name: Nullable<String>,
    #[default = "NULL"] r#type: Nullable<String>,
    #[default = "NULL"] tags: Nullable<List>,
    #[default = "NULL"] metadata: Nullable<List>,
) -> List {
    let filter = FilterArgs::default()
        .set_rid(_id)
        .set_name(name)
        .set_kind(r#type)
        .set_tags(tags)
        .set_metadata(metadata);

    let assets = db
        .find_assets(filter.into())
        .expect("could not retrieve `Asset`s");

    assets.into_iter().collect()
}

#[extendr]
fn add_asset(
    db: &mut Database,
    file: String,
    #[default = "NULL"] name: Nullable<String>,
    #[default = "NULL"] r#type: Nullable<String>,
    #[default = "NULL"] description: Nullable<String>,
    #[default = "NULL"] tags: Nullable<List>,
    #[default = "NULL"] metadata: Nullable<List>,
) -> String {
    let file = ResourcePath::new(file.into()).expect("invalid file");
    let mut asset = AssetBuilder::new().set_path(file);

    if let Nullable::NotNull(value) = name {
        asset.set_name(value);
    }

    if let Nullable::NotNull(value) = r#type {
        asset.set_kind(value);
    }

    if let Nullable::NotNull(value) = description {
        asset.set_description(value);
    }

    if let Nullable::NotNull(value) = tags {
        asset.set_tags(list_to_string_hashset(value).into_iter().collect());
    }

    if let Nullable::NotNull(value) = metadata {
        asset.set_metadata(list_to_metadata(value));
    }

    let path = db
        .add_asset(asset.into())
        .expect("could not create `Asset`");

    path.to_str()
        .expect("could not convert path to str")
        .to_string()
}

// ***************
// *** exports ***
// ***************

extendr_module! {
    mod database;
    fn database;
    fn find_container;
    fn find_containers;
    fn find_asset;
    fn find_assets;
    fn add_asset;
    impl Database;
    use container;
    use asset;
}

// ***************
// *** helpers ***
// ***************

fn db_server_path() -> Result<PathBuf> {
    #[allow(unused_mut)] // must be `mut` for Windows
    let mut exe = PathBuf::from(format!("thot-local-database-{CURRENT_PLATFORM:}"));
    #[cfg(target_os = "windows")]
    exe.set_extension("exe");

    let exe = exe
        .to_str()
        .expect("could not converst executable path to str");

    let path = R!(r#"system.file({{ exe }}, package = "thot", mustWork = TRUE)"#)
        .map_err(|err| Error::Other(format!("{err:?}")))?
        .as_str()
        .expect("could not convert `system.file` call to str");

    Ok(PathBuf::from(path))
}

fn list_to_string_hashset(list: List) -> HashSet<String> {
    list.values()
        .map(|tag| tag.as_str().expect("invalid string value").to_string())
        .collect()
}

fn list_to_metadata(list: List) -> Metadata {
    let mut metadata = Metadata::new();
    for (key, value) in list.iter() {
        if key.is_empty() {
            panic!("metadata keys cannot be empty");
        }

        let value =
            functions::robj_to_value(value).expect("could not convert value to serde_json::Value");

        metadata.insert(key.to_string(), value);
    }

    metadata
}

// --------------
// --- Filter ---
// --------------

struct FilterArgs {
    rid: Nullable<String>,
    name: Nullable<String>,
    kind: Nullable<String>,
    tags: Nullable<List>,
    metadata: Nullable<List>,
}

impl FilterArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_rid(mut self, value: Nullable<String>) -> Self {
        self.rid = value;
        self
    }

    pub fn set_name(mut self, value: Nullable<String>) -> Self {
        self.name = value;
        self
    }

    pub fn set_kind(mut self, value: Nullable<String>) -> Self {
        self.kind = value;
        self
    }

    pub fn set_tags(mut self, value: Nullable<List>) -> Self {
        self.tags = value;
        self
    }

    pub fn set_metadata(mut self, value: Nullable<List>) -> Self {
        self.metadata = value;
        self
    }
}

impl Default for FilterArgs {
    fn default() -> Self {
        Self {
            rid: Nullable::Null,
            name: Nullable::Null,
            kind: Nullable::Null,
            tags: Nullable::Null,
            metadata: Nullable::Null,
        }
    }
}

impl Into<StdFilter> for FilterArgs {
    fn into(self) -> StdFilter {
        let rid = match self.rid {
            Nullable::Null => None,
            Nullable::NotNull(value) => Some(ResourceId::from_str(&value).expect("invalid id")),
        };

        let name = match self.name {
            Nullable::Null => None,
            Nullable::NotNull(value) => Some(Some(value)),
        };

        let kind = match self.kind {
            Nullable::Null => None,
            Nullable::NotNull(value) => Some(Some(value)),
        };

        let tags = match self.tags {
            Nullable::Null => None,
            Nullable::NotNull(tags) => Some(list_to_string_hashset(tags)),
        };

        let metadata = match self.metadata {
            Nullable::Null => None,
            Nullable::NotNull(metadata) => Some(list_to_metadata(metadata)),
        };

        StdFilter {
            rid,
            name,
            kind,
            tags,
            metadata,
        }
    }
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
