//! Graph node.
use crate::types::ResourceId;
use has_id::HasId;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// TODO Clean up serde.
/// A graph node for a resource.
/// The id of the node matches the id of the resource.
/// Contains data.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(HasId, Clone, PartialEq)]
pub struct ResourceNode<D>
where
    D: HasId<Id = ResourceId>,
{
    #[id]
    id: ResourceId,
    data: D,
}

impl<D> ResourceNode<D>
where
    D: HasId<Id = ResourceId>,
{
    pub fn new(data: D) -> Self {
        let id = data.id().clone();

        Self { id, data }
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    /// Consumes self, returning the data.
    pub fn into_data(self) -> D {
        self.data
    }
}

impl<D> Deref for ResourceNode<D>
where
    D: HasId<Id = ResourceId>,
{
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<D> DerefMut for ResourceNode<D>
where
    D: HasId<Id = ResourceId>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<D> fmt::Debug for ResourceNode<D>
where
    D: fmt::Debug + HasId<Id = ResourceId>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.data())
    }
}
