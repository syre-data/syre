//! A container tree.
use super::ContainerTree;
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::container::add_assets;
use crate::components::canvas::CanvasStateReducer;
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use futures::stream::StreamExt;
use std::path::PathBuf;
use std::str::FromStr;
use thot_core::types::ResourceId;
use thot_desktop_lib::types::AddAssetInfo;
use thot_local::types::AssetFileAction;
use thot_ui::types::Message;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// *****************
// *** Component ***
// *****************

/// Container tree with controls.
#[tracing::instrument]
#[function_component(ContainerTreeController)]
pub fn container_tree_controller() -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let node_ref = use_node_ref(); // @todo: Remove in favor of `graph_state.dragover_container`
                                   // See https://github.com/yewstack/yew/issues/3125.

    // listen for file drop events
    {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let node_ref = node_ref.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                // TODO Listen to window events.
                // NOTE Used for *nix and macOS machines.
                //      For Windows machine, look in the `Container` component.
                let mut events = tauri_sys::event::listen::<Vec<PathBuf>>("tauri://file-drop")
                    .await
                    .expect("could not create `tauri://file-drop` listener");

                while let Some(event) = events.next().await {
                    // get active container id
                    let node = node_ref
                        .cast::<web_sys::HtmlElement>()
                        .expect("could not cast node to element");

                    let active_nodes = node
                        .query_selector_all(".dragover-active")
                        .expect("could not query node");

                    if active_nodes.length() > 1 {
                        tracing::error!("more than one node is dragover active");
                        graph_state.dispatch(GraphStateAction::ClearDragOverContainer);
                        app_state.dispatch(AppStateAction::AddMessage(Message::error(
                            "Could not add files",
                        )));
                        return;
                    }

                    let Some(active_node) = active_nodes.get(0) else {
                        continue;
                    };

                    let active_node = active_node
                        .dyn_ref::<web_sys::HtmlElement>()
                        .expect("could not cast node to element");

                    let container_id = active_node
                        .dataset()
                        .get("rid")
                        .expect("could not get `ResourceId` from element");

                    let container_id = ResourceId::from_str(&container_id)
                        .expect("could not convest string to `ResourceId`");

                    // create assets
                    let action = match &app_state.user_settings {
                        None => AssetFileAction::Copy,
                        Some(user_settings) => user_settings.general.ondrop_asset_action.clone(),
                    };

                    // TODO Handle buckets.
                    let assets = event
                        .payload
                        .into_iter()
                        .map(|path| AddAssetInfo {
                            path,
                            action: action.clone(),
                            bucket: None,
                        })
                        .collect::<Vec<AddAssetInfo>>();

                    match add_assets(container_id.clone(), assets).await {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::debug!(err);
                            panic!("{err}");
                        }
                    }
                }
            });
        });
    }

    let container_tree_fallback = html! { <Loading text={"Loading container tree"} /> };

    html! {
    <div ref={node_ref} class={classes!("container-tree-controller")}>
        <div class={classes!("container-tree")}>
            <Suspense fallback={container_tree_fallback}>
                <ContainerTree root={graph_state.graph.root().clone()} />
            </Suspense>
        </div>
    </div>
    }
}
