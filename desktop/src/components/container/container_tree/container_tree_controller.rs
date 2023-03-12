//! A container tree.
use super::ContainerTree;
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::ResourceIdArgs;
use crate::commands::project::AnalyzeArgs;
use crate::common::invoke;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::constants::MESSAGE_TIMEOUT;
use futures::stream::StreamExt;
use serde_wasm_bindgen as swb;
use std::path::PathBuf;
use std::str::FromStr;
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer};
use thot_core::types::ResourceId;
use thot_local::types::AssetFileAction;
use thot_local_database::command::container::{AddAssetInfo, AddAssetsArgs};
use thot_ui::types::ContainerPreview;
use thot_ui::types::Message;
use thot_ui::widgets::container::container_tree::ContainerPreviewSelect;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(PartialEq)]
enum AnalysisState {
    Standby,
    Analyzing,
    Complete,
    Paused,
    Error,
}

// *****************
// *** Component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct GraphControllerProps {
    /// Path to the container.
    pub root: PathBuf,
}

/// Container tree with controls.
#[function_component(ContainerTreeController)]
pub fn container_tree_controller(props: &GraphControllerProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let analysis_state = use_state(|| AnalysisState::Standby);
    let node_ref = use_node_ref(); // @todo: Remove in favor of `graph_state.dragover_container`
                                   // See https://github.com/yewstack/yew/issues/3125.

    // listen for file drop events
    {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let node_ref = node_ref.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    // @todo: Listen to window events.
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
                            panic!("more than one node is `dragover-active`");
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
                            Some(user_settings) => {
                                user_settings.general.ondrop_asset_action.clone()
                            }
                        };

                        // @todo: Handle buckets.
                        let assets = event
                            .payload
                            .into_iter()
                            .map(|path| AddAssetInfo {
                                path,
                                action: action.clone(),
                                bucket: None,
                            })
                            .collect::<Vec<AddAssetInfo>>();

                        let assets = invoke(
                            "add_assets",
                            AddAssetsArgs {
                                container: container_id.clone(),
                                assets,
                            },
                        )
                        .await
                        .expect("could not invoke `add_assets`");

                        let assets: Vec<ResourceId> = swb::from_value(assets).expect(
                            "could not convert result of `add_assets` to `Vec<ResourceId>`",
                        );

                        // update container
                        let container = invoke(
                            "get_container",
                            ResourceIdArgs {
                                rid: container_id.clone(),
                            },
                        )
                        .await
                        .expect("could not invoke `add_assets`");

                        let container: CoreContainer = swb::from_value(container)
                            .expect("could not convert result of `get_container` to `Container`");

                        let assets = container
                            .assets
                            .into_values()
                            .filter(|asset| assets.contains(&asset.rid))
                            .collect::<Vec<CoreAsset>>();

                        let num_assets = assets.len();

                        // update container
                        graph_state.dispatch(GraphStateAction::InsertContainerAssets(
                            container_id.clone(),
                            assets,
                        ));

                        // notify user
                        let num_assets_msg = if num_assets == 0 {
                            "No assets added"
                        } else if num_assets == 1 {
                            "1 asset added"
                        } else {
                            // todo[3]: Display number of assets added.
                            "Assets added"
                        };

                        app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                            Message::success(num_assets_msg),
                            MESSAGE_TIMEOUT,
                            app_state.clone(),
                        ));
                    }
                });
            },
            (),
        );
    }

    let set_preview = {
        let canvas_state = canvas_state.clone();

        Callback::from(move |preview: ContainerPreview| {
            canvas_state.dispatch(CanvasStateAction::SetPreview(preview));
        })
    };

    let analyze = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let analysis_state = analysis_state.clone();

        Callback::from(move |_: MouseEvent| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let analysis_state = analysis_state.clone();

            spawn_local(async move {
                // analyze
                let root = graph_state.graph.root();
                let max_tasks = None;
                analysis_state.set(AnalysisState::Analyzing);
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::info("Running analysis"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));

                // @todo: Handle analysis error.
                let res = invoke(
                    "analyze",
                    &AnalyzeArgs {
                        root: root.clone(),
                        max_tasks,
                    },
                )
                .await;

                // update tree
                let update = invoke("get_container", &ResourceIdArgs { rid: root.clone() })
                    .await
                    .expect("could not `get_container`");

                let update: ResourceTree<CoreContainer> = swb::from_value(update)
                    .expect("could not convert result of `get_container` to `Container`");

                graph_state.dispatch(GraphStateAction::SetGraph(update));

                analysis_state.set(AnalysisState::Complete);
                app_state.dispatch(AppStateAction::AddMessage(Message::success(
                    "Analysis complete",
                )));
            })
        })
    };

    let container_tree_fallback = html! { <Loading text={"Loading container tree"} /> };

    html! {
        <div ref={node_ref}
            class={classes!("container-tree-controller")}>

            <div class={classes!("container-tree-controls")}>
                <ContainerPreviewSelect onchange={set_preview} />
                <button
                    onclick={analyze}
                    disabled={*analysis_state == AnalysisState::Analyzing}>

                    { "Analyze" }
                </button>
            </div>

            <div class={classes!("container-tree")}>
                <Suspense fallback={container_tree_fallback}>
                    <ContainerTree root={graph_state.graph.root().clone()} />
                </Suspense>
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./container_tree_controller_test.rs"]
mod container_tree_controller_test;
