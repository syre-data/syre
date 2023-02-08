//! A container tree.
use super::ContainerTree as ContainerTreeUi;
use crate::commands::container::LoadContainerTreeArgs;
use crate::common::invoke;
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use serde_wasm_bindgen as swb;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thot_core::project::Container as CoreContainer;
use thot_ui::types::ContainerPreview;
use thot_ui::widgets::container::container_tree::ContainerPreviewSelect;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// *****************
// *** Component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct ContainerTreeControllerProps {
    /// Path to the container.
    pub root: PathBuf,
}

/// Container tree with controls.
#[function_component(ContainerTreeController)]
pub fn container_tree_controller(props: &ContainerTreeControllerProps) -> Html {
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeStateReducer` context not found");

    let root = use_state(|| None);

    {
        // load container
        let tree_state = tree_state.clone();
        let root = root.clone();
        let root_path = props.root.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let container = invoke(
                        "load_container_tree",
                        LoadContainerTreeArgs { root: root_path },
                    )
                    .await
                    .expect("could not invoke `load_container_tree`");

                    let container: CoreContainer = swb::from_value(container)
                        .expect("could not convert result of `load_container_tree` to JsValue");

                    let rid = container.rid.clone();
                    let container = Arc::new(Mutex::new(container));
                    tree_state.dispatch(ContainerTreeStateAction::InsertContainerTree(container));

                    root.set(Some(rid));
                });
            },
            (),
        );
    }

    let set_preview = {
        let tree_state = tree_state.clone();

        Callback::from(move |preview: ContainerPreview| {
            tree_state.dispatch(ContainerTreeStateAction::SetPreview(preview));
        })
    };

    let container_tree_fallback = html! { <Loading text={"Loading container tree"} /> };

    html! {
        <div class={classes!("container-tree-controller")}>
            <div class={classes!("container-tree-controls")}>
                <ContainerPreviewSelect onchange={set_preview} />
            </div>
            <div class={classes!("container-tree")}>
                if let Some(root) = (*root).clone() {
                    <Suspense fallback={container_tree_fallback}>
                        <ContainerTreeUi {root} />
                    </Suspense>
                } else {
                    { container_tree_fallback }
                }
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./container_tree_controller_test.rs"]
mod container_tree_controller_test;
