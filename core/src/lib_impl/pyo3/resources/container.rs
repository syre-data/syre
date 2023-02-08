//! Project and database Containers.
use crate::db::resources::Container as DbContainer;
use crate::project::Container as CoreContainer;
use crate::types::ResourceId;
use std::collections::HashSet;

impl Into<DbContainer> for CoreContainer {
    fn into(self) -> DbContainer {
        let children = self.children.into_keys().collect::<HashSet<ResourceId>>();
        let assets = self.assets.into_keys().collect::<HashSet<ResourceId>>();

        DbContainer {
            rid: self.rid,
            properties: self.properties.into(),
            children,
            assets,
            parent: None,
        }
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
