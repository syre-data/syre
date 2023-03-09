//! State Redcucer for the [`ContainerTree`](super::ContainerTree).
use crate::commands::common::UpdatePropertiesArgs;
use crate::commands::container::UpdateScriptAssociationsArgs;
use std::collections::HashMap;
use std::rc::Rc;
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer};
use thot_core::types::ResourceId;
use thot_ui::types::ContainerPreview;
use yew::prelude::*;

type ContainerTree = ResourceTree<CoreContainer>;

pub type AssetContainerMap = HashMap<ResourceId, ResourceId>;

pub enum ContainerTreeStateAction {
    /// Set the preview state.
    SetPreview(ContainerPreview),

    ///  Sets the [`ContainerTree`].
    SetTree(ContainerTree),

    /// Update a [`Container`](CoreContainer)'s [`StandardProperties`](thot_core::project::StandardProperties).
    UpdateContainerProperties(UpdatePropertiesArgs),

    /// Add a [`Container`](CoreContainer) as a child.
    ///
    /// # Fields
    /// #. `parent`: `ResourceId` of the parent.
    /// #. `child`: Child `Container`.
    InsertChildContainer(ResourceId, CoreContainer),

    /// Insert [`Asset`](CoreAsset)s into a [`Container`](CoreContainer).
    InsertContainerAssets(ResourceId, Vec<CoreAsset>),

    /// Update a [`Container`](CoreContainer)'s
    /// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
    UpdateContainerScriptAssociations(UpdateScriptAssociationsArgs),

    /// Update an [`Asset`](CoreAsset).
    UpdateAsset(CoreAsset),

    SetDragOverContainer(ResourceId),
    ClearDragOverContainer,
}

#[derive(PartialEq, Clone)]
pub struct ContainerTreeState {
    pub tree: ContainerTree,
    pub preview: ContainerPreview,

    /// Map from an [`Asset`](CoreAsset)'s id to its [`Container`](CoreContainer)'s.
    pub asset_map: AssetContainerMap,

    /// Indicates the `Container` which currently has something dragged over it.
    /// Used to indicate which `Container` dropped files should be added to as `Asset`s.
    pub dragover_container: Option<ResourceId>,
}

impl ContainerTreeState {
    pub fn new(tree: ContainerTree) -> Self {
        ContainerTreeState {
            tree,
            preview: ContainerPreview::None,
            asset_map: HashMap::new(),
            dragover_container: None,
        }
    }
}

// @remove
// impl PartialEq for ContainerTreeState {
//     fn eq(&self, other: &Self) -> bool {
//         if self.preview != other.preview {
//             return false;
//         }

//         // asset map
//         if self.asset_map != other.asset_map {
//             return false;
//         }

//         // containers
//         if self.tree != other.preview
//         true
//     }
// }

impl Reducible for ContainerTreeState {
    type Action = ContainerTreeStateAction;

    // @note: Actions that change a `Container` must first `clone`
    // the `Container` then re-`insert` it into the state's `Container` store
    // so equality can be evaluated.
    // A `Container`'s value must never be changed in place.
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();

        match action {
            ContainerTreeStateAction::SetPreview(preview) => {
                current.preview = preview;
            }

            ContainerTreeStateAction::SetTree(tree) => {
                current.tree = tree;
            }

            ContainerTreeStateAction::UpdateContainerProperties(update) => {
                let container = current
                    .tree
                    .get_mut(&update.rid)
                    .expect("`Container` not found");

                container.properties = update.properties;
            }

            ContainerTreeStateAction::UpdateContainerScriptAssociations(update) => {
                let mut container = current
                    .tree
                    .get_mut(&update.rid)
                    .expect("`Container` not found");

                container.scripts = update.associations;
            }

            ContainerTreeStateAction::InsertChildContainer(parent, child) => {
                // map assets
                for rid in child.assets.keys() {
                    current.asset_map.insert(rid.clone(), child.rid.clone());
                }

                // insert child into store
                current.tree.insert(parent, child);
            }

            ContainerTreeStateAction::InsertContainerAssets(container, assets) => {
                let mut container = current
                    .tree
                    .get_mut(&container)
                    .expect("`Container` not found");

                for asset in assets {
                    current
                        .asset_map
                        .insert(asset.rid.clone(), container.rid.clone());

                    container.assets.insert(asset.rid.clone(), asset);
                }
            }

            ContainerTreeStateAction::UpdateAsset(asset) => {
                let container = current
                    .asset_map
                    .get(&asset.rid)
                    .expect("`Asset` `Container` not found");

                let mut container = current
                    .tree
                    .get_mut(&container)
                    .expect("`Container` not found");

                // @todo: Ensure `Asset` exists in `Container` before update.
                container.assets.insert(asset.rid.clone(), asset.clone());
            }

            ContainerTreeStateAction::SetDragOverContainer(rid) => {
                current.dragover_container = Some(rid);
            }

            ContainerTreeStateAction::ClearDragOverContainer => {
                current.dragover_container = None;
            }
        };

        current.into()
    }
}

pub type ContainerTreeStateReducer = UseReducerHandle<ContainerTreeState>;

#[cfg(test)]
#[path = "./container_tree_state_test.rs"]
mod container_tree_state_test;
