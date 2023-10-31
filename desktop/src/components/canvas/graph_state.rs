//! State Redcucer for the [`ContainerTree`](super::ContainerTree).
use crate::commands::asset::{
    AssetPropertiesUpdate, BulkUpdatePropertiesArgs as BulkUpdateAssetPropertiesArgs,
};
use crate::commands::common::BulkUpdateResourcePropertiesArgs;
use crate::commands::container::{
    BulkUpdatePropertiesArgs as BulkUpdateContainerPropertiesArgs, BulkUpdateScriptAssociationArgs,
    ContainerPropertiesUpdate, ScriptAssociationsBulkUpdate,
    UpdatePropertiesArgs as UpdateContainerPropertiesArgs, UpdateScriptAssociationsArgs,
};
use std::collections::HashMap;
use std::rc::Rc;
use thot_core::graph::ResourceTree;
use thot_core::project::{container::AssetMap, Asset, Container, RunParameters};
use thot_core::types::{ResourceId, ResourcePath};
use yew::prelude::*;

type ContainerTree = ResourceTree<Container>;
pub type AssetContainerMap = HashMap<ResourceId, ResourceId>;

pub enum GraphStateAction {
    ///  Sets the [`ContainerTree`].
    SetGraph(ContainerTree),

    /// Removes a `Container` subtree from the graph.
    RemoveSubtree(ResourceId),

    /// Move a subtree.
    ///
    /// # Fields
    /// `parent`: New parent.
    /// `root`: Root of the subtree to move.
    MoveSubtree {
        parent: ResourceId,
        root: ResourceId,
    },

    /// Update a [`Container`]'s [`StandardProperties`](thot_::project::StandardProperties).
    UpdateContainerProperties(UpdateContainerPropertiesArgs),

    /// Add a [`Container`] as a child.
    ///
    /// # Fields
    /// #. `parent`: `ResourceId` of the parent.
    /// #. `child`: Child `Container`.
    InsertChildContainer(ResourceId, Container),

    /// Insert the graph as a child of `parent.`
    InsertSubtree {
        parent: ResourceId,
        graph: ContainerTree,
    },

    /// Set a [`Container`]'s [`Asset`]s.
    SetContainerAssets(ResourceId, AssetMap),

    /// Insert [`Asset`]s into a [`Container`].
    InsertContainerAssets(ResourceId, Vec<Asset>),

    /// Update a [`Container`]'s
    /// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
    UpdateContainerScriptAssociations(UpdateScriptAssociationsArgs),

    /// Remove all associations with `Script` from [`Container`]'s.
    RemoveContainerScriptAssociations(ResourceId),

    // Remove an [`Asset`].
    RemoveAsset(ResourceId),

    /// Update an [`Asset`].
    UpdateAsset(Asset),

    UpdateAssetPath {
        asset: ResourceId,
        path: ResourcePath,
    },

    /// Move an `Asset` to another `Container`.  
    MoveAsset {
        asset: ResourceId,
        container: ResourceId,
    },

    SetDragOverContainer(ResourceId),
    ClearDragOverContainer,

    /// Bulk update `Container`s.
    BulkUpdateContainerProperties(BulkUpdateContainerPropertiesArgs),

    /// Bulk update `Asset`s.
    BulkUpdateAssetProperties(BulkUpdateAssetPropertiesArgs),

    /// Bulk update resource properties.
    BulkUpdateResourceProperties(BulkUpdateResourcePropertiesArgs),

