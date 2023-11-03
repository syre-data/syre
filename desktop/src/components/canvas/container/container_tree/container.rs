//! UI for a `Container` preview within a [`Graph`](super::Graph).
//! Acts as a wrapper around a [`thot_ui::widgets::container::container_tree::Container`].
use super::super::super::asset::CreateAssets;
use crate::app::ShadowBox;
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::common::{PathBufArgs, ResourceIdArgs};
use crate::commands::container::AddAssetWindowsArgs;
use crate::common::invoke;
use crate::components::canvas::{
    CanvasStateAction, CanvasStateReducer, GraphStateAction, GraphStateReducer,
};
use crate::constants::{MESSAGE_TIMEOUT, SCRIPT_DISPLAY_NAME_MAX_LENGTH};
use crate::routes::Route;
use std::path::PathBuf;
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset as CoreAsset, Container as CoreContainer};
use thot_core::types::{ResourceId, ResourceMap};
use thot_ui::types::Message;
use thot_ui::widgets::container::container_tree::{
    container::ContainerMenuEvent, container::ContainerProps as ContainerUiProps,
    Container as ContainerUi,
};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileReader;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

type Graph = ResourceTree<CoreContainer>;

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub rid: ResourceId,

    #[prop_or_default]
    pub r#ref: NodeRef,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,
}

