//! Database functionality.
use current_platform::CURRENT_PLATFORM;
use extendr_api::prelude::*;
use std::ops::Deref;
use std::path::PathBuf;
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::lib_impl::extendr::container;
use thot_core::project::Container;
use thot_core::types::ResourceId;
use thot_lang::{Database as BaseDb, Error, Result};

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
    // REMOVE
    pub fn temp(&self) -> bool {
        true
    }
    // pub fn root(&self) -> Container {
    //     self.0.root().expect("could not get database root `Container`")
    // }
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

// #[extendr]
// fn find_containers(db: Database, #[default = "NULL"] name: Nullable<String>, #[default = "NULL"] r#type: Nullable<String>) -> Vec<Container> {
//     let mut filter = StdFilter::default();
//     filter.name = Some(name);
//     filter.kind = Some(r#type);

//     db.find_containers(filter)
// }

extendr_module! {
    mod database;
    fn database;
    // fn find_containers;
    impl Database;
    use container;
}

// ***************
// *** helpers ***
// ***************

fn db_server_path() -> Result<PathBuf> {
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

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
