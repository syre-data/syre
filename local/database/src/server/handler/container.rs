//! Implementation of `Container` related functionality.
use super::super::Database;
use crate::command::container::AddAssetInfo;
use crate::command::container::{
    AddAssetsArgs, NewChildArgs, UpdatePropertiesArgs, UpdateScriptAssociationsArgs,
};
use crate::command::ContainerCommand;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::LocalSettings;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thot_core::db::StandardSearchFilter;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container as CoreContainer, StandardProperties};
use thot_core::types::ResourceId;
use thot_local::project::asset::AssetBuilder;
use thot_local::project::container;
use thot_local::project::resources::Container as LocalContainer;

impl Database {
    pub fn handle_command_container(&mut self, cmd: ContainerCommand) -> JsValue {
        match cmd {
            ContainerCommand::LoadTree(root) => {
                let container = self.load_container_tree(&root);
                serde_json::to_value(container).expect("could not convert `Container` into JsValue")
            }

            ContainerCommand::Load(path) => {
                let container = self.load_container(&path);
                serde_json::to_value(container).expect("could not convert `Container` to JsValue")
            }

            ContainerCommand::Get(rid) => {
                let container: Option<CoreContainer> = {
                    if let Some(container) = self.store.get_container(&rid) {
                        let container = container.lock().expect("could not lock `Container`");
                        Some(container.clone().into())
                    } else {
                        None
                    }
                };

                serde_json::to_value(container).expect("could not convert `Container` to JSON")
            }

            ContainerCommand::Find(root, filter) => {
                let containers = self.find_containers(&root, filter);
                serde_json::to_value(containers).expect("could not convert `Container`s to JSON")
            }

            ContainerCommand::UpdateProperties(UpdatePropertiesArgs { rid, properties }) => {
                let res = self.update_container_properties(rid, properties);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            ContainerCommand::UpdateScriptAssociations(UpdateScriptAssociationsArgs {
                rid,
                associations,
            }) => {
                let res = self.update_container_script_associations(rid, associations);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            ContainerCommand::AddAssets(AddAssetsArgs { container, assets }) => {
                let asset_rids = self.container_add_assets(&container, assets);
                serde_json::to_value(asset_rids)
                    .expect("could not convert `Asset` `ResourceId`s to JSON")
            }
            ContainerCommand::NewChild(NewChildArgs { name, parent }) => {
                let child = self.container_new_child(&parent, name);
                serde_json::to_value(child).expect("could not convert child `Container` to JsValue")
            }

            // @todo: Handle errors.
            ContainerCommand::GetPath(rid) => {
                let path = self.get_container_path(&rid);
                serde_json::to_value(path).expect("could not convert path to JsValue")
            }
        }
    }

    /// Loads a [`Container`](LocalContainer) tree from settings.
    ///
    /// # Returns
    /// Reference to the root [`Container`](LocalContainer).
    fn load_container_tree(&mut self, path: &Path) -> Result<CoreContainer> {
        if let Some(cid) = self.store.get_path_container(path) {
            if let Some(container) = self.store.get_container(cid) {
                // already loaded
                let container = container.lock().expect("could not lock `Container`");
                return Ok(container.clone().into());
            }
        }

        let mut container = LocalContainer::load(path)?;
        container
            .load_children(true)
            .expect("could not load children");

        let container_val = container.clone().into();

        self.store
            .insert_container_tree(Arc::new(Mutex::new(container)))?;

        Ok(container_val)
    }

    /// Loads a single [`Container`](LocalContainer) from settings.
    ///
    /// # Returns
    /// Reference to the loaded [`Container`](LocalContainer).
    fn load_container(&mut self, path: &Path) -> Result<CoreContainer> {
        if let Some(cid) = self.store.get_path_container(&path) {
            // already loaded
            if let Some(container) = self.store.get_container(&cid) {
                let container = container.lock().expect("could not lock `Container`");
                return Ok(container.clone().into());
            }
        }

        let container = LocalContainer::load(path)?;
        let container_val = container.clone().into();
        self.store.insert_container(container)?;
        Ok(container_val)
    }

    /// # Arguments
    /// 1. Root `Container`.
    /// 2. Search filter.
    fn find_containers(
        &self,
        root: &ResourceId,
        filter: StandardSearchFilter,
    ) -> HashSet<CoreContainer> {
        let containers = self.store.find_containers(&root, filter);
        let containers = containers
            .values()
            .map(|container| {
                let container = container.lock().expect("could not lock `Container`");
                container.clone().into()
            })
            .collect();

        containers
    }

    fn update_container_properties(
        &mut self,
        rid: ResourceId,
        properties: StandardProperties,
    ) -> Result {
        let Some(container) = self.store.get_container(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist".to_string())).into());
        };

        let mut container = container.lock().expect("could not lock `Container`");
        container.properties = properties;
        container.save()?;
        Ok(())
    }

    fn update_container_script_associations(
        &mut self,
        rid: ResourceId,
        associations: ScriptMap,
    ) -> Result {
        let Some(container) = self.store.get_container(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist".to_string())).into());
        };

        let mut container = container.lock().expect("could not lock `Container`");
        container.scripts = associations;
        container.save()?;
        Ok(())
    }

    fn container_add_assets(
        &mut self,
        container: &ResourceId,
        assets: Vec<AddAssetInfo>,
    ) -> Result<Vec<ResourceId>> {
        let Some(container) = self.store.get_container(&container) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist".to_string())).into());
        };

        let mut container = container.lock().expect("could not lock `Container`");
        let container_path = container.base_path()?;

        // @todo: Ensure file is not an Asset with the Container already.
        let mut asset_rids: Vec<thot_core::types::ResourceId> = Vec::with_capacity(assets.len());
        for AddAssetInfo {
            path,
            action,
            bucket,
        } in assets
        {
            // create asset
            let mut asset = AssetBuilder::new(path);
            asset.set_container(container_path.clone());
            if let Some(bucket) = bucket {
                asset.set_bucket(bucket);
            }

            let asset = asset.create(action)?;
            asset_rids.push(asset.rid.clone());

            let aid = asset.rid.clone();
            container.insert_asset(asset)?;
            container.save()?;

            self.store.insert_asset(aid, container.rid.clone());
        }

        Ok(asset_rids)
    }

    fn container_new_child(&mut self, parent: &ResourceId, name: String) -> Result<CoreContainer> {
        let Some(parent) = self.store.get_container(&parent) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist".to_string())).into());
        };

        let mut parent = parent.lock().expect("could not lock `Container`");

        // create child
        // @todo: Ensure unique and valid path.
        let child_path = parent.base_path()?.join(&name);
        let cid = container::new(&child_path)?;

        parent.register_child(cid.clone());
        parent.save()?;

        // insert into store
        self.load_container(&child_path)?;
        let Some(child) = self.store.get_container(&cid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("child `Container` could not load".to_string())).into());
        };

        let mut child = child.lock().expect("could not lock `Container`");
        child.properties.name = Some(name);
        child.save()?;

        Ok(child.clone().into())
    }

    fn get_container_path(&self, container: &ResourceId) -> Result<Option<PathBuf>> {
        let Some(container) = self.store.get_container(&container) else {
            return Ok(None);
        };

        let container = container.lock().expect("could not lock `Container`");
        let path = container.base_path()?;
        Ok(Some(path))
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
