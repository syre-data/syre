use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use syre_core::{
    project::{AnalysisAssociation, Container, ContainerProperties},
    types::{ResourceId, UserId, UserPermissions},
};

/// Properties for a Container.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct StoredProperties {
    pub rid: ResourceId,
    pub properties: ContainerProperties,
    pub analyses: Vec<AnalysisAssociation>,
}

impl From<Container> for StoredProperties {
    fn from(container: Container) -> Self {
        Self {
            rid: container.rid().clone(),
            properties: container.properties,
            analyses: container.analyses,
        }
    }
}

/// Settings for a Container
#[derive(PartialEq, Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    pub creator: Option<UserId>,
    pub created: DateTime<Utc>,
    pub permissions: Vec<UserPermissions>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            creator: None,
            created: Utc::now(),
            permissions: vec![],
        }
    }
}
