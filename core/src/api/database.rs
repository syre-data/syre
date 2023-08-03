//! API interface functionality.
use crate::db::StandardSearchFilter as StdFilter;
use crate::project::{Asset, Container};
use crate::Result;
use std::path::PathBuf;

pub trait Database {
    /// Returns the root of the database.
    fn root(&self) -> PathBuf;

    /// Finds a single Container matching the search fitler.
    fn find_container(&self, search: StdFilter) -> Option<Container>;

    /// Finds all Containers matching the search filter.
    fn find_containers(&self, search: StdFilter) -> Vec<Container>;

    /// Finds a single Asset matching the search filter.
    fn find_asset(&self, search: StdFilter) -> Option<Asset>;

    /// Finds all Assets matching the search filter.
    fn find_assets(&self, search: StdFilter) -> Vec<Asset>;

    /// Adds an Asset to the database.
    fn add_asset(&mut self, asset: Asset) -> Result;
}
