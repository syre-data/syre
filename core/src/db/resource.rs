//! Database object functionality.
use crate::project::standard_properties::StandardProperties;
use crate::types::ResourceId;
use has_id::HasId;
use std::hash::Hash;

/// Functionality for database objects.
pub trait Resource: HasId<Id = ResourceId> + Hash + Clone + PartialEq + Eq {}

/// Functionality for standard objects.
pub trait StandardResource: Resource {
    /// Retrieve a reference to the [`StandardProperties`] of the object.
    fn properties(&self) -> &StandardProperties;

    /// Retrieve a mutable reference to the [`StandardProperties`] of the object.
    fn properties_mut(&mut self) -> &mut StandardProperties;
}

#[cfg(test)]
#[path = "./resource_test.rs"]
mod resource_test;
