//! `rextendr` `impl` for [`Container`].
use crate::project::{Container, Metadata};
use extendr_api::prelude::*;

#[extendr]
impl Container {
    pub fn name(&self) -> Option<&str> {
        self.properties.name.as_ref().map(|name| name.as_str())
    }

    // TODO[h]: Change to `r#type`.
    // See https://github.com/extendr/extendr/issues/528 for more info.
    pub fn kind(&self) -> Option<&str> {
        self.properties.kind.as_ref().map(|name| name.as_str())
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.properties.tags
    }

    // TODO[h]
    // pub fn metadata(&self) -> &Metadata {
    //     &self.properties.metadata
    // }
}

extendr_module! {
    mod container;
    impl Container;
}
