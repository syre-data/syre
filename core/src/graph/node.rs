//! Graph node.
use has_id::HasId;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

pub type NodeId = Uuid;

/// A graph node.
/// Contains data.
// #[derive(HasId)]
pub struct Node<D> {
    // #[id]
    id: NodeId,
    data: D,
}

impl<D> Node<D> {
    pub fn new(data: D) -> Self {
        Self {
            id: NodeId::new_v4(),
            data,
        }
    }
}

impl<D> Deref for Node<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<D> DerefMut for Node<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
#[path = "./node_test.rs"]
mod node_test;
