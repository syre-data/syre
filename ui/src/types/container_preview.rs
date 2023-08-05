//! Container preview.
use std::fmt;
use yew::html::IntoPropValue;
use yew::prelude::*;

/// Preview types.
#[derive(PartialEq, Clone)]
pub enum ContainerPreview {
    None,
    Type,
    Description,
    Tags,
    Metadata,
    Assets,
    Scripts,
}

impl Into<String> for ContainerPreview {
    fn into(self) -> String {
        match self {
            ContainerPreview::None => "None".to_string(),
            ContainerPreview::Type => "Type".to_string(),
            ContainerPreview::Description => "Description".to_string(),
            ContainerPreview::Tags => "Tags".to_string(),
            ContainerPreview::Metadata => "Metadata".to_string(),
            ContainerPreview::Assets => "Data".to_string(),
            ContainerPreview::Scripts => "Scripts".to_string(),
        }
    }
}

impl Into<AttrValue> for ContainerPreview {
    fn into(self) -> AttrValue {
        Into::<String>::into(self).into()
    }
}

impl IntoPropValue<Option<AttrValue>> for ContainerPreview {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(self.into())
    }
}

impl From<String> for ContainerPreview {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Type" => ContainerPreview::Type,
            "Description" => ContainerPreview::Description,
            "Tags" => ContainerPreview::Tags,
            "Metadata" => ContainerPreview::Metadata,
            "Data" => ContainerPreview::Assets,
            "Scripts" => ContainerPreview::Scripts,
            _ => ContainerPreview::None,
        }
    }
}

impl fmt::Display for ContainerPreview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}
