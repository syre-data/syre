//! Implementation of `Container` related functionality.
use super::super::Database;
use crate::command::container::AddAssetInfo;
use crate::command::container::{
    AddAssetsArgs, BulkUpdateContainerPropertiesArgs, BulkUpdateScriptAssociationsArgs,
    ContainerPropertiesUpdate, ScriptAssociationBulkUpdate, UpdatePropertiesArgs,
    UpdateScriptAssociationsArgs,
};
use crate::command::ContainerCommand;
use crate::Result;
use serde_json::Value as JsValue;
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::db::StandardSearchFilter;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container as CoreContainer, ContainerProperties, RunParameters};
use thot_core::types::ResourceId;
use thot_local::project::asset::AssetBuilder;
use thot_local::project::resources::Container;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_container(&mut self, cmd: ContainerCommand) -> JsValue {
        match cmd {
            ContainerCommand::Get(rid) => {
                let container: Option<CoreContainer> = {
                    if let Some(container) = self.store.get_container(&rid) {
                        Some((*container).clone().into())
                    } else {
                        None
                    }
                };

                serde_json::to_value(container).expect("could not convert `Container` to JSON")
            }

            ContainerCommand::GetWithMetadata(rid) => {
                let container = self.store.get_container_with_metadata(&rid);
                serde_json::to_value(container).expect("could not convert `Container` to JSON")
            }

            ContainerCommand::ByPath(path) => {
                let Some(rid) = self.store.get_path_container(&path) else {
                    let value: Option<CoreContainer> = None;
                    return serde_json::to_value(value).expect("could not convert `None` to JSON");
                };

                let container: Option<CoreContainer> = {
                    if let Some(container) = self.store.get_container(&rid) {
                        Some((*container).clone().into())
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

            ContainerCommand::FindWithMetadata(root, filter) => {
                let containers = self.find_containers_with_metadata(&root, filter);
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

            // @todo: Handle errors.
            ContainerCommand::GetPath(rid) => {
                let path = self.get_container_path(&rid);
                serde_json::to_value(path).expect("could not convert path to JsValue")
            }

            ContainerCommand::Parent(rid) => {
                let parent: Result<Option<CoreContainer>> =
                    self.get_container_parent(&rid).map(|opt| match opt {
                        Some(container) => Some((*container).clone()),
                        None => None,
                    });

                serde_json::to_value(parent).expect("could not convert `Container` to JsValue")
            }

            ContainerCommand::BulkUpdateProperties(BulkUpdateContainerPropertiesArgs {
                rids,
                update,
            }) => {
                let res = self.bulk_update_container_properties(&rids, &update);
                serde_json::to_value(res).unwrap()
            }

            ContainerCommand::BulkUpdateScriptAssociations(BulkUpdateScriptAssociationsArgs {
                containers,
                update,
            }) => {
                let res = self.bulk_update_container_script_associations(&containers, &update);
                serde_json::to_value(res).unwrap()
            }
        }
    }

    /// # Arguments
    /// 1. Root `Container`.
    /// 2. Search filter.
    fn find_containers(
        &self,
        root: &ResourceId,
        filter: StandardSearchFilter,
    ) -> HashSet<CoreContainer> {
        self.store
            .find_containers(&root, filter)
            .into_iter()
            .map(|container| (*container).clone())
            .collect()
    }

    /// # Arguments
    /// 1. Root `Container`.
    /// 2. Search filter.
    fn find_containers_with_metadata(
        &self,
        root: &ResourceId,
        filter: StandardSearchFilter,
    ) -> HashSet<CoreContainer> {
        self.store.find_containers_with_metadata(&root, filter)
    }

    fn update_container_properties(
        &mut self,
        rid: ResourceId,
        properties: ContainerProperties,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Container` does not exist",
            ))
            .into());
        };

        container.properties = properties;
        container.save()?;
        Ok(())
    }

    fn update_container_script_associations(
        &mut self,
        rid: ResourceId,
        associations: ScriptMap,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Script` does not exist",
            ))
            .into());
        };

        container.scripts = associations;
        container.save()?;
        Ok(())
    }

    fn container_add_assets(
        &mut self,
        container: &ResourceId,
        assets: Vec<AddAssetInfo>,
    ) -> Result<HashSet<ResourceId>> {
        let Some(container) = self.store.get_container_mut(&container) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Script` does not exist",
            ))
            .into());
        };

        // @todo: Ensure file is not an Asset with the Container already.
        let mut asset_ids = HashSet::with_capacity(assets.len());
        for AddAssetInfo {
            path,
            action,
            bucket,
        } in assets
        {
            // create asset
            let mut asset = AssetBuilder::new(path);
            asset.set_container(container.base_path().into());
            if let Some(bucket) = bucket {
                asset.set_bucket(bucket);
            }

            let asset = asset.create(action)?;
            asset_ids.insert(asset.rid.clone());
            container.insert_asset(asset);
        }

        container.save()?;
        let cid = container.rid.clone();

        for aid in asset_ids.iter() {
            self.store.insert_asset(aid.clone(), cid.clone());
        }

        Ok(asset_ids)
    }

    fn get_container_path(&self, container: &ResourceId) -> Option<PathBuf> {
        let Some(container) = self.store.get_container(&container) else {
            return None;
        };

        Some(container.base_path().into())
    }

    fn get_container_parent(&self, rid: &ResourceId) -> Result<Option<&Container>> {
        let Some(graph) = self.store.get_container_graph(rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Container` does not exist",
            ))
            .into());
        };

        let parent = graph.parent(rid)?;
        let Some(parent) = parent else {
            return Ok(None);
        };

        let parent = graph.get(parent).expect("could not get parent `Container`");
        Ok(Some(parent))
    }

    /// Bulk update `Container` properties.
    #[tracing::instrument(skip(self))]
    fn bulk_update_container_properties(
        &mut self,
        containers: &Vec<ResourceId>,
        update: &ContainerPropertiesUpdate,
    ) -> Result {
        for container in containers {
            self.update_container_properties_from_update(container, update)?;
        }

        Ok(())
    }

    /// Update a `Container`'s properties.
    #[tracing::instrument(skip(self))]
    fn update_container_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &ContainerPropertiesUpdate,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Container` does not exist",
            ))
            .into());
        };

        // basic properties
        if let Some(name) = update.name.as_ref() {
            container.properties.name = name.clone();
        }

        if let Some(kind) = update.kind.as_ref() {
            container.properties.kind = kind.clone();
        }

        if let Some(description) = update.description.as_ref() {
            container.properties.description = description.clone();
        }

        // tags
        container
            .properties
            .tags
            .append(&mut update.tags.insert.clone());

        container.properties.tags.sort();
        container.properties.tags.dedup();
        container
            .properties
            .tags
            .retain(|tag| !update.tags.remove.contains(tag));

        // metadata
        container
            .properties
            .metadata
            .extend(update.metadata.insert.clone());

        for key in update.metadata.remove.iter() {
            container.properties.metadata.remove(key);
        }

        container.save()?;
        Ok(())
    }

    fn bulk_update_container_script_associations(
        &mut self,
        containers: &Vec<ResourceId>,
        update: &ScriptAssociationBulkUpdate,
    ) -> Result {
        for rid in containers {
            self.update_container_script_associations_from_update(rid, update)?;
        }

        Ok(())
    }

    /// Update a `Container`'s `ScriptAssociations`.
    ///
    /// # Note
    /// Updates are processed in the following order:
    /// 1. New associations are added.
    /// 2. Present associations are updated.
    /// 3. Associations are removed.
    fn update_container_script_associations_from_update(
        &mut self,
        rid: &ResourceId,
        update: &ScriptAssociationBulkUpdate,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist(
                "`Container` does not exist",
            ))
            .into());
        };

        for assoc in update.add.iter() {
            container.scripts.insert(
                assoc.script.clone(),
                RunParameters {
                    priority: assoc.priority.clone(),
                    autorun: assoc.autorun.clone(),
                },
            );
        }

        for u in update.update.iter() {
            let Some(script) = container.scripts.get_mut(&u.script) else {
                continue;
            };

            if let Some(priority) = u.priority.as_ref() {
                script.priority = priority.clone();
            }

            if let Some(autorun) = u.autorun.as_ref() {
                script.autorun = autorun.clone();
            }
        }

        for script in update.remove.iter() {
            container.scripts.remove(script);
        }

        container.save()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
