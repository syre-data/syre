//! Interaction with a [`Database`](thot_local::db::Database).
pub mod functions;
pub mod update_actor;

pub use update_actor::{UpdateActor, UpdateActorHandle};
