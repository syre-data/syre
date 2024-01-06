//! UI for a `Container` preview within a [`Graph`](super::Graph).
//! Acts as a wrapper around a [`thot_ui::widgets::container::container_tree::Container`].
use crate::app::ShadowBox;
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::asset::remove_asset;
use crate::commands::common::open_file;
use crate::commands::container::{add_asset_windows, get_container_path};
use crate::commands::graph::{duplicate_container_tree, remove_container_tree};
use crate::components::canvas::asset::CreateAssets;
use crate::components::canvas::selection_action::{selection_action, SelectionAction};
use crate::components::canvas::{
    CanvasStateAction, CanvasStateReducer, GraphStateAction, GraphStateReducer,
};
use crate::constants::MESSAGE_TIMEOUT;
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
                        match get_container_path(rid).await {
                            Some(path) => match open_file(path).await {
                                Ok(_) => {}
                                Err(err) => {
                                    let mut msg = Message::error("Could not open file");
                                    msg.set_details(err);
                                    app_state.dispatch(AppStateAction::AddMessage(msg));
                                }
                            },

                            None => {
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
                        let dup = match duplicate_container_tree(rid.clone()).await {
                            Ok(dup) => dup,
                            Err(err) => {
                                let mut msg = Message::error("Could not duplicate tree");
                                msg.set_details(format!("{err:?}"));
                                app_state.dispatch(AppStateAction::AddMessage(msg));
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
                    spawn_local(async move {
                        match remove_container_tree(rid.clone()).await {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::debug!(err);
                                panic!("{err}");
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
                    canvas_state.dispatch(CanvasStateAction::SelectAssetOnly(rid));
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

        Callback::from(move |(asset, e): (ResourceId, MouseEvent)| {
            e.stop_propagation();
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
                let mut path = match get_container_path(rid).await {
                    Some(path) => path,
                    None => {
                        let mut msg = Message::error("Could not get container path");
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                path.push(asset.path.clone());
                match open_file(path).await {
                    Ok(_) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not open file");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                };
            });
        })
    };

    let onclick_asset_remove = {
        let app_state = app_state.clone();
        Callback::from(move |rid: ResourceId| {
            let app_state = app_state.clone();
            spawn_local(async move {
                match remove_asset(rid.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not remove asset");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                };
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

                let container_id = container_id.clone();
                let onload = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
                    let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
                    let file = file_reader.result().unwrap();
                    let file = js_sys::Uint8Array::new(&file);

                    let mut contents = vec![0; file.length() as usize];
                    file.copy_to(&mut contents);

                    let name = name.clone();
                    let container_id = container_id.clone();
                    spawn_local(async move {
                        // create assets
                        // TODO Handle buckets.
                        match add_asset_windows(container_id.clone(), name, contents).await {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::debug!(err);
                                panic!("{err}");
                            }
                        }
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
