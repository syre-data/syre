//! A container tree.
use super::ContainerTree;
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::ResourceIdArgs;
use crate::commands::project::AnalyzeArgs;
use crate::common::invoke;
use crate::components::canvas::canvas_state::ResourceType;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::constants::MESSAGE_TIMEOUT;
use futures::stream::StreamExt;
use std::path::PathBuf;
use std::str::FromStr;
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset, Container};
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
use yew_icons::{Icon, IconId};

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

/// Container tree with controls.
#[tracing::instrument(level = "debug")]
#[function_component(ContainerTreeController)]
pub fn container_tree_controller() -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let analysis_state = use_state(|| AnalysisState::Standby);
    let node_ref = use_node_ref(); // @todo: Remove in favor of `graph_state.dragover_container`
                                   // See https://github.com/yewstack/yew/issues/3125.

    let show_analyze_options = use_state(|| false);

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
                        web_sys::console::error_1(
                            &"more than one node is dragover active".into(),
                        );
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
        
                    let assets: Vec<ResourceId> = invoke(
                        "add_assets",
                        AddAssetsArgs {
                            container: container_id.clone(),
                            assets,
                        },
                    )
                    .await
                    .expect("could not invoke `add_assets`");
        
                    // update container
                    let container: Container = invoke(
                        "get_container",
                        ResourceIdArgs {
                            rid: container_id.clone(),
                        },
                    )
                    .await
                    .expect("could not invoke `get_container`");
        
                    let assets = container
                        .assets
                        .into_values()
                        .filter(|asset| assets.contains(&asset.rid))
                        .collect::<Vec<Asset>>();
        
                    // update container
                    let num_assets = assets.len();
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
        });
    }

    {
        let canvas_state = canvas_state.clone();
        let show_analyze_options = show_analyze_options.clone();
        use_effect_with(canvas_state, move |canvas_state| {
            if canvas_state.selected.len() != 1 {
                show_analyze_options.set(false);
                return;
            }
            let item = canvas_state
                .selected
                .iter()
                .next()
                .clone()
                .expect("selected has 1 item");
            let item_type = canvas_state
                .resource_type(item)
                .expect("item should have type");
            if item_type != ResourceType::Container {
                show_analyze_options.set(false);
                return;
            }
            show_analyze_options.set(true);
        })
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
        let canvas_state = canvas_state.clone();

        Callback::from(move |_: MouseEvent| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let analysis_state = analysis_state.clone();
            let project_id = canvas_state.project.clone();

            spawn_local(async move {
                let root = graph_state.graph.root();

                let max_tasks = None;
                analysis_state.set(AnalysisState::Analyzing);
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::info("Running analysis"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));

                let res = invoke(
                    "analyze",
                    &AnalyzeArgs {
                        root: root.clone(),
                        max_tasks,
                    },
                )
                .await;

                // update tree
                let update: ResourceTree<Container> =
                    invoke("load_project_graph", &ResourceIdArgs { rid: project_id })
                        .await
                        .expect("could not `load_project_graph");

                graph_state.dispatch(GraphStateAction::SetGraph(update));
                analysis_state.set(AnalysisState::Complete);

                match res {
                    Err(err) => {
                        web_sys::console::error_1(&format!("{err:?}").into());

                        let mut msg = Message::error("Error while analyzing");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }

                    Ok(()) => {
                        app_state.dispatch(AppStateAction::AddMessage(Message::success(
                            "Analysis complete",
                        )));
                    }
                }
            })
        })
    };

    let analyze_container = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let analysis_state = analysis_state.clone();
        let canvas_state = canvas_state.clone();

        Callback::from(move |_: MouseEvent| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let analysis_state = analysis_state.clone();
            let project_id = canvas_state.project.clone();

            let selected = canvas_state.selected.clone();
            let selected_rid = selected
                .iter()
                .next()
                .expect("a container should be selected")
                .clone();

            spawn_local(async move {
                let root = selected_rid;

                let max_tasks = None;
                analysis_state.set(AnalysisState::Analyzing);
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::info("Running analysis"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));

                let res = invoke(
                    "analyze",
                    &AnalyzeArgs {
                        root: root.clone(),
                        max_tasks,
                    },
                )
                .await;

                // update tree
                let update: ResourceTree<Container> =
                    invoke("load_project_graph", &ResourceIdArgs { rid: project_id })
                        .await
                        .expect("could not `load_project_graph");

                graph_state.dispatch(GraphStateAction::SetGraph(update));
                analysis_state.set(AnalysisState::Complete);

                match res {
                    Err(err) => {
                        web_sys::console::error_1(&format!("{err:?}").into());

                        let mut msg = Message::error("Error while analyzing");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }

                    Ok(()) => {
                        app_state.dispatch(AppStateAction::AddMessage(Message::success(
                            "Analysis complete",
                        )));
                    }
                }
            })
        })
    };

    let container_tree_fallback = html! { <Loading text={"Loading container tree"} /> };
    let mut primary_analyze_btn_classes = classes!("btn-primary", "primary-analyze-btn");
    if *show_analyze_options {
        primary_analyze_btn_classes.push("with_options");
    }

    html! {
        <div ref={node_ref}
            class={classes!("container-tree-controller")}>

            <div class={classes!("container-tree-controls")}>
                <ContainerPreviewSelect onchange={set_preview} />
                <div class={classes!("analyze-commands-group")}>
                    <button
                        class={primary_analyze_btn_classes}
                        onclick={analyze.clone()}
                        disabled={*analysis_state == AnalysisState::Analyzing}>

                        { "Analyze" }
                    </button>
                    if *show_analyze_options {
                        <div class={classes!("dropdown")}>
                            <button class={classes!("btn-primary", "dropdown-btn")}>
                                <Icon
                                    icon_id={IconId::FontAwesomeSolidAngleDown}
                                    height={"12px"} />
                            </button>
                            <ul class={classes!("dropdown-content")}>
                                <li class={classes!("clickable")}
                                    onclick={analyze.clone()}>
                                    { "Project" }
                                </li>
                                <li class={classes!("clickable")}
                                    onclick={analyze_container}>
                                    { "Container" }
                                </li>
                            </ul>
                        </div>
                    }
                </div>
            </div>

            <div class={classes!("container-tree")}>
                <Suspense fallback={container_tree_fallback}>
                    <ContainerTree root={graph_state.graph.root().clone()} />
                </Suspense>
            </div>
        </div>
    }
}
