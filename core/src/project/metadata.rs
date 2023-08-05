//! Metadata.
use std::collections::HashMap;

pub type Metadata = HashMap<String, serde_json::Value>;

pub trait InheritMetadata {
    /// Returns owned and inherited [`Metadata`].
    fn metadata_all(&self) -> &Metadata;

    /// Returns owned [`Metadata`].
    fn metadata_owned(&self) -> &Metadata;

    /// Returns inherited [`Metadata`].
    fn metadata_inherited(&self) -> &Metadata;
}
