//! Common types for metadata.
use serde_json::Value as JsValue;
use std::collections::HashMap;
use yew::html::IntoPropValue;
use yew::virtual_dom::AttrValue;

pub type Metadatum = (String, JsValue);
pub type MetadataBulk = HashMap<String, Vec<JsValue>>;

/// Types a metadatum value can assume.
#[derive(PartialEq, Clone, Debug)]
pub enum MetadatumType {
    String,
    Bool,
    Number,
    Array,
    Object,
}

impl Default for MetadatumType {
    fn default() -> Self {
        Self::String
    }
}

impl Into<String> for MetadatumType {
    fn into(self) -> String {
        match self {
            MetadatumType::String => "String".to_string(),
            MetadatumType::Number => "Number".to_string(),
            MetadatumType::Bool => "Boolean".to_string(),
            MetadatumType::Array => "Array".to_string(),
            MetadatumType::Object => "Object".to_string(),
        }
    }
}

impl Into<AttrValue> for MetadatumType {
    fn into(self) -> AttrValue {
        Into::<String>::into(self).into()
    }
}

impl IntoPropValue<Option<AttrValue>> for MetadatumType {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(self.into())
    }
}

/// Returns the type the string represents.
pub fn type_from_string(s: &str) -> Option<MetadatumType> {
    match s {
        "String" => Some(MetadatumType::String),
        "Number" => Some(MetadatumType::Number),
        "Boolean" => Some(MetadatumType::Bool),
        "Array" => Some(MetadatumType::Array),
        "Object" => Some(MetadatumType::Object),
        _ => None,
    }
}

/// Returns the type of the value.
pub fn type_of_value(value: &JsValue) -> Option<MetadatumType> {
    match value {
        JsValue::Null => None,
        JsValue::String(_) => Some(MetadatumType::String),
        JsValue::Number(_) => Some(MetadatumType::Number),
        JsValue::Bool(_) => Some(MetadatumType::Bool),
        JsValue::Array(_) => Some(MetadatumType::Array),
        JsValue::Object(_) => Some(MetadatumType::Object),
    }
}

#[cfg(test)]
#[path = "./types_test.rs"]
mod types_test;
