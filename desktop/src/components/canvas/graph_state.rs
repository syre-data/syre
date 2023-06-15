//! State Redcucer for the [`ContainerTree`](super::ContainerTree).
use crate::commands::common::{BulkUpdatePropertiesArgs, UpdatePropertiesArgs};
use crate::commands::container::UpdateScriptAssociationsArgs;
use crate::commands::types::StandardPropertiesUpdate;
use std::collections::HashMap;
use std::rc::Rc;
use thot_core::graph::ResourceTree;
use thot_core::project::{container::AssetMap, Asset, Container};
use thot_core::types::ResourceId;
use yew::prelude::*;

type ContainerTree = ResourceTree<Container>;

pub type AssetContainerMap = HashMap<ResourceId, ResourceId>;

pub enum GraphStateAction {
    ///  Sets the [`ContainerTree`].
    SetGraph(ContainerTree),

    /// Update a [`Container`](Container)'s [`StandardProperties`](thot_::project::StandardProperties).
    UpdateContainerProperties(UpdatePropertiesArgs),

    /// Add a [`Container`](Container) as a child.
    ///
    /// # Fields
    /// #. `parent`: `ResourceId` of the parent.
    /// #. `child`: Child `Container`.
    InsertChildContainer(ResourceId, Container),

    /// Update [`Asset`](CoreAsset)s of a [`Container`](CoreContainer).
    UpdateContainerAssets(ResourceId, AssetMap),

    /// Insert [`Asset`](Asset)s into a [`Container`](Container).
    InsertContainerAssets(ResourceId, Vec<Asset>),

    /// Update a [`Container`](Container)'s
    /// [`ScriptAssociation`](thot_::project::ScriptAssociation)s.
    UpdateContainerScriptAssociations(UpdateScriptAssociationsArgs),

    /// Remove all associations with `Script` from [`Container`](CoreContainer)'s
    RemoveContainerScriptAssociations(ResourceId),

    /// Update an [`Asset`](Asset).
    UpdateAsset(Asset),

    SetDragOverContainer(ResourceId),
    ClearDragOverContainer,

    /// Bulk update `Container`s.
    BulkUpdateContainerProperties(BulkUpdatePropertiesArgs),
}

#[derive(PartialEq, Clone)]
pub struct GraphState {
    pub graph: ContainerTree,

    /// Map from an [`Asset`](Asset)'s id to its [`Container`](Container)'s.
    pub asset_map: AssetContainerMap,

    /// Indicates the `Container` which currently has something dragged over it.
    /// Used to indicate which `Container` dropped files should be added to as `Asset`s.
    pub dragover_container: Option<ResourceId>,
}

impl GraphState {
    pub fn new(graph: ContainerTree) -> Self {
        let mut asset_map = AssetContainerMap::new();
        for container in graph.nodes().values() {
            for aid in container.assets.keys() {
                asset_map.insert(aid.clone(), container.rid.clone());
            }
        }

        Self {
            graph,
            asset_map,
            dragover_container: None,
        }
    }

    /// Update a `Container`'s properties.
    #[tracing::instrument(skip(state))]
    fn update_container_properties_from_update(
        state: &mut Self,
        rid: &ResourceId,
        update: &StandardPropertiesUpdate,
    ) {
        let container = state
            .graph
            .get_mut(&rid)
            .expect("could not find `Container`");

        // basic properties
        if let Some(name) = update.name.as_ref() {
            container.properties.name = name.clone();
        }

        if let Some(kind) = update.kind.as_ref() {
            container.properties.kind = kind.clone();
        }

        if let Some(description) = update.description.as_ref() {
            container.properties.description = description.clone();
        }

        // tags
        container
            .properties
            .tags
            .append(&mut update.tags.add.clone());

        container.properties.tags.sort();
        container.properties.tags.dedup();
        container
            .properties
            .tags
            .retain(|tag| !update.tags.remove.contains(tag));
    }
}

impl Reducible for GraphState {
    type Action = GraphStateAction;

    // @note: Actions that change a `Container` must first `clone`
    // the `Container` then re-`insert` it into the state's `Container` store
    // so equality can be evaluated.
    // A `Container`'s value must never be changed in place.
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();

        match action {
            GraphStateAction::SetGraph(graph) => {
                let mut asset_map = AssetContainerMap::new();
                for container in graph.nodes().values() {
                    for aid in container.assets.keys() {
                        asset_map.insert(aid.clone(), container.rid.clone());
                    }
                }

                current.graph = graph;
                current.asset_map = asset_map;
            }

            GraphStateAction::UpdateContainerProperties(update) => {
                let container = current
                    .graph
                    .get_mut(&update.rid)
                    .expect("`Container` not found");

                container.properties = update.properties;
            }

            GraphStateAction::UpdateContainerScriptAssociations(update) => {
                let container = current
                    .graph
                    .get_mut(&update.rid)
                    .expect("`Container` not found");

                container.scripts = update.associations;
            }

            GraphStateAction::RemoveContainerScriptAssociations(rid) => {
                for cid in current.graph.nodes().clone().into_keys() {
                    let container = current
                        .graph
                        .get_mut(&cid)
                        .expect("`Container` not found in graph");
                    container.scripts.remove(&rid);
                }
            }

            GraphStateAction::InsertChildContainer(parent, child) => {
                // map assets
                for rid in child.assets.keys() {
                    current.asset_map.insert(rid.clone(), child.rid.clone());
                }

                // insert child into store
                current
                    .graph
                    .insert(parent, child)
                    .expect("could not insert child node");
            }

            GraphStateAction::InsertContainerAssets(container, assets) => {
                let container = current
                    .graph
                    .get_mut(&container)
                    .expect("`Container` not found");

                for asset in assets {
                    current
                        .asset_map
                        .insert(asset.rid.clone(), container.rid.clone());

                    container.assets.insert(asset.rid.clone(), asset);
                }
            }

            GraphStateAction::UpdateContainerAssets(container_rid, assets) => {
                let Some(container) = current
                    .graph
                    .get_mut(&container_rid)
                     else {
                        panic!("`Container` not found")
                    };

                container.assets = assets;
            }

            GraphStateAction::UpdateAsset(asset) => {
                let container = current
                    .asset_map
                    .get(&asset.rid)
                    .expect("`Asset` `Container` not found");

                let container = current
                    .graph
                    .get_mut(&container)
                    .expect("`Container` not found");

                // @todo: Ensure `Asset` exists in `Container` before update.
                container.assets.insert(asset.rid.clone(), asset.clone());
            }

            GraphStateAction::SetDragOverContainer(rid) => {
                current.dragover_container = Some(rid);
            }

            GraphStateAction::ClearDragOverContainer => {
                current.dragover_container = None;
            }

            GraphStateAction::BulkUpdateContainerProperties(BulkUpdatePropertiesArgs {
                rids,
                update,
            }) => {
                for rid in rids {
                    Self::update_container_properties_from_update(&mut current, &rid, &update);
                }
            }
        };

        current.into()
    }
}

pub type GraphStateReducer = UseReducerHandle<GraphState>;

#[cfg(test)]
#[path = "./graph_state_test.rs"]
mod graph_state_test;
