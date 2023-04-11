/// System level functionality related to Thot.
/// This includes handling system level resources such as users and Scripts,
/// as well as system settings.
#[cfg(feature = "fs")]
pub mod common;

#[cfg(feature = "fs")]
pub mod template;

#[cfg(feature = "fs")]
pub mod collections;

#[cfg(feature = "fs")]
pub mod settings;

#[cfg(feature = "fs")]
pub mod projects;

#[cfg(feature = "fs")]
pub mod scripts;

#[cfg(feature = "fs")]
pub mod users;

pub mod resources;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
