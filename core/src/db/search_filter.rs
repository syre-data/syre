//! Search filter functionality.
use super::StandardResource;
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
#[derive(Default, Debug, Clone)]
pub struct StandardSearchFilter {
    pub rid: Option<ResourceId>,
    pub name: Option<Option<String>>,
    pub kind: Option<Option<String>>,
    pub tags: Option<HashSet<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl<T> SearchFilter<T> for StandardSearchFilter
where
    T: StandardResource,
{
    fn matches(&self, resource: &T) -> bool {
        if let Some(s_rid) = self.rid.as_ref() {
            if s_rid != resource.id() {
                return false;
            }
        }

        let props = resource.properties();
        if let Some(s_name) = self.name.as_ref() {
            if s_name != &props.name {
                return false;
            }
        }
        if let Some(s_kind) = self.kind.as_ref() {
            if s_kind != &props.kind {
                return false;
            }
        }
        if let Some(s_tags) = self.tags.as_ref() {
            for s_tag in s_tags {
                if !props.tags.contains(s_tag) {
                    return false;
                }
            }
        }
        if let Some(s_md) = self.metadata.as_ref() {
            for (s_key, s_val) in s_md {
                match props.metadata.get(s_key) {
                    None => return false,
                    Some(f_val) => {
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

#[cfg(test)]
#[path = "./search_filter_test.rs"]
mod search_filter_test;
