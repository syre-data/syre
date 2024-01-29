//! Hook for obtaining a [`Container`](CoreContainer).
use crate::components::canvas::ContainerTreeStateReducer;
use syre_core::project::Container as CoreContainer;
use syre_core::types::ResourceId;
use yew::prelude::*;

/// Gets a [`Container`](CoreContainer).
#[hook]
pub fn use_container(rid: ResourceId) -> UseStateHandle<CoreContainer> {
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeStateReducer` context not found");

    let container = use_state(|| {
        tree_state
            .tree
            .get(&rid)
            .cloned()
            .expect("`Container` not found")
    });

    // tree_state updates
    {
        let tree_state = tree_state.clone();
        let container = container.clone();

        use_effect_with((rid, tree_state), move |(rid, tree_state)| {
            container.set(
                tree_state
                    .tree
                    .get(&rid)
                    .cloned()
                    .expect("`Container` not found"),
            )
        })
    }

    container
}
