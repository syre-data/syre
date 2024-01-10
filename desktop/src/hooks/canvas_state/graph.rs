//! Loads a `Project`'s graph.
use crate::commands::graph::load_project_graph;
use thot_core::graph::ResourceTree;
use thot_core::project::Container;
use thot_core::types::ResourceId;
use thot_local_database::error::server::LoadProjectGraph;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

type ContainerTree = ResourceTree<Container>;

/// Gets a `Project`'s graph.
#[tracing::instrument]
#[hook]
pub fn use_project_graph(
    project: &ResourceId,
) -> SuspensionResult<Result<ContainerTree, LoadProjectGraph>> {
    let graph: UseStateHandle<Option<Result<ContainerTree, LoadProjectGraph>>> = use_state(|| None);
    if let Some(graph) = graph.as_ref() {
        match graph {
            Ok(graph) => return Ok(Ok(graph.clone())),
            Err(err) => {
                return Ok(Err(err.clone()));
            }
        }
    }

    let (s, handle) = Suspension::new();
    {
        let project = project.clone();
        let graph = graph.clone();

        spawn_local(async move {
            match load_project_graph(project).await {
                Ok(project_graph) => {
                    graph.set(Some(Ok(project_graph)));
                    handle.resume();
                }

                Err(err) => {
                    graph.set(Some(Err(err)));
                }
            };
        });
    }

    Err(s)
}
