//! State Redcucer for the [`ContainerTree`](super::ContainerTree).
use crate::commands::common::BulkUpdateResourcePropertiesArgs;
use crate::commands::container::{
    BulkUpdatePropertiesArgs as BulkUpdateContainerPropertiesArgs, UpdateAnalysisAssociationsArgs,
    UpdatePropertiesArgs as UpdateContainerPropertiesArgs,
};
use crate::types::RcEq;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::graph::ResourceTree;
use syre_core::project::{container::AssetMap, Asset, Container, RunParameters};
use syre_core::types::ResourceId;
use syre_local_database::command::asset::{
    BulkUpdatePropertiesArgs as BulkUpdateAssetPropertiesArgs,
    PropertiesUpdate as AssetPropertiesUpdate,
};
use syre_local_database::command::container::{
    AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs,
    PropertiesUpdate as ContainerPropertiesUpdate,
};
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
    /// `name`: Name of the root `Container`.
    MoveSubtree {
        parent: ResourceId,
        root: ResourceId,
        name: String,
    },

    /// Update a [`Container`]'s [`StandardProperties`](syre_::project::StandardProperties).
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
    /// [`AnalysisAssociation`](syre_core::project::AnalysisAssociation)s.
    UpdateContainerAnalysisAssociations(UpdateAnalysisAssociationsArgs),

    /// Remove all associations with analysis from [`Container`]'s.
    RemoveContainerAnalysisAssociations(ResourceId),

    // Remove an [`Asset`].
    RemoveAsset(ResourceId),

    /// Update an [`Asset`].
    UpdateAsset(Asset),

    UpdateAssetPath {
        asset: ResourceId,
        path: PathBuf,
    },

    /// Move an `Asset` to another `Container`.  
    MoveAsset {
        asset: ResourceId,
        container: ResourceId,
        path: PathBuf,
    },

    /// Bulk update `Container`s.
    BulkUpdateContainerProperties(BulkUpdateContainerPropertiesArgs),

    /// Bulk update `Asset`s.
    BulkUpdateAssetProperties(BulkUpdateAssetPropertiesArgs),

    /// Bulk update resource properties.
    BulkUpdateResourceProperties(BulkUpdateResourcePropertiesArgs),

    /// Bulk update `Container` `ScriptAssociation`s.
    BulkUpdateContainerScriptAssociations(BulkUpdateAnalysisAssociationsArgs),
}

#[derive(Clone, PartialEq)]
pub struct GraphState {
    pub graph: RcEq<ContainerTree>,

