pub(self) mod canvas;
mod nav;
mod project_bar;
mod properties;
pub(self) mod workspace;

pub(self) use canvas::{CONTAINER_WIDTH, Canvas};
pub(self) use nav::NavBar;
pub(crate) use project_bar::ProjectBar;
pub(self) use properties::PropertiesBar;
pub(crate) use workspace::Workspace;
