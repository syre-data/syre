//! Loads a `Project`'s graph.
use crate::commands::graph::get_or_load_project_graph;
use syre_core::graph::ResourceTree;
use syre_core::project::Container;
use syre_core::types::ResourceId;
use syre_local_database::error::server::LoadProjectGraph;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

type ContainerTree = ResourceTree<Container>;

/// Gets a `Project`'s graph.
#[tracing::instrument]
#[hook]
pub fn use_load_project_graph(
    project: &ResourceId,
) -> SuspensionResult<Result<ContainerTree, LoadProjectGraph>> {
    let graph: UseStateHandle<Option<Result<ContainerTree, LoadProjectGraph>>> = use_state(|| None);
    if let Some(graph) = graph.as_ref() {
        match graph {
            Ok(graph) => return Ok(Ok(graph.clone())),
            Err(err) => return Ok(Err(err.clone())),
        }
    }

    let (s, handle) = Suspension::new();
    {
        let graph = graph.setter();
        let project = project.clone();
        spawn_local(async move {
            match get_or_load_project_graph(project.clone()).await {
                Ok(project_graph) => {
                    graph.set(Some(Ok(project_graph)));
                }

                Err(err) => {
                    graph.set(Some(Err(err)));
                }
            }

            handle.resume();
        });
    }

    Err(s)
}
