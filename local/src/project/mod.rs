//! Functionality and resources related to Syre Projects.
pub mod config;
pub mod flag;
pub mod analysis;
pub mod asset;
pub mod container;
pub mod project;
pub mod script;


pub use flag::Flag;
pub use analysis::Analyses;
pub use asset::{Asset, Assets};
pub use container::Container;
pub use project::Project;
pub use script::Script;
