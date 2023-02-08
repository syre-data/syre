//! ['yew'] implementations for ['Project'].
use crate::project::Project;
use yew::virtual_dom::Key;

impl Into<Key> for Project {
    fn into(self) -> Key {
        self.rid.into()
    }
}

impl Into<Key> for &Project {
    fn into(self) -> Key {
        self.rid.clone().into()
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
