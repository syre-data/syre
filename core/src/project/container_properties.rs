//! Container properties.
use super::Metadata;
use crate::types::Creator;
use chrono::prelude::*;
use serde_json::Value as JsValue;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerProperties {
    created: DateTime<Utc>,
    pub creator: Creator,

    pub name: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Metadata,
}

impl ContainerProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            created: Utc::now(),
            creator: Creator::User(None),

            name: name.into(),
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

// ***************
// *** Builder ***
// ***************

#[derive(Default)]
pub struct Builder {
    created: Option<DateTime<Utc>>,
    creator: Creator,
    name: String,
    kind: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    metadata: Metadata,
}

impl Builder {
    pub fn set_created(&mut self, value: DateTime<Utc>) -> &mut Self {
        self.created = Some(value);
        self
    }

    pub fn clear_created(&mut self) -> &mut Self {
        self.created = None;
        self
    }

    pub fn set_creator(&mut self, value: Creator) -> &mut Self {
        self.creator = value;
        self
    }

    pub fn set_name(&mut self, value: String) -> &mut Self {
        self.name = value;
        self
    }

    pub fn set_kind(&mut self, value: String) -> &mut Self {
        self.kind = Some(value);
        self
    }

    pub fn clear_kind(&mut self) -> &mut Self {
        self.kind = None;
        self
    }

    pub fn set_description(&mut self, value: String) -> &mut Self {
        self.description = Some(value);
        self
    }

    pub fn clear_description(&mut self) -> &mut Self {
        self.description = None;
        self
    }

    pub fn set_tags(&mut self, value: Vec<String>) -> &mut Self {
        self.tags = value;
        self
    }

    pub fn clear_tags(&mut self) -> &mut Self {
        self.tags.clear();
        self
    }

    pub fn add_tag(&mut self, value: impl Into<String>) -> &mut Self {
        let value = value.into();
        if self.tags.iter().filter(|tag| tag == &&value).count() == 0 {
            self.tags.push(value);
        }

        self
    }

    pub fn remove_tag(&mut self, value: impl Into<String>) -> &mut Self {
        let value = value.into();
        self.tags.retain(|tag| tag != &value);
        self
    }

    pub fn set_metadata(&mut self, value: Metadata) -> &mut Self {
        self.metadata = value;
        self
    }

    pub fn clear_metadata(&mut self) -> &mut Self {
        self.metadata.clear();
        self
    }

    pub fn set_metadatum(
        &mut self,
        key: impl Into<String>,
        value: impl Into<JsValue>,
    ) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn remove_metadatum(&mut self, key: impl Into<String>) -> &mut Self {
        self.metadata.remove(&key.into());
        self
    }
}

impl Builder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            created: None,
            creator: Creator::default(),
            name: name.into(),
            kind: None,
            description: None,
            tags: Vec::default(),
            metadata: Metadata::default(),
        }
    }
}

impl Into<ContainerProperties> for Builder {
    fn into(self) -> ContainerProperties {
        ContainerProperties {
            created: self.created.unwrap_or_else(|| Utc::now()),
            creator: self.creator,
            name: self.name,
            kind: self.kind,
            description: self.description,
            tags: self.tags,
            metadata: self.metadata,
        }
    }
}
