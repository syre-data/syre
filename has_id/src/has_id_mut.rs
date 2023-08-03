//! The object has a `mut`able id.
use crate::HasId;

// @todo[2]: Remove
/// Indicates that the id can be mutated.
pub trait HasIdMut: HasId {
    fn id_mut(&mut self) -> &mut Self::Id;
}
