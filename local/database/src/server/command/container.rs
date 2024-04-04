//! Implementation of `Container` related functionality.
use super::super::Database;
use crate::command::container::{
    AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs, BulkUpdatePropertiesArgs,
    PropertiesUpdate, UpdateAnalysisAssociationsArgs, UpdatePropertiesArgs,
};
use crate::command::ContainerCommand;
use crate::error::server::{
    Rename as RenameError, Update as UpdateError, UpdateContainer as UpdateContainerError,
};
use crate::Result;
use serde_json::Value as JsValue;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::result::Result as StdResult;
use syre_core::db::StandardSearchFilter;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::container::AnalysisMap;
use syre_core::project::{Container as CoreContainer, ContainerProperties, RunParameters};
use syre_core::types::ResourceId;
use syre_local::common;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_container(&mut self, cmd: ContainerCommand) -> JsValue {
        match cmd {
            ContainerCommand::Get(container) => {
                serde_json::to_value(self.handle_container_get(&container)).unwrap()
            }

            ContainerCommand::GetWithMetadata(rid) => {
                let container = self.object_store.get_container_with_metadata(&rid);
                serde_json::to_value(container).unwrap()
            }

            ContainerCommand::ByPath(path) => {
                serde_json::to_value(self.handle_container_by_path(&path)).unwrap()
            }

            ContainerCommand::Find(root, filter) => {
                let containers = self.handle_container_find(&root, filter);
                serde_json::to_value(containers).unwrap()
            }

            ContainerCommand::FindWithMetadata(root, filter) => {
                let containers = self.handle_container_find_with_metadata(&root, filter);
                serde_json::to_value(containers).unwrap()
            }

            ContainerCommand::UpdateProperties(UpdatePropertiesArgs { rid, properties }) => {
                let res = self.handle_container_update_properties(rid, properties);
                serde_json::to_value(res).unwrap()
            }

            ContainerCommand::UpdateAnalysisAssociations(UpdateAnalysisAssociationsArgs {
                rid,
                associations,
            }) => {
                let res = self.handle_container_update_analysis_associations(&rid, associations);
                serde_json::to_value(res).unwrap()
            }

            ContainerCommand::Path(rid) => {
                let path = self.handle_container_path(&rid);
                serde_json::to_value(path).unwrap()
            }

            ContainerCommand::Parent(rid) => {
                serde_json::to_value(self.handle_container_parent(&rid)).unwrap()
            }

            ContainerCommand::BulkUpdateProperties(BulkUpdatePropertiesArgs { rids, update }) => {
                let res = self.handle_container_bulk_update_properties(&rids, &update);
                serde_json::to_value(res).unwrap()
            }

            ContainerCommand::BulkUpdateAnalysisAssociations(
                BulkUpdateAnalysisAssociationsArgs { containers, update },
            ) => {
                let res =
                    self.handle_container_bulk_update_analysis_associations(&containers, &update);
                serde_json::to_value(res).unwrap()
            }
        }
    }

    fn handle_container_get(&self, container: &ResourceId) -> Option<CoreContainer> {
        if let Some(container) = self.object_store.get_container(container) {
            Some((*container).clone().into())
        } else {
            None
        }
    }

    fn handle_container_by_path(&self, path: &PathBuf) -> Option<CoreContainer> {
        let Some(container) = self
            .object_store
            .get_path_container_canonical(path)
            .unwrap()
        else {
            return None;
        };

        if let Some(container) = self.object_store.get_container(&container) {
            Some((*container).clone().into())
        } else {
            None
        }
    }

    /// # Arguments
    /// 1. Root `Container`.
    /// 2. Search filter.
    fn handle_container_find(
        &self,
        root: &ResourceId,
        filter: StandardSearchFilter,
    ) -> HashSet<CoreContainer> {
        self.object_store
            .find_containers(&root, filter)
            .into_iter()
            .map(|container| (*container).clone())
            .collect()
    }

    /// # Arguments
    /// 1. Root `Container`.
    /// 2. Search filter.
    fn handle_container_find_with_metadata(
        &self,
        root: &ResourceId,
        filter: StandardSearchFilter,
    ) -> HashSet<CoreContainer> {
        self.object_store
            .find_containers_with_metadata(&root, filter)
    }

    #[tracing::instrument(skip(self))]
    fn handle_container_update_properties(
        &mut self,
        rid: ResourceId,
        properties: ContainerProperties,
    ) -> StdResult<(), UpdateContainerError> {
        let Some(container) = self.object_store.get_container(&rid) else {
            return Err(UpdateContainerError::ResourceNotFound);
        };

        let graph = self.object_store.get_graph_of_container(&rid).unwrap();
        if properties.name != container.properties.name && &container.rid != graph.root() {
            if let Err(err) = self.rename_container_folder(&rid, &properties.name) {
                return Err(UpdateContainerError::Rename(err));
            }
        }

        let container = self.object_store.get_container_mut(&rid).unwrap();
        container.properties = properties;
        if let Err(err) = container.save() {
            return Err(UpdateContainerError::Save(err));
        }

        if let Err(err) = self
            .data_store
            .container()
            .update(container.rid.clone(), container.properties.clone().into())
        {
            tracing::error!(?err);
        }

        Ok(())
    }

    /// Renames a Container's folder.
    ///
    /// # Side effects
    /// + Updates the Container's graph.
    /// + Updates the store's mappings.
    ///
    /// # Errors
    /// + If the new name results in a name clash between sibling Containers.
    #[tracing::instrument(skip(self, name))]
    fn rename_container_folder(
        &mut self,
        container: &ResourceId,
        name: impl Into<String>,
    ) -> StdResult<(), RenameError> {
        let container_path = {
            let name: String = name.into();
            let Some(container) = self.object_store.get_container(container) else {
                return Err(RenameError::ResourceNotFound);
            };

            let graph = self
                .object_store
                .get_graph_of_container(&container.rid)
                .unwrap();

            // get siblings to check for name clash
            let siblings = graph.siblings(&container.rid).unwrap();
            let sibling_names: Vec<String> = siblings
                .iter()
                .map(|sid| {
                    let sibling = graph.get(sid).unwrap();
                    sibling.properties.name.clone()
                })
                .collect();

            if sibling_names.contains(&name) {
                return Err(RenameError::NameConflict);
            }

            // create unique sanitized path
            let mut container_path = container.base_path().parent().unwrap().to_path_buf();

            container_path.push(common::sanitize_file_path(name));
            let container_path = common::unique_file_name(PathBuf::from(container_path)).unwrap();

            // rename folder
            match fs::rename(container.base_path(), &container_path) {
                Ok(_) => {}
                Err(err) => {
                    let from = container.base_path();
                    tracing::error!(?err, ?from, ?container_path);
                    return Err(RenameError::Rename(err.kind()));
                }
            };

            container_path
        };

        // update descendant's path
        self.object_store
            .update_subgraph_path(container, container_path)?;

        Ok(())
    }

    fn handle_container_update_analysis_associations(
        &mut self,
        container: &ResourceId,
        associations: AnalysisMap,
    ) -> StdResult<(), UpdateError> {
        let Some(container) = self.object_store.get_container_mut(container) else {
            return Err(UpdateError::ResourceNotFound);
        };

        container.analyses = associations;
        container.save()?;
        Ok(())
    }

    fn handle_container_path(&self, container: &ResourceId) -> Option<PathBuf> {
        let Some(container) = self.object_store.get_container(&container) else {
            return None;
        };

        Some(container.base_path().into())
    }

    fn handle_container_parent(
        &self,
        rid: &ResourceId,
    ) -> StdResult<Option<CoreContainer>, ResourceError> {
        let Some(graph) = self.object_store.get_graph_of_container(rid) else {
            return Err(ResourceError::does_not_exist("Container does not exist"));
        };

        let parent = graph.parent(rid)?;
        let Some(parent) = parent else {
            return Ok(None);
        };

        let parent = graph.get(parent).unwrap();
        Ok(Some((*parent).clone()))
    }

    /// Bulk update `Container` properties.
    fn handle_container_bulk_update_properties(
        &mut self,
        containers: &Vec<ResourceId>,
        update: &PropertiesUpdate,
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
        update: &PropertiesUpdate,
    ) -> Result {
        let Some(container) = self.object_store.get_container_mut(&rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
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
        if let Err(err) = self
            .data_store
            .container()
            .update(container.rid.clone(), container.properties.clone().into())
        {
            tracing::error!(?err);
        }

        Ok(())
    }

    fn handle_container_bulk_update_analysis_associations(
        &mut self,
        containers: &Vec<ResourceId>,
        update: &AnalysisAssociationBulkUpdate,
    ) -> Result {
        // TODO Collect errors
        for rid in containers {
            self.update_container_analysis_associations_from_update(rid, update)?;
        }

        Ok(())
    }

    /// Update a `Container`'s analysis associations.
    ///
    /// # Note
    /// Updates are processed in the following order:
    /// 1. New associations are added.
    /// 2. Present associations are updated.
    /// 3. Associations are removed.
    fn update_container_analysis_associations_from_update(
        &mut self,
        rid: &ResourceId,
        update: &AnalysisAssociationBulkUpdate,
    ) -> Result {
        let Some(container) = self.object_store.get_container_mut(&rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        for assoc in update.add.iter() {
            container.analyses.insert(
                assoc.analysis.clone(),
                RunParameters {
                    priority: assoc.priority.clone(),
                    autorun: assoc.autorun.clone(),
                },
            );
        }

        for u in update.update.iter() {
            let Some(script) = container.analyses.get_mut(&u.analysis) else {
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
            container.analyses.remove(script);
        }

        container.save()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
