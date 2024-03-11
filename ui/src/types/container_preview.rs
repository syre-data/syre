//! Container preview.
use std::fmt;
use yew::html::IntoPropValue;
use yew::prelude::*;

/// Preview types.
#[derive(PartialEq, Clone, Debug)]
pub enum ContainerPreview {
    None,
    Type,
    Description,
    Tags,
    Metadata,
    Assets,
    Analysis,
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
            ContainerPreview::Analysis => "Analysis",
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
    /// Converts
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "type" => ContainerPreview::Type,
            "description" => ContainerPreview::Description,
            "tags" => ContainerPreview::Tags,
            "metadata" => ContainerPreview::Metadata,
            "data" => ContainerPreview::Assets,
            "analysis" => ContainerPreview::Analysis,
            "none" => ContainerPreview::None,
            _ => {
                panic!("Invalid container preview string `{s}`");
            }
        }
    }
}

impl fmt::Display for ContainerPreview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}
