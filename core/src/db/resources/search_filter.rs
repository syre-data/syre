//! Search filter functionality.
use super::{StandardObject, StandardProperties};
use crate::types::ResourceId;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// *************
// *** Trait ***
// *************

pub trait SearchFilter<T> {
    /// Returns `true` if the object matches the filter,
    /// otherwise `false`.
    fn matches(&self, obj: &T) -> bool;
}

// ***********************
// *** Standard Filter ***
// ***********************

/// Search filter for all properties.
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct StandardSearchFilter {
    pub rid: Option<ResourceId>,
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub tags: Option<HashSet<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl StandardSearchFilter {
    /// Creates a new search filter with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            rid: None,
            name: None,
            kind: None,
            tags: None,
            metadata: None,
        }
    }

    pub fn from_filters(
        id_filter: Option<ResourceIdSearchFilter>,
        props_filter: Option<StandardPropertiesSearchFilter>,
    ) -> Self {
        let mut filter = Self::new();
        // id filter
        if let Some(id_filter) = id_filter {
            filter.rid = id_filter.rid;
        }

        // props filter
        if let Some(props_filter) = props_filter {
            filter.name = props_filter.name;
            filter.kind = props_filter.kind;
            filter.tags = props_filter.tags;
            filter.metadata = props_filter.metadata;
        }

        filter
    }
}

impl<T> SearchFilter<T> for StandardSearchFilter
where
    T: StandardObject,
{
    fn matches(&self, obj: &T) -> bool {
        let p_filter: StandardPropertiesSearchFilter = self.clone().into();
        let rid_filter: ResourceIdSearchFilter = self.clone().into();

        p_filter.matches(obj) && rid_filter.matches(obj.id())
    }
}

impl Into<StandardPropertiesSearchFilter> for StandardSearchFilter {
    fn into(self) -> StandardPropertiesSearchFilter {
        StandardPropertiesSearchFilter {
            name: self.name,
            kind: self.kind,
            tags: self.tags,
            metadata: self.metadata,
        }
    }
}

impl Into<ResourceIdSearchFilter> for StandardSearchFilter {
    fn into(self) -> ResourceIdSearchFilter {
        ResourceIdSearchFilter { rid: self.rid }
    }
}

// ***************************
// *** Standard Properties ***
// ***************************

/// Search filter for [`super::standard_properties::StandardProperties`].
///
/// # Fields
/// Mimic [`super::standard_properties::StandardProperties`] fields wrapped in `Option`s.
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct StandardPropertiesSearchFilter {
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub tags: Option<HashSet<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl StandardPropertiesSearchFilter {
    /// Creates a new search filter with all fields set to `None`.
    pub fn new() -> StandardPropertiesSearchFilter {
        StandardPropertiesSearchFilter {
            name: None,
            kind: None,
            tags: None,
            metadata: None,
        }
    }
}

impl<T> SearchFilter<T> for StandardPropertiesSearchFilter
where
    T: StandardObject,
{
    fn matches(&self, obj: &T) -> bool {
        self.matches(obj.properties())
    }
}

impl SearchFilter<StandardProperties> for StandardPropertiesSearchFilter {
    fn matches(&self, props: &StandardProperties) -> bool {
        if let Some(s_name) = &self.name {
            if s_name != &props.name {
                return false;
            }
        }
        if let Some(s_kind) = &self.kind {
            if s_kind != &props.kind {
                return false;
            }
        }
        if let Some(s_tags) = &self.tags {
            for s_tag in s_tags {
                if !props.tags.contains(s_tag) {
                    return false;
                }
            }
        }
        if let Some(s_md) = &self.metadata {
            for (s_key, s_val) in s_md {
                match props.metadata.get(s_key) {
                    None => return false,
                    Some((f_val, _)) => {
                        if f_val != s_val {
                            return false;
                        }
                    }
                }
            }
        }

        // all search criteria matched
        true
    }
}

// *******************
// *** Resource Id ***
// *******************

/// Search filter for [`crate::types::ResourceId`].
///
/// # Fields
/// Mimic [`crate::types::ResourceId`] fields wrapped in `Option`s.
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct ResourceIdSearchFilter {
    pub rid: Option<ResourceId>,
}

impl ResourceIdSearchFilter {
    /// Create a new search filter with all fields set to `None`.
    pub fn new() -> ResourceIdSearchFilter {
        ResourceIdSearchFilter { rid: None }
    }
}

impl SearchFilter<ResourceId> for ResourceIdSearchFilter {
    /// Returns whether the resource id matches a filter.
    fn matches(&self, rid: &ResourceId) -> bool {
        if let Some(id) = &self.rid {
            if id != rid {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
#[path = "./search_filter_test.rs"]
mod search_filter_test;
