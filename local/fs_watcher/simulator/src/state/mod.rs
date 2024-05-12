//! Track simulator state.
pub mod action;
pub mod app;
pub mod graph;

pub use app::{
    Asset, Container, ContainerConfig, Data, Project, ProjectConfig, Reference, Resource,
};

pub use action::Action;

pub trait HasPath {
    fn path(&self) -> &std::path::PathBuf;
}
