//! Template of a [`ResourceTree`].
use crate::graph::ResourceTree as GraphTree;
use has_id::HasIdSerde;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Result as SerdeResult, Value as JsValue};

pub struct ResourceTree;

impl ResourceTree {
    /// Creates a template from the tree.
    pub fn from_tree<T>(tree: GraphTree<T>) -> SerdeResult<JsValue>
    where
        T: HasIdSerde<'static> + Clone + Serialize,
    {
        serde_json::to_value(tree)
    }

    /// Creates a new tree from the template.
    pub fn to_tree<T>(template: JsValue) -> SerdeResult<GraphTree<T>>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(template)
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
