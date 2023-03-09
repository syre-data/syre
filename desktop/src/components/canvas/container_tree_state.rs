//! State Redcucer for the [`ContainerTree`](super::ContainerTree).
use crate::commands::common::UpdatePropertiesArgs;
use crate::commands::container::UpdateScriptAssociationsArgs;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use thot_core::project::container::{AssetMap, ContainerStore};
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer};
use thot_core::types::ResourceId;
use thot_ui::types::ContainerPreview;
use yew::{prelude::*, set_custom_panic_hook};

pub type AssetContainerMap = HashMap<ResourceId, ResourceId>;

pub enum ContainerTreeStateAction {
    /// Set the preview state.
    SetPreview(ContainerPreview),

    /// Insert a [`Container`](CoreContainer) into the state.
    InsertContainer(CoreContainer),

    /// Insert a [`Container`](CoreContainer) tree into the state.
    InsertContainerTree(Arc<Mutex<CoreContainer>>),

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

    /// Update [`Asset`](CoreAsset)s of a [`Container`](CoreContainer).
    UpdateContainerAssets(ResourceId, AssetMap),

    /// Update a [`Container`](CoreContainer)'s
    /// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
    UpdateContainerScriptAssociations(UpdateScriptAssociationsArgs),

    /// Remove all associations with `Script` from [`Container`](CoreContainer)'s
    RemoveContainerScriptAssociations(ResourceId),

    /// Update an [`Asset`](CoreAsset).
    UpdateAsset(CoreAsset),

    SetDragOverContainer(ResourceId),
    ClearDragOverContainer,
}

#[derive(Clone)]
pub struct ContainerTreeState {
    pub preview: ContainerPreview,
    pub containers: ContainerStore, // @todo: `Container`s should not be wrapped in a `Mutex`, but currently `thot_core::project::Container` requires this.

    /// Map from an [`Asset`](CoreAsset)'s id to its [`Container`](CoreContainer)'s.
    pub asset_map: AssetContainerMap,

    /// Indicates the `Container` which currently has something dragged over it.
    /// Used to indicate which `Container` dropped files should be added to as `Asset`s.
    pub dragover_container: Option<ResourceId>,
}

impl ContainerTreeState {
    pub fn new() -> Self {
        ContainerTreeState {
            preview: ContainerPreview::None,
            containers: ContainerStore::new(),
            asset_map: HashMap::new(),
            dragover_container: None,
        }
    }
}

