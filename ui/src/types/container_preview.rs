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

impl ContainerPreview {
    pub fn as_str(&self) -> &str {
        match self {
            ContainerPreview::None => "None",
            ContainerPreview::Type => "Type",
            ContainerPreview::Description => "Description",
            ContainerPreview::Tags => "Tags",
            ContainerPreview::Metadata => "Metadata",
            ContainerPreview::Assets => "Data",
            ContainerPreview::Scripts => "Scripts",
        }
    }
}

impl AsRef<str> for ContainerPreview {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ToHtml for ContainerPreview {
    fn to_html(&self) -> Html {
        self.as_str().into_html()
    }
}

impl Into<AttrValue> for ContainerPreview {
    fn into(self) -> AttrValue {
        self.to_string().into()
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
        write!(f, "{}", self.as_str())
    }
}
