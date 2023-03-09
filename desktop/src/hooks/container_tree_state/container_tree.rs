//! Gets a `Container` tree.
use crate::commands::container::LoadContainerTreeArgs;
use crate::common::invoke;
use serde_wasm_bindgen as swb;
use std::path::Path;
use thot_core::graph::ResourceTree;
use thot_core::project::Container as CoreContainer;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

type ContainerTree = ResourceTree<CoreContainer>;

/// Gets a `Container` tree.
#[hook]
pub fn use_container_tree(root_path: &Path) -> SuspensionResult<ContainerTree> {
    let tree: UseStateHandle<Option<ContainerTree>> = use_state(|| None);
    if let Some(tree) = *tree {
        return Ok(tree.clone());
    }

    // load container
    let (s, handle) = Suspension::new();
    {
        let root_path = root_path.to_path_buf();
        let tree = tree.clone();

        spawn_local(async move {
            let container = invoke(
                "load_container_tree",
                LoadContainerTreeArgs { root: root_path },
            )
            .await
            .expect("could not invoke `load_container_tree`");

            let c_tree: ContainerTree = swb::from_value(container)
                .expect("could not convert result of `load_container_tree` to JsValue");

            tree.set(Some(c_tree));
            handle.resume();
        });
    }

    Err(s)
}

#[cfg(test)]
#[path = "./container_tree_test.rs"]
mod container_tree_test;
