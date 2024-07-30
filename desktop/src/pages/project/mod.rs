mod canvas;
pub(self) mod common;
mod layers;
mod project_bar;
mod properties;
mod state;
mod workspace;

pub(self) use canvas::Canvas;
pub(self) use layers::LayersNav;
pub(self) use project_bar::ProjectBar;
pub(self) use properties::PropertiesBar;
pub use workspace::Workspace;
