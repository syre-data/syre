//! Implementation of `Container` related functionality.
use super::super::Database;
use crate::command::container::{
    AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs, BulkUpdatePropertiesArgs,
    PropertiesUpdate, UpdateAnalysisAssociationsArgs, UpdatePropertiesArgs,
};
use crate::command::ContainerCommand;
use crate::Result;
use serde_json::Value as JsValue;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use syre_core::db::StandardSearchFilter;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::container::AnalysisMap;
use syre_core::project::{Container as CoreContainer, ContainerProperties, RunParameters};
use syre_core::types::ResourceId;
use syre_local::common;
use syre_local::error::ContainerError;
use syre_local::error::Error as LocalError;
use syre_local::project::resources::Container;

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
                let Some(rid) = self.store.get_path_container_canonical(&path).unwrap() else {
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

            ContainerCommand::UpdateAnalysisAssociations(UpdateAnalysisAssociationsArgs {
                rid,
                associations,
            }) => {
                let res = self.update_container_script_associations(rid, associations);
                serde_json::to_value(res).expect("could not convert result to JSON")
            }

            // TODO Handle errors.
            ContainerCommand::Path(rid) => {
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

            ContainerCommand::BulkUpdateProperties(BulkUpdatePropertiesArgs { rids, update }) => {
                let res = self.bulk_update_container_properties(&rids, &update);
                serde_json::to_value(res).unwrap()
            }

            ContainerCommand::BulkUpdateAnalysisAssociations(
                BulkUpdateAnalysisAssociationsArgs { containers, update },
            ) => {
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

    #[tracing::instrument(skip(self))]
    fn update_container_properties(
        &mut self,
        rid: ResourceId,
        properties: ContainerProperties,
    ) -> Result {
        let Some(container) = self.store.get_container(&rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        let graph = self.store.get_graph_of_container(&rid).unwrap();
        if properties.name != container.properties.name && &container.rid != graph.root() {
            self.rename_container_folder(&rid, &properties.name)?;
        }

        let container = self
            .store
            .get_container_mut(&rid)
            .expect("Container no longer exists");

        container.properties = properties;
        container.save()?;
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
    ) -> Result {
        let container_path = {
            let name: String = name.into();
            let Some(container) = self.store.get_container(container) else {
                return Err(CoreError::Resource(ResourceError::does_not_exist(
                    "`Container` does not exist",
                ))
                .into());
            };
            let graph = self.store.get_graph_of_container(&container.rid).unwrap();

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
                return Err(
                    LocalError::ContainerError(ContainerError::ContainerNameConflict).into(),
                );
            }

            // create unique sanitized path
            let mut container_path = container
                .base_path()
                .parent()
                .expect("invalid path")
                .to_path_buf();

            container_path.push(common::sanitize_file_path(name));
            let container_path = common::unique_file_name(PathBuf::from(container_path)).unwrap();

            // rename folder
            match fs::rename(container.base_path(), &container_path) {
                Ok(_) => {}
                Err(err) => {
                    let from = container.base_path();
                    tracing::debug!(?err, ?from, ?container_path);
                    return Err(LocalError::from(err).into());
                }
            };

            container_path
        };

        // update descendant's path
        self.store.update_subgraph_path(container, container_path)?;
        Ok(())
    }

    fn update_container_script_associations(
        &mut self,
        rid: ResourceId,
        associations: AnalysisMap,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Script` does not exist",
            ))
            .into());
        };

        container.analyses = associations;
        container.save()?;
        Ok(())
    }

    fn get_container_path(&self, container: &ResourceId) -> Option<PathBuf> {
        let Some(container) = self.store.get_container(&container) else {
            return None;
        };

        Some(container.base_path().into())
    }

    fn get_container_parent(&self, rid: &ResourceId) -> Result<Option<&Container>> {
        let Some(graph) = self.store.get_graph_of_container(rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
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
        let Some(container) = self.store.get_container_mut(&rid) else {
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
        Ok(())
    }

    fn bulk_update_container_script_associations(
        &mut self,
        containers: &Vec<ResourceId>,
        update: &AnalysisAssociationBulkUpdate,
    ) -> Result {
        // TODO Collect errors
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
        update: &AnalysisAssociationBulkUpdate,
    ) -> Result {
        let Some(container) = self.store.get_container_mut(&rid) else {
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
