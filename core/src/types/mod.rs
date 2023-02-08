//! Common types.
pub mod creator;
pub mod local_id;
pub mod resource_id;
pub mod resource_map;
pub mod resource_path;
pub mod user_id;
pub mod user_permissions;

// Reexport
pub use creator::Creator;
pub use local_id::LocalId;
pub use resource_id::ResourceId;
pub use resource_map::{ResourceMap, ResourceStore};
pub use resource_path::ResourcePath;
pub use user_id::UserId;
pub use user_permissions::UserPermissions;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