impl PartialEq for ContainerTreeState {
    fn eq(&self, other: &Self) -> bool {
        if self.preview != other.preview {
            return false;
        }

        // asset map
        if self.asset_map != other.asset_map {
            return false;
        }

        // containers
        if self.containers.len() != other.containers.len() {
            return false;
        }

        for rid in self.containers.keys() {
            if !other.containers.contains_key(rid) {
                return false;
            }
        }

        for (rid, container) in self.containers.iter() {
            let Some(o_container) = other.containers.get(rid) else {
                return false;
            };

            match (container, o_container) {
                (None, None) => continue,
                (Some(c), Some(oc)) => {
                    if Arc::as_ptr(c) != Arc::as_ptr(oc) {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

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

            ContainerTreeStateAction::InsertContainer(container) => {
                // map assets
                for rid in container.assets.keys() {
                    current.asset_map.insert(rid.clone(), container.rid.clone());
                }

                current
                    .containers
                    .insert(container.rid.clone(), Some(Arc::new(Mutex::new(container))));
            }

            ContainerTreeStateAction::UpdateContainerAssets(container_rid, assets) => {
                let Some(container) = current
                    .containers
                    .get(&container_rid)
                    .cloned() else {
                        panic!("`Container` not found")
                    };
                let container = container.expect("`Container` not loaded");
                let mut container = container.lock().expect("could not lock `Container`");
                container.assets = assets;
            }

            ContainerTreeStateAction::InsertContainerTree(root) => {
                flatten_container_tree(root, &mut current.containers, &mut current.asset_map);
            }

            ContainerTreeStateAction::UpdateContainerProperties(update) => {
                let mut container = current
                    .containers
                    .get(&update.rid)
                    .expect("`Container` not stored")
                    .as_ref()
                    .expect("`Container` not loaded")
                    .lock()
                    .expect("could not lock `Container`")
                    .clone();

                container.properties = update.properties;
                current
                    .containers
                    .insert(container.rid.clone(), Some(Arc::new(Mutex::new(container))));
            }

            ContainerTreeStateAction::UpdateContainerScriptAssociations(update) => {
                let mut container = current
                    .containers
                    .get(&update.rid)
                    .expect("`Container` not stored")
                    .as_ref()
                    .expect("`Container` not loaded")
                    .lock()
                    .expect("could not lock `Container`")
                    .clone();

                container.scripts = update.associations;
                current
                    .containers
                    .insert(container.rid.clone(), Some(Arc::new(Mutex::new(container))));
            }

            ContainerTreeStateAction::RemoveContainerScriptAssociations(rid) => {
                for container in current.containers.values() {
                    let Some(container) = container else { panic!("`Container` not loaded") };
                    let mut container = container.lock().expect("could not lock `Container`");
                    container.scripts.remove(&rid);

                    // @remove
                    // let containers = current.containers.get(&rid).unwrap().clone().unwrap();
                    // let containers = containers.lock().unwrap();
                    // web_sys::console::log_1(&format!("{:#?}", containers.scripts).into());
                }
            }

            ContainerTreeStateAction::InsertChildContainer(parent, child) => {
                // @todo: Set creator to user.
                let parent = current
                    .containers
                    .get(&parent)
                    .expect("parent id not found as key in store")
                    .as_ref()
                    .expect("parent container not loaded")
                    .clone();

                // map assets
                for rid in child.assets.keys() {
                    current.asset_map.insert(rid.clone(), child.rid.clone());
                }

                // insert child into store
                let c_rid = child.rid.clone();
                let child = Some(Arc::new(Mutex::new(child)));
                current.containers.insert(c_rid.clone(), child.clone());

                // add child to parent
                parent
                    .lock()
                    .expect("could not lock parent container")
                    .children
                    .insert(c_rid, child);
            }

            ContainerTreeStateAction::InsertContainerAssets(container, assets) => {
                let mut container = current
                    .containers
                    .get(&container)
                    .expect("`Container` not found")
                    .as_ref()
                    .expect("`Container` not loaded")
                    .lock()
                    .expect("could not lock `Container`")
                    .clone();

                for asset in assets {
                    current
                        .asset_map
                        .insert(asset.rid.clone(), container.rid.clone());

                    container.assets.insert(asset.rid.clone(), asset);
                }

                current
                    .containers
                    .insert(container.rid.clone(), Some(Arc::new(Mutex::new(container))));
            }

            ContainerTreeStateAction::UpdateAsset(asset) => {
                let container = current
                    .asset_map
                    .get(&asset.rid)
                    .expect("`Asset` `Container` not found");

                let mut container = current
                    .containers
                    .get(&container)
                    .expect("`Container` not found")
                    .as_ref()
                    .expect("`Container` not loaded")
                    .lock()
                    .expect("could not lock `Container`")
                    .clone();

                // @todo: Ensure `Asset` exists in `Container` before update.
                container.assets.insert(asset.rid.clone(), asset.clone());

                current
                    .containers
                    .insert(container.rid.clone(), Some(Arc::new(Mutex::new(container))));
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

/// Flattens all the nodes of a `Container` tree,
/// inserting the nodes into the store and
/// their `Asset`s into the given `Asset` map.
///
/// # Arguments
/// 1. Root [`Container`](CoreContainer) of the tree.
/// 2. [`ContainerStore`] to store all [`Container`](CoreContainer) nodes of the tree.
/// 3. [`AssetContainerMap`].
fn flatten_container_tree(
    root: Arc<Mutex<CoreContainer>>,
    containers: &mut ContainerStore,
    asset_map: &mut AssetContainerMap,
) {
    let root_c = root.lock().expect("could not obtain lock for container");
    for (rid, child) in root_c.children.iter() {
        if let Some(child) = child {
            flatten_container_tree(child.clone(), containers, asset_map);
        } else {
            containers.insert(rid.clone(), None);
        }
    }

    // map assets
    for rid in root_c.assets.keys() {
        asset_map.insert(rid.clone(), root_c.rid.clone());
    }

    containers.insert(root_c.rid.clone(), Some(root.clone()));
}

#[cfg(test)]
#[path = "./container_tree_state_test.rs"]
mod container_tree_state_test;
