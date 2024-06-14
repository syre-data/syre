//! Container related commands.
use super::types::{MetadataAction, TagsAction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::db::StandardSearchFilter;
use syre_core::project::container::AnalysisMap;
use syre_core::project::{AnalysisAssociation, ContainerProperties};
use syre_core::types::ResourceId;

/// Container related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum ContainerCommand {
    /// Retrieves a [`Container`](syre_core::project::Container) by [`ResourceId`].
    Get(ResourceId),

    /// Retrievea a [`Container`](syre_core::project::Container) with inherited metadata by [`ResourceId`].
    GetWithMetadata(ResourceId),

    /// Retrieves a [`Container`](syre_core::project::Container) by its path.
    ByPath(PathBuf),

    /// Retrieves [`Container`](syre_core::project::Container)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),

    /// Retrieves [`Container`](syre_core::project::Container)s based on a filter.
    /// Lineage is compiled.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    FindWithMetadata(ResourceId, StandardSearchFilter),

    /// Updates a [`Container`](syre_core::project::Container)'s properties.
    UpdateProperties(UpdatePropertiesArgs),

    /// Updates a [`Container`](syre_core::project::Container)'s
    /// [`AnalysisAssociation`](syre_core::project::AnalysisAssociation)s.
    UpdateAnalysisAssociations(UpdateAnalysisAssociationsArgs),

    /// Gets the path of a [`Container`](syre_local::project::resources::Container).
    Path(ResourceId),

    /// Gets the parent of a [`Container`](syre_core::project::Container).
    Parent(ResourceId),

    /// Update multiple [`Container`](syre_core::project::Container)s' properties.
    BulkUpdateProperties(BulkUpdatePropertiesArgs),

    /// Update multiple `Container`s `AnalysisAssociations`.
    BulkUpdateAnalysisAssociations(BulkUpdateAnalysisAssociationsArgs),
}

// *****************
// *** Arguments ***
// *****************

/// Arguments for updating a resource's [`StandardProperties`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePropertiesArgs {
    pub rid: ResourceId,
    pub properties: ContainerProperties,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PropertiesUpdate {
    pub name: Option<String>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: PropertiesUpdate,
}

/// Arguments for updating a [`Container`](syre_core::project::Container)'s
/// [`AnalysisAssociation`](syre_core::project::AnalysisAssociation)s.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateAnalysisAssociationsArgs {
    pub rid: ResourceId,
    pub associations: AnalysisMap,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BulkUpdateAnalysisAssociationsArgs {
    pub containers: Vec<ResourceId>,
    pub update: AnalysisAssociationBulkUpdate,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AnalysisAssociationBulkUpdate {
    pub add: Vec<AnalysisAssociation>,
    pub remove: Vec<ResourceId>,
    pub update: Vec<RunParametersUpdate>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunParametersUpdate {
    pub analysis: ResourceId,
    pub autorun: Option<bool>,
    pub priority: Option<i32>,
}

impl RunParametersUpdate {
    pub fn new(analysis: ResourceId) -> Self {
        Self {
            analysis,
            autorun: None,
            priority: None,
        }
    }
}