    /// Bulk update `Container` `ScriptAssociation`s.
    BulkUpdateContainerScriptAssociations(BulkUpdateScriptAssociationArgs),
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
    #[tracing::instrument(skip(self))]
    fn update_container_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &ContainerPropertiesUpdate,
    ) {
        let container = self
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
            .append(&mut update.tags.insert.clone());

        container.properties.tags.sort();
        container.properties.tags.dedup();
        container
            .properties
            .tags
            .retain(|tag| !update.tags.remove.contains(tag));

        // metadata
        container
            .properties
            .metadata
            .extend(update.metadata.insert.clone());

        for key in update.metadata.remove.iter() {
            container.properties.metadata.remove(key);
        }
    }

    /// Update an `Assets`'s properties.
    #[tracing::instrument(skip(self))]
    fn update_asset_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &AssetPropertiesUpdate,
    ) {
        let container = self.asset_map.get(rid).expect("`Asset` map not found");
        let container = self
            .graph
            .get_mut(container)
            .expect("could not find `Container`");

        let asset = container.assets.get_mut(rid).expect("`Asset` not found");

        // basic properties
        if let Some(name) = update.name.as_ref() {
            asset.properties.name = name.clone();
        }

        if let Some(kind) = update.kind.as_ref() {
            asset.properties.kind = kind.clone();
        }

        if let Some(description) = update.description.as_ref() {
            asset.properties.description = description.clone();
        }

        // tags
        asset
            .properties
            .tags
            .append(&mut update.tags.insert.clone());

        asset.properties.tags.sort();
        asset.properties.tags.dedup();
        asset
            .properties
            .tags
            .retain(|tag| !update.tags.remove.contains(tag));

        // metadata
        asset
            .properties
            .metadata
            .extend(update.metadata.insert.clone());

        for key in update.metadata.remove.iter() {
            asset.properties.metadata.remove(key);
        }
    }

    /// Update a `Container`'s `ScriptAssociations`.
    ///
    /// # Note
    /// Updates are processed in the following order:
    /// 1. New associations are added.
    /// 2. Present associations are updated.
    /// 3. Associations are removed.
    pub fn update_container_script_associations_from_update(
        &mut self,
        rid: &ResourceId,
        update: &ScriptAssociationsBulkUpdate,
    ) {
        let container = self
            .graph
            .get_mut(&rid)
            .expect("could not find `Container`");

        for assoc in update.add.iter() {
            container.scripts.insert(
                assoc.script.clone(),
                RunParameters {
                    priority: assoc.priority.clone(),
                    autorun: assoc.autorun.clone(),
                },
            );
        }

        for u in update.update.iter() {
            let Some(script) = container.scripts.get_mut(&u.script) else {
                continue;
            };

            if let Some(priority) = u.priority.as_ref() {
                script.priority = priority.clone();
            }

            if let Some(autorun) = u.autorun.as_ref() {
                script.autorun = autorun.clone();
            }
        }

        for script in update.remove.iter() {
            container.scripts.remove(script);
        }
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
                // let mut asset_map = AssetContainerMap::new();
                current.asset_map.clear();
                for container in graph.nodes().values() {
                    for aid in container.assets.keys() {
                        current.asset_map.insert(aid.clone(), container.rid.clone());
                    }
                }

                current.graph = graph;
            }

            GraphStateAction::RemoveSubtree(root) => {
                let graph = current.graph.remove(&root).unwrap();
                for container in graph.nodes().values() {
                    for asset in container.assets.keys() {
                        current.asset_map.remove(asset);
                    }
                }
            }

            GraphStateAction::MoveSubtree { parent, root } => {
                current.graph.mv(&root, &parent).unwrap();
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

            GraphStateAction::InsertSubtree { parent, graph } => {
                for container in graph.nodes().values() {
                    for aid in container.assets.keys() {
                        current.asset_map.insert(aid.clone(), container.rid.clone());
                    }
                }

                current.graph.insert_tree(&parent, graph).unwrap();
            }

            GraphStateAction::SetContainerAssets(container_rid, assets) => {
                let Some(container) = current.graph.get_mut(&container_rid) else {
                    panic!("`Container` not found")
                };

                container.assets = assets;
                current.asset_map.clear();
                for asset in container.assets.keys() {
                    current
                        .asset_map
                        .insert(asset.clone(), container.rid.clone());
                }
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

            GraphStateAction::RemoveAsset(asset) => {
                let container = current.asset_map.get(&asset).unwrap();
                let container = current
                    .graph
                    .get_mut(container)
                    .expect("`Container` not found");

                container.assets.remove(&asset);
                current.asset_map.remove(&asset);
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

                // TODO Ensure `Asset` exists in `Container` before update.
                container.assets.insert(asset.rid.clone(), asset.clone());
            }

            GraphStateAction::UpdateAssetPath { asset, path } => {
                let container = current.asset_map.get(&asset).unwrap();
                let container = current.graph.get_mut(container).unwrap();
                let asset = container.assets.get_mut(&asset).unwrap();
                asset.path = path;
            }

            GraphStateAction::MoveAsset { asset, container } => {
                let container_o = current.asset_map.get(&asset).unwrap();
                let container_o = current.graph.get_mut(container_o).unwrap();
                let asset = container_o.assets.remove(&asset).unwrap();
                current.asset_map.remove(&asset.rid);

                let container = current.graph.get_mut(&container).unwrap();
                let aid = asset.rid.clone();
                container.insert_asset(asset);
                current.asset_map.insert(aid, container.rid.clone());
            }

            GraphStateAction::SetDragOverContainer(rid) => {
                current.dragover_container = Some(rid);
            }

            GraphStateAction::ClearDragOverContainer => {
                current.dragover_container = None;
            }

            GraphStateAction::BulkUpdateContainerProperties(
                BulkUpdateContainerPropertiesArgs { rids, update },
            ) => {
                for rid in rids {
                    current.update_container_properties_from_update(&rid, &update);
                }
            }

            GraphStateAction::BulkUpdateAssetProperties(BulkUpdateAssetPropertiesArgs {
                rids,
                update,
            }) => {
                for rid in rids {
                    current.update_asset_properties_from_update(&rid, &update);
                }
            }

            GraphStateAction::BulkUpdateResourceProperties(BulkUpdateResourcePropertiesArgs {
                rids,
                update,
            }) => {
                for rid in rids {
                    if self.asset_map.contains_key(&rid) {
                        current.update_asset_properties_from_update(&rid, &update.clone().into());
                    } else {
                        current
                            .update_container_properties_from_update(&rid, &update.clone().into());
                    }
                }
            }

            GraphStateAction::BulkUpdateContainerScriptAssociations(
                BulkUpdateScriptAssociationArgs { containers, update },
            ) => {
                for container in containers {
                    current.update_container_script_associations_from_update(&container, &update);
                }
            }
        };

        current.into()
    }
}

pub type GraphStateReducer = UseReducerHandle<GraphState>;
