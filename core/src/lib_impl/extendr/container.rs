//! `rextendr` `impl` for [`Container`].
use crate::project::Container;
use extendr_api::prelude::*;

#[extendr]
impl Container {
    pub fn name(&self) -> Option<&str> {
        self.properties.name.as_ref().map(|name| name.as_str())
    }
}

extendr_module! {
    mod container;
    impl Container;
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
