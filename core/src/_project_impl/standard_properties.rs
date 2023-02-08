//! Standard properties associated with other resources.
use super::Metadata;
use crate::types::Creator;
use crate::types::{ResourceId, UserPermissions};
use chrono::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ***************************
// *** Standard Properties ***
// ***************************

/// Standard resource properties.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct StandardProperties {
    created: DateTime<Utc>,
    pub creator: Creator,

    #[cfg_attr(feature = "serde", serde(default))]
    pub permissions: HashMap<ResourceId, UserPermissions>,

    pub name: Option<String>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Metadata,
}

impl StandardProperties {
    pub fn new() -> Self {
        Self {
            created: Utc::now(),
            creator: Creator::User(None),
            permissions: HashMap::new(),

            name: None,
            kind: None,
            description: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }
}

impl Default for StandardProperties {
    fn default() -> Self {
        StandardProperties {
            created: Utc::now(),
            creator: Creator::User(None),
            permissions: HashMap::new(),

            name: None,
            kind: None,
            description: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
#[path = "standard_properties_test.rs"]
mod standard_properties_test;
