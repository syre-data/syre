//! ['yew'] implementations for ['ResourceId'].
use crate::types::ResourceId;
use yew::html::IntoPropValue;
use yew::virtual_dom::{AttrValue, Key};

impl Into<Key> for ResourceId {
    fn into(self) -> Key {
        self.to_string().into()
    }
}

impl IntoPropValue<AttrValue> for ResourceId {
    fn into_prop_value(self) -> AttrValue {
        self.to_string().into()
    }
}

impl IntoPropValue<Option<AttrValue>> for ResourceId {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(self.to_string().into())
    }
}

#[cfg(test)]
#[path = "./resource_id_test.rs"]
mod resource_id_test;
