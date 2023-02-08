//! Hook for obtaining a [`Container`](CoreContainer).
use crate::components::canvas::ContainerTreeStateReducer;
use std::sync::{Arc, Mutex};
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use yew::prelude::*;

type ContainerWrapper = Arc<Mutex<CoreContainer>>;

/// Gets a [`Container`](CoreContainer).
#[hook]
pub fn use_container(rid: ResourceId) -> UseStateHandle<Option<ContainerWrapper>> {
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeStateReducer` context not found");

    let container = use_state(|| {
        if let Some(container) = tree_state.containers.get(&rid) {
            container.clone()
        } else {
            None
        }
    });

    {
        let tree_state = tree_state.clone();
        let container = container.clone();

        use_effect_with_deps(
            move |tree_state| {
                let container_val = if let Some(c) = tree_state.containers.get(&rid) {
                    c.clone()
                } else {
                    None
                };

                container.set(container_val);
            },
            tree_state,
        )
    }

    container
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