    /// Map from an [`Asset`](Asset)'s id to its [`Container`](Container)'s.
    pub asset_map: AssetContainerMap,
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
            graph: RcEq::new(graph),
            asset_map,
        }
    }

    /// Update a `Container`'s properties.
    fn update_container_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &ContainerPropertiesUpdate,
    ) {
        let graph = Rc::get_mut(&mut self.graph).unwrap();
        let container = graph.get_mut(&rid).expect("could not find `Container`");

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
    fn update_asset_properties_from_update(
        &mut self,
        rid: &ResourceId,
        update: &AssetPropertiesUpdate,
    ) {
        let graph = Rc::get_mut(&mut self.graph).unwrap();
        let container = self.asset_map.get(rid).expect("`Asset` map not found");
        let container = graph
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
        update: &AnalysisAssociationBulkUpdate,
    ) {
        let graph = Rc::get_mut(&mut self.graph).unwrap();
        let container = graph.get_mut(&rid).expect("could not find `Container`");

        for assoc in update.add.iter() {
            container.analyses.insert(
                assoc.analysis.clone(),
                RunParameters {
                    priority: assoc.priority.clone(),
                    autorun: assoc.autorun.clone(),
                },
            );
        }

        for u in update.update.iter() {
            let Some(script) = container.analyses.get_mut(&u.analysis) else {
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
            container.analyses.remove(script);
        }
    }
}

impl Reducible for GraphState {
    type Action = GraphStateAction;

    // NB: Actions that change a `Container` must first `clone`
    // the `Container` then re-`insert` it into the state's `Container` store
    // so equality can be evaluated.
    // A `Container`'s value must never be changed in place.
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        if Rc::strong_count(&current.graph) > 1 {
            current.graph = RcEq::new((**current.graph).clone());
        }

        match action {
            GraphStateAction::SetGraph(graph) => {
                current.asset_map.clear();
                for container in graph.nodes().values() {
                    for aid in container.assets.keys() {
                        current.asset_map.insert(aid.clone(), container.rid.clone());
                    }
                }

                current.graph = RcEq::new(graph);
            }

            GraphStateAction::RemoveSubtree(root) => {
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                if let Ok(graph) = graph.remove(&root) {
                    for container in graph.nodes().values() {
                        for asset in container.assets.keys() {
                            current.asset_map.remove(asset);
                        }
                    }
                }
            }

            GraphStateAction::MoveSubtree { parent, root, name } => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                graph.mv(&root, &parent).unwrap();
                let root = graph.get_mut(&root).unwrap();
                root.properties.name = name;
            }

            GraphStateAction::UpdateContainerProperties(update) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container = graph.get_mut(&update.rid).expect("`Container` not found");

                container.properties = update.properties;
            }

            GraphStateAction::UpdateContainerAnalysisAssociations(update) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container = graph.get_mut(&update.rid).expect("`Container` not found");

                container.analyses = update.associations;
            }

            GraphStateAction::RemoveContainerAnalysisAssociations(rid) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                for cid in graph.nodes().clone().into_keys() {
                    let container = graph.get_mut(&cid).expect("`Container` not found in graph");

                    container.analyses.remove(&rid);
                }
            }

            GraphStateAction::InsertChildContainer(parent, child) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                // map assets
                for rid in child.assets.keys() {
                    current.asset_map.insert(rid.clone(), child.rid.clone());
                }

                // insert child into store
                graph
                    .insert(parent, child)
                    .expect("could not insert child node");
            }

            GraphStateAction::InsertSubtree { parent, graph } => {
                current.graph = RcEq::new((**current.graph).clone());
                let root_graph = Rc::get_mut(&mut current.graph).unwrap();

                for container in graph.nodes().values() {
                    for aid in container.assets.keys() {
                        current.asset_map.insert(aid.clone(), container.rid.clone());
                    }
                }

                root_graph.insert_tree(&parent, graph).unwrap();
            }

            GraphStateAction::SetContainerAssets(container_rid, assets) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let Some(container) = graph.get_mut(&container_rid) else {
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
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container = graph.get_mut(&container).expect("`Container` not found");

                for asset in assets {
                    current
                        .asset_map
                        .insert(asset.rid.clone(), container.rid.clone());

                    container.assets.insert(asset.rid.clone(), asset);
                }
            }

            GraphStateAction::RemoveAsset(asset) => match current.asset_map.get(&asset) {
                Some(container) => {
                    current.graph = RcEq::new((**current.graph).clone());
                    let graph = Rc::get_mut(&mut current.graph).unwrap();

                    if let Some(container) = graph.get_mut(container) {
                        container.assets.remove(&asset);
                    }

                    current.asset_map.remove(&asset);
                }

                None => {
                    if let Some(container) =
                        current.graph.iter_nodes().find_map(|(cid, container)| {
                            if container.assets.contains_key(&asset) {
                                Some(cid.clone())
                            } else {
                                None
                            }
                        })
                    {
                        current.graph = RcEq::new((**current.graph).clone());
                        let graph = Rc::get_mut(&mut current.graph).unwrap();

                        let container = graph.get_mut(&container).unwrap();
                        container.assets.remove(&asset);
                    }

                    current.asset_map.remove(&asset);
                }
            },

            GraphStateAction::UpdateAsset(asset) => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container = current
                    .asset_map
                    .get(&asset.rid)
                    .expect("`Asset` `Container` not found");

                let container = graph.get_mut(&container).expect("`Container` not found");

                // TODO Ensure `Asset` exists in `Container` before update.
                container.assets.insert(asset.rid.clone(), asset.clone());
            }

            GraphStateAction::UpdateAssetPath { asset, path } => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container = current.asset_map.get(&asset).unwrap();
                let container = graph.get_mut(container).unwrap();
                let asset = container.assets.get_mut(&asset).unwrap();
                asset.path = path;
            }

            GraphStateAction::MoveAsset {
                asset,
                container,
                path,
            } => {
                current.graph = RcEq::new((**current.graph).clone());
                let graph = Rc::get_mut(&mut current.graph).unwrap();

                let container_old = current.asset_map.get(&asset).unwrap();
                let container_old = graph.get_mut(container_old).unwrap();
                let mut asset = container_old.assets.remove(&asset).unwrap();
                asset.path = path;
                current.asset_map.remove(&asset.rid);

                let container = graph.get_mut(&container).unwrap();
                let aid = asset.rid.clone();
                container.insert_asset(asset);
                current.asset_map.insert(aid, container.rid.clone());
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
                BulkUpdateAnalysisAssociationsArgs { containers, update },
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
pub type GraphStateDispatcher = UseReducerDispatcher<GraphState>;
