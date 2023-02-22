//! Hook for obtaining a [`Container`](CoreContainer).
use crate::components::canvas::ContainerTreeStateReducer;
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use yew::prelude::*;

/// Gets a [`Container`](CoreContainer).
#[hook]
pub fn use_container(rid: ResourceId) -> UseStateHandle<CoreContainer> {
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeStateReducer` context not found");

    let container = use_state(|| {
        tree_state
            .containers
            .get(&rid)
            .cloned()
            .flatten()
            .expect("`Container` not loaded")
            .lock()
            .expect("could not lock container")
            .clone()
    });

    // tree_state updates
    {
        let tree_state = tree_state.clone();
        let container = container.clone();

        use_effect_with_deps(
            move |(rid, tree_state)| {
                container.set(
                    tree_state
                        .containers
                        .get(&rid)
                        .cloned()
                        .flatten()
                        .expect("`Container` not loaded")
                        .lock()
                        .expect("could not lock container")
                        .clone(),
                )
            },
            (rid, tree_state),
        )
    }

    container
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
