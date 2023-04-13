use thot_core::{project::Asset, types::ResourceId};
use yew::prelude::*;

use crate::{app::AppStateReducer, components::canvas::GraphStateReducer};

#[hook]
pub fn use_asset(rid: &ResourceId) -> UseStateHandle<Asset> {
    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let container = graph_state
        .asset_map
        .get(rid)
        .expect("`Asset`'s `Container` not found");

    let container = graph_state
        .graph
        .get(container)
        .expect("`Container` not found");

    let asset = use_state(|| {
        container
            .assets
            .get(rid)
            .expect("`Asset` not found")
            .clone()
    });

    {
        let rid = rid.clone();
        let asset = asset.clone();
        let graph_state = graph_state.clone();
        use_effect_with_deps(
            move |rid| {
                let container = graph_state
                    .asset_map
                    .get(rid)
                    .expect("`Asset`'s `Container` not found");

                let container = graph_state
                    .graph
                    .get(container)
                    .expect("`Container` not found");

                asset.set(
                    container
                        .assets
                        .get(rid)
                        .expect("`Asset` not found")
                        .clone(),
                );
            },
            rid,
        )
    }
    asset
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
