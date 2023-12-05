//! Loads a `Project`'s graph.
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
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
pub fn use_project_graph(project: &ResourceId) -> SuspensionResult<ContainerTree> {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let graph: UseStateHandle<Option<ContainerTree>> = use_state(|| None);
    if let Some(graph) = (*graph).clone() {
        return Ok(graph);
    }

    let (s, handle) = Suspension::new();
    {
        let app_state = app_state.clone();
        let project = project.clone();
        let graph = graph.clone();

        spawn_local(async move {
            let Ok(p_graph) =
                invoke::<ContainerTree>("load_project_graph", ResourceIdArgs { rid: project })
                    .await
            else {
                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                    "Could not get project's graph",
                )));
                return;
            };

            graph.set(Some(p_graph));
            handle.resume();
        });
    }

    Err(s)
}
