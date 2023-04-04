use extendr_api::prelude::*;
use thot_core::types::ResourceId;
use std::path::PathBuf;

/// Return whether Thot is running in development mode.
/// @export
#[extendr]
fn create_database(dev_root: Option<PathBuf>) -> Result<Datab {
    true

}


/// A Thot Database.
/// @export
#[pyclass]
pub struct Database {
    root: ResourceId,
    root_path: PathBuf,
    // db: DbClient,
}

impl Database {

}

#[extendr]
impl Database {

}

extendr_module! {
    mod database;
    fn create_database;
    impl Database;
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