#[tracing::instrument(skip(props), fields(?props.rid))]
#[function_component(Container)]
pub fn container(props: &ContainerProps) -> HtmlResult {
    // -------------
    // --- setup ---
    // -------------
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state = use_context::<GraphStateReducer>().expect("`GraphReducer` context not found");
    let is_root = &props.rid == graph_state.graph.root();

    let navigator = use_navigator().expect("navigator not found");
    let show_create_assets = use_state(|| false);
    let selected = canvas_state.selected.contains(&props.rid);
    let multiple_selected = canvas_state.selected.len() > 1;

    let Some(project_scripts) = projects_state.project_scripts.get(&canvas_state.project) else {
        app_state.dispatch(AppStateAction::AddMessage(Message::error(
            "Project scripts not loaded",
        )));
        navigator.push(&Route::Dashboard);
        return Ok(html! {{ "Project scripts not loaded. Redirecting to home." }});
    };

    let script_names = project_scripts
        .iter()
        .map(|(rid, script)| {
            let mut name = script.name.clone().unwrap_or(
                Into::<PathBuf>::into(script.path.clone())
                    .file_name()
                    .expect("could not get `Script`'s file name")
                    .to_str()
                    .expect("could not convert file name to str")
                    .to_string(),
            );

            if name.len() > SCRIPT_DISPLAY_NAME_MAX_LENGTH {
                name.replace_range(0..(SCRIPT_DISPLAY_NAME_MAX_LENGTH + 3), "...");
            };

            (rid.clone(), name)
        })
        .collect::<ResourceMap<String>>();

    // -------------------
    // --- interaction ---
    // -------------------

    let onmousedown = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
    });

    let onclick = {
        let rid = props.rid.clone();
        let canvas_state = canvas_state.clone();
        let selected = selected.clone();
        let multiple_selected = multiple_selected.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            e.prevent_default();

            let rid = rid.clone();
            match selection_action(selected, multiple_selected, e) {
                SelectionAction::SelectOnly => {
                    canvas_state.dispatch(CanvasStateAction::ClearSelected);
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(rid));
                }

                SelectionAction::Select => {
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(rid));
                }

                SelectionAction::Unselect => {
                    canvas_state.dispatch(CanvasStateAction::Unselect(rid));
                }
            }
        })
    };

    // -------------------
    // --- menu events ---
    // -------------------

    let on_menu_event = {
        let app_state = app_state.clone();
        let canvas_state = canvas_state.clone();
        let graph_state = graph_state.clone();
        let rid = props.rid.clone();
        let show_create_assets = show_create_assets.clone();

        Callback::from(move |event: ContainerMenuEvent| {
            let rid = rid.clone();
            match event {
                ContainerMenuEvent::AddAssets => show_create_assets.set(true),

                ContainerMenuEvent::OpenFolder => {
                    let app_state = app_state.clone();

                    spawn_local(async move {
                        match invoke::<PathBuf>("get_container_path", ResourceIdArgs { rid }).await
                        {
                            Ok(path) => {
                                match invoke::<()>("open_file", PathBufArgs { path }).await {
                                    Ok(_) => {}
                                    Err(_err) => app_state.dispatch(AppStateAction::AddMessage(
                                        Message::error("Could not open file"),
                                    )),
                                }
                            }

                            Err(_err) => {
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not get file path",
                                )));
                            }
                        }
                    });
                }

                ContainerMenuEvent::DuplicateTree => {
                    let app_state = app_state.clone();
                    let graph_state = graph_state.clone();

                    spawn_local(async move {
                        let dup = invoke::<Graph>(
                            "duplicate_container_tree",
                            ResourceIdArgs { rid: rid.clone() },
                        )
                        .await;

                        let dup = match dup {
                            Ok(dup) => dup,
                            Err(err) => {
                                web_sys::console::error_1(&format!("{err:?}").into());
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not duplicate tree",
                                )));
                                return;
                            }
                        };

                        let mut graph = graph_state.graph.clone();
                        let parent = graph
                            .parent(&rid)
                            .expect("parent `Container` not found")
                            .expect("could not get `Container` parent")
                            .clone();

                        match graph.insert_tree(&parent, dup) {
                            Ok(_) => graph_state.dispatch(GraphStateAction::SetGraph(graph)),
                            Err(_) => {
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not duplicate tree",
                                )));
                            }
                        }
                    });
                }

                ContainerMenuEvent::Remove => {
                    let app_state = app_state.clone();
                    let canvas_state = canvas_state.clone();
                    let graph_state = graph_state.clone();

                    spawn_local(async move {
                        invoke::<()>("remove_container_tree", ResourceIdArgs { rid: rid.clone() })
                            .await
                            .expect("could not invoke `remove_container_tree`");

                        let mut graph = graph_state.graph.clone();
                        match graph.remove(&rid) {
                            Ok(removed) => {
                                // unselect removed elements
                                let mut rids = Vec::with_capacity(removed.nodes().len());
                                for (cid, container) in removed.nodes() {
                                    rids.push(cid.clone());
                                    let mut aids = container
                                        .assets
                                        .keys()
                                        .map(|rid| rid.clone())
                                        .collect::<Vec<ResourceId>>();

                                    rids.append(&mut aids);
                                }

                                canvas_state.dispatch(CanvasStateAction::UnselectMany(rids));
                                graph_state.dispatch(GraphStateAction::SetGraph(graph));
                            }
                            Err(_) => {
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not remove tree",
                                )));
                            }
                        }
                    });
                }
            }
        })
    };

    let close_create_asset = {
        let show_create_assets = show_create_assets.clone();

        Callback::from(move |_: MouseEvent| {
            show_create_assets.set(false);
        })
    };

    // --------------
    // --- assets ---
    // --------------

    let onclick_asset = {
        let app_state = app_state.clone();
        let canvas_state = canvas_state.clone();
        let graph_state = graph_state.clone();
        let multiple_selected = multiple_selected.clone();

        Callback::from(move |(asset, e): (ResourceId, MouseEvent)| {
            let Some(asset) = get_asset(&asset, graph_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::error("Could not load asset"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));
                return;
            };

            let rid = asset.rid.clone();
            let selected = canvas_state.selected.contains(&rid);
            match selection_action(selected, multiple_selected, e) {
                SelectionAction::SelectOnly => {
                    canvas_state.dispatch(CanvasStateAction::ClearSelected);
                    canvas_state.dispatch(CanvasStateAction::SelectAsset(rid));
                }

                SelectionAction::Select => {
                    canvas_state.dispatch(CanvasStateAction::SelectAsset(rid));
                }

                SelectionAction::Unselect => {
                    canvas_state.dispatch(CanvasStateAction::Unselect(rid));
                }
            }
        })
    };

    let ondblclick_asset = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let rid = props.rid.clone();

        Callback::from(move |(asset, _e): (ResourceId, MouseEvent)| {
            let rid = rid.clone();
            let Some(asset) = get_asset(&asset, graph_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::error("Could not load asset"),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));
                return;
            };

            let app_state = app_state.clone();
            spawn_local(async move {
                let Ok(mut path) =
                    invoke::<PathBuf>("get_container_path", ResourceIdArgs { rid }).await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not get container path",
                    )));
                    return;
                };

                path.push(asset.path.clone());
                let Ok(_) = invoke::<()>("open_file", PathBufArgs { path }).await else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not open file",
                    )));
                    return;
                };
            });
        })
    };

    let onclick_asset_remove = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let canvas_state = canvas_state.clone();
        let container_id = props.rid.clone();

        Callback::from(move |rid: ResourceId| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let canvas_state = canvas_state.clone();
            let container_id = container_id.clone();

            spawn_local(async move {
                let Ok(_) = invoke::<()>("remove_asset", ResourceIdArgs { rid: rid.clone() }).await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not remove asset",
                    )));
                    return;
                };

                let Some(_container) = graph_state.graph.get(&container_id).cloned() else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not update container",
                    )));
                    return;
                };

                graph_state.dispatch(GraphStateAction::RemoveAsset(rid.clone()));
                canvas_state.dispatch(CanvasStateAction::Unselect(rid.clone()));
            });
        })
    };

    let onadd_assets = {
        let show_create_assets = show_create_assets.clone();

        Callback::from(move |_: ()| {
            show_create_assets.set(false);
        })
    };

    // ---------------
    // --- scripts ---
    // ---------------

    // ----------------------
    // --- on drop events ---
    // ----------------------

    // NOTE Used for Windows machines.
    //      For *nix and macOS machine, look in the `ContainerTreeController` component.

    let ondragenter = {
        let graph_state = graph_state.clone();
        let container_id = props.rid.clone();

        Callback::from(move |_: DragEvent| {
            graph_state.dispatch(GraphStateAction::SetDragOverContainer(container_id.clone()));
        })
    };

    let ondragleave = {
        let graph_state = graph_state.clone();

        Callback::from(move |_: DragEvent| {
            graph_state.dispatch(GraphStateAction::ClearDragOverContainer);
        })
    };

    let ondrop = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let container_id = props.rid.clone();

        Callback::from(move |e: DragEvent| {
            graph_state.dispatch(GraphStateAction::ClearDragOverContainer);

            // drag and drop on Windows
            let drop_data = e.data_transfer().unwrap();
            let files = drop_data.files().unwrap();
            for index in 0..files.length() {
                let file = files.item(index).expect("could not get `File`");
                let name = file.name();

                let file_reader = web_sys::FileReader::new().unwrap();
                file_reader.read_as_array_buffer(&file).unwrap();

                let app_state = app_state.clone();
                let graph_state = graph_state.clone();
                let container_id = container_id.clone();
                let onload = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
                    let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
                    let file = file_reader.result().unwrap();
                    let file = js_sys::Uint8Array::new(&file);

                    let mut contents = vec![0; file.length() as usize];
                    file.copy_to(&mut contents);

                    let app_state = app_state.clone();
                    let graph_state = graph_state.clone();
                    let name = name.clone();
                    let container_id = container_id.clone();
                    spawn_local(async move {
                        // create assets
                        // TODO Handle buckets.
                        let asset: Vec<ResourceId> = invoke(
                            "add_asset_windows",
                            AddAssetWindowsArgs {
                                container: container_id.clone(),
                                name,
                                contents,
                            },
                        )
                        .await
                        .expect("could not invoke `add_asset_windows`");

                        // update container
                        let container: CoreContainer = invoke(
                            "get_container",
                            ResourceIdArgs {
                                rid: container_id.clone(),
                            },
                        )
                        .await
                        .expect("could not invoke `get_container`");

                        // update container
                        let asset = container
                            .assets
                            .get(&asset[0])
                            .expect("could not find `Asset`")
                            .clone();

                        graph_state.dispatch(GraphStateAction::InsertContainerAssets(
                            container_id.clone(),
                            vec![asset],
                        ));

                        app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                            Message::success("Added asset"),
                            MESSAGE_TIMEOUT,
                            app_state.clone(),
                        ));
                    });
                });

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
            }
        })
    };

    // ----------
    // --- ui ---
    // ----------

    // props
    let container = graph_state
        .graph
        .get(&props.rid)
        .expect("`Container` not found");

    let mut class = Classes::new();
    if selected {
        class.push("selected");
    }

    let c_props = props! {
            ContainerUiProps {
                r#ref: props.r#ref.clone(),
                class,
                visible: canvas_state.is_visible(&container.rid),
                is_root,
                rid: props.rid.clone(),
                properties: container.properties.clone(),
                assets: container.assets.clone(),
                active_assets: canvas_state.selected.clone(),
                scripts: container.scripts.clone(),
                script_names,
                preview: canvas_state.preview.clone(),
                onmousedown,
                onclick,
                onclick_asset,
                ondblclick_asset,
                onclick_asset_remove,
                onadd_child: props.onadd_child.clone(),
                on_menu_event,
                ondragenter,
                ondragleave,
                ondrop,
            }
    };

    let container_name = &container.properties.name;

    Ok(html! {
        <>
        <ContainerUi ..c_props />
        if *show_create_assets {
            <ShadowBox
                title={format!("Add Asset to {container_name}")}
                onclose={close_create_asset}>

                <CreateAssets
                    container={container.rid.clone()}
                    onsuccess={onadd_assets} />
            </ShadowBox>
        }
        </>
    })
}

// ***************
// *** helpers ***
// ***************

enum SelectionAction {
    SelectOnly,
    Select,
    Unselect,
}

/// Determines the selection action from the current action and state.
///
/// # Arguments
/// 1. If the clicked resource is currently selected.
/// 2. If at least one other resource is currently selected.
/// 3. The [`MouseEvent`].
fn selection_action(selected: bool, multiple: bool, e: MouseEvent) -> SelectionAction {
    if e.shift_key() {
        if selected {
            return SelectionAction::Unselect;
        } else {
            return SelectionAction::Select;
        }
    }

    if selected {
        if multiple {
            return SelectionAction::SelectOnly;
        }

        return SelectionAction::Unselect;
    }

    SelectionAction::SelectOnly
}

/// Gets an `Asset`.
///
/// # Arguments
/// 1. `Asset`'s `ResourceId`.
/// 2. Tree state.
fn get_asset(rid: &ResourceId, graph_state: GraphStateReducer) -> Option<CoreAsset> {
    let Some(container) = graph_state.asset_map.get(&rid) else {
        return None;
    };

    let container = graph_state
        .graph
        .get(&container)
        .expect("`Container` not found");

    container.assets.get(&rid).cloned()
}
