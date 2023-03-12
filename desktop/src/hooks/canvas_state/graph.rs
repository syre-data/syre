//! Loads a `Project`'s graph.
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use serde_wasm_bindgen as swb;
use thot_core::graph::ResourceTree;
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

type ContainerTree = ResourceTree<CoreContainer>;

/// Gets a `Project`'s graph.
#[hook]
pub fn use_project_graph(project: &ResourceId) -> SuspensionResult<ContainerTree> {
    let graph: UseStateHandle<Option<ContainerTree>> = use_state(|| None);
    if let Some(graph) = (*graph).clone() {
        return Ok(graph);
    }

    let (s, handle) = Suspension::new();
    {
        let project = project.clone();
        let graph = graph.clone();

        spawn_local(async move {
            let p_graph = invoke("load_project_graph", ResourceIdArgs { rid: project })
                .await
                .expect("could not invoke `load_project_graph`");

            let p_graph: ContainerTree = swb::from_value(p_graph)
                .expect("could not convert result of `load_project_graph` to JsValue");

            graph.set(Some(p_graph));
            handle.resume();
        });
    }

    Err(s)
}

#[cfg(test)]
#[path = "./graph_test.rs"]
mod graph_test;
