use syre_core::graph::ResourceTree;
use syre_local::project::resources::{Container, Project};

type ContainerTree = ResourceTree<Container>;

pub struct ProjectResources {
    pub project: Option<Project>,
    pub graph: Option<ContainerTree>,
}
