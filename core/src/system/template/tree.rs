//! Template of a [`ResourceTree`].
use crate::graph::ResourceTree as GraphTree;
use crate::types::ResourceId;
use has_id::{HasId, HasIdSerde};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Result as SerdeResult, Value as JsValue};

pub struct ResourceTree;

impl ResourceTree {
    /// Creates a template from the tree.
    pub fn from_tree<T>(tree: GraphTree<T>) -> SerdeResult<JsValue>
    where
        T: HasId<Id = ResourceId> + HasIdSerde<'static, Id = ResourceId> + Serialize,
    {
        serde_json::to_value(tree)
    }

    /// Creates a new tree from the template.
    pub fn to_tree<T>(template: JsValue) -> SerdeResult<GraphTree<T>>
    where
        T: HasId<Id = ResourceId> + HasIdSerde<'static, Id = ResourceId> + DeserializeOwned,
    {
        serde_json::from_value(template)
    }
}
