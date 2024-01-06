//! Loads a `Project`'s graph.
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::graph::load_project_graph;
use thot_core::graph::ResourceTree;
use thot_core::project::Container;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

type ContainerTree = ResourceTree<Container>;

/// Gets a `Project`'s graph.
#[tracing::instrument]
#[hook]
pub fn use_project_graph(project: &ResourceId) -> SuspensionResult<Result<ContainerTree, String>> {
    let graph: UseStateHandle<Option<Result<ContainerTree, String>>> = use_state(|| None);
    if let Some(graph) = (*graph).clone() {
        return Ok(graph);
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
