//! API interface functionality.
use crate::db::resources::asset::Asset;
use crate::db::resources::container::Container;
use crate::db::resources::search_filter::StandardPropertiesSearchFilter as StdPropsFilter;
use crate::Result;
use std::path::PathBuf;

pub trait Database {
    /// Returns the root of the database.
    fn root(&self) -> PathBuf;

    /// Finds a single Container matching the search fitler.
    fn find_container(&self, search: StdPropsFilter) -> Option<Container>;

    /// Finds all Containers matching the search filter.
    fn find_containers(&self, search: StdPropsFilter) -> Vec<Container>;

    /// Finds a single Asset matching the search filter.
    fn find_asset(&self, search: StdPropsFilter) -> Option<Asset>;

    /// Finds all Assets matching the search filter.
    fn find_assets(&self, search: StdPropsFilter) -> Vec<Asset>;

    /// Adds an Asset to the database.
    fn add_asset(&mut self, asset: Asset) -> Result;

    // @remove
    // /// Returns whether the datbase is running in development mode or not.
    // fn dev_mode() -> bool;
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
