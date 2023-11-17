//! Container related commands.
use super::types::{MetadataAction, TagsAction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::db::StandardSearchFilter;
use thot_core::project::container::ScriptMap;
use thot_core::project::{ContainerProperties, ScriptAssociation};
use thot_core::types::ResourceId;

/// Container related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum ContainerCommand {
    /// Retrieves a [`Container`](thot_core::project::Container) by [`ResourceId`].
    Get(ResourceId),

    /// Retrievea a [`Container`](thot_core::project::Container) with inherited metadata by [`ResourceId`].
    GetWithMetadata(ResourceId),

    /// Retrieves a [`Container`](thot_core::project::Container) by its path.
    ByPath(PathBuf),

    /// Retrieves [`Container`](thot_core::project::Container)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),

    /// Retrieves [`Container`](thot_core::project::Container)s based on a filter.
    /// Lineage is compiled.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    FindWithMetadata(ResourceId, StandardSearchFilter),

    /// Updates a [`Container`](thot_core::project::Container)'s properties.
    UpdateProperties(UpdatePropertiesArgs),

    /// Updates a [`Container`](thot_core::project::Container)'s
    /// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
    UpdateScriptAssociations(UpdateScriptAssociationsArgs),

    /// Gets the path of a [`Container`](thot_local::project::resources::Container).
    Path(ResourceId),

    /// Gets the parent of a [`Container`](thot_core::project::Container).
    Parent(ResourceId),

    /// Update multiple [`Container`](thot_core::project::Container)s' properties.
    BulkUpdateProperties(BulkUpdateContainerPropertiesArgs),

    /// Update multiple `Container`s `ScriptAssociations`.
    BulkUpdateScriptAssociations(BulkUpdateScriptAssociationsArgs),
}

// *****************
// *** Arguments ***
// *****************

/// Arguments for updating a resource's [`StandardProperties`].
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdatePropertiesArgs {
    pub rid: ResourceId,
    pub properties: ContainerProperties,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ContainerPropertiesUpdate {
    pub name: Option<String>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkUpdateContainerPropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: ContainerPropertiesUpdate,
}

/// Arguments for updating a [`Container`](thot_core::project::Container)'s
/// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateScriptAssociationsArgs {
    pub rid: ResourceId,
    pub associations: ScriptMap,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkUpdateScriptAssociationsArgs {
    pub containers: Vec<ResourceId>,
    pub update: ScriptAssociationBulkUpdate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptAssociationBulkUpdate {
    pub add: Vec<ScriptAssociation>,
    pub remove: Vec<ResourceId>,
    pub update: Vec<RunParametersUpdate>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunParametersUpdate {
    pub script: ResourceId,
    pub autorun: Option<bool>,
    pub priority: Option<i32>,
}
