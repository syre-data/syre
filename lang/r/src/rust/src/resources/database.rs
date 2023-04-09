use extendr_api::prelude::*;
use std::path::PathBuf;
use thot_core::types::ResourceId;

/// Return whether Thot is running in development mode.
/// @export
#[extendr]
fn database(dev_root: String) -> Result<Database> {
    let dev_root = PathBuf::from(dev_root);
    let db = Database::new(dev_root)?;
    Ok(db)
}

/// A Thot Database.
/// @export
pub struct Database {
    root: ResourceId,
    root_path: PathBuf,
    // db: DbClient,
}

impl Database {
    pub fn new(root_path: PathBuf) -> Result<Self> {
        Ok(Self {
            root: ResourceId::new(),
            root_path,
        })
    }
}

#[extendr]
impl Database {
    pub fn root_path<'a>(&'a self) -> &'a str {
        self.root_path
            .to_str()
            .expect("could not convert root path to str")
    }

    // pub fn root(&self) -> Container {
    // }
}

extendr_module! {
    mod database;
    fn database;
    impl Database;
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
