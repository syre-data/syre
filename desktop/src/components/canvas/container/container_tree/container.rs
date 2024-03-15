//! UI for a `Container` preview within a [`Graph`](super::Graph).
//! Acts as a wrapper around a [`syre_ui::widgets::container::container_tree::Container`].
use crate::actions::container::Action as ContainerAction;
use crate::app::ShadowBox;
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::asset::remove_asset;
use crate::commands::common::open_file;
use crate::commands::container::{
    add_asset_from_contents, get_container_path, update_analysis_associations,
    UpdateAnalysisAssociationsArgs,
};
use crate::commands::graph::{duplicate_container_tree, remove_container_tree};
use crate::components::canvas::asset::CreateAssets;
use crate::components::canvas::selection_action::{selection_action, SelectionAction};
use crate::components::canvas::{
    CanvasStateAction, CanvasStateReducer, GraphStateAction, GraphStateReducer,
};
use crate::constants::MESSAGE_TIMEOUT;
use crate::routes::Route;
use std::collections::HashMap;
use syre_core::project::{AnalysisAssociation, Asset, RunParameters};
use syre_core::types::{ResourceId, ResourceMap};
use syre_desktop_lib::error::{RemoveResource, Trash as TrashError};
use syre_local::types::AnalysisKind;
use syre_ui::types::Message;
use syre_ui::widgets::container::container_tree::container::{
    Container as ContainerUi, ContainerMenuEvent, ContainerProps as ContainerUiProps, Flags,
};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileReader;
use yew::prelude::*;
use yew::props;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub rid: ResourceId,

    #[prop_or_default]
    pub r#ref: NodeRef,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,
}

#[function_component(Container)]
pub fn container(props: &ContainerProps) -> HtmlResult {
    // -------------
    // --- setup ---
    // -------------
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let is_root = &props.rid == graph_state.graph.root();

    let navigator = use_navigator().unwrap();
    let show_create_assets = use_state(|| false);
    let selected = canvas_state.selected.contains(&props.rid);
    let multiple_selected = canvas_state.selected.len() > 1;

    let Some(project_scripts) = projects_state.project_analyses.get(&canvas_state.project) else {
        app_state.dispatch(AppStateAction::AddMessage(Message::error(
            "Project scripts not loaded.",
        )));

        navigator.push(&Route::Dashboard);
        return Ok(html! {{ "Project scripts not loaded. Redirecting to home." }});
    };

    let analysis_names = project_scripts
        .values()
        .map(|analysis| match analysis {
            AnalysisKind::Script(script) => {
                let name = script
                    .name
                    .clone()
                    .unwrap_or(script.path.to_string_lossy().to_string());

                (script.rid.clone(), name)
            }

            AnalysisKind::ExcelTemplate(template) => {
                let name = template
                    .name
                    .clone()
                    .unwrap_or(template.template.path.to_string_lossy().to_string());

                (template.rid.clone(), name)
            }
        })
        .collect::<ResourceMap<String>>();

    // -------------------
    // --- interaction ---
    // -------------------

    let onmousedown = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
    });

    let onclick = use_callback(
        (
            props.rid.clone(),
            selected.clone(),
            multiple_selected.clone(),
        ),
        {
            let canvas_state = canvas_state.dispatcher();
            move |e: MouseEvent, (rid, selected, multiple_selected)| {
                e.stop_propagation();
                e.prevent_default();

                let rid = rid.clone();
                match selection_action(*selected, *multiple_selected, e) {
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
            }
        },
    );

    // -------------------
    // --- menu events ---
    // -------------------

    let on_menu_event = use_callback((props.rid.clone(), graph_state.graph.clone()), {
        let app_state = app_state.dispatcher();
        let graph_state = graph_state.dispatcher();
        let show_create_assets = show_create_assets.setter();

        move |event, (rid, graph)| {
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
                                    let mut msg = Message::error("Could not open file.");
                                    msg.set_details(err);
                                    app_state.dispatch(AppStateAction::AddMessage(msg));
                                }
                            },

                            None => {
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not get file path.",
                                )));
                            }
                        }
                    });
                }

                ContainerMenuEvent::DuplicateTree => {
                    let app_state = app_state.clone();
                    let graph_state = graph_state.clone();
                    let graph = graph.clone();

                    spawn_local(async move {
                        let dup = match duplicate_container_tree(rid.clone()).await {
                            Ok(dup) => dup,
                            Err(err) => {
                                let mut msg = Message::error("Could not duplicate tree.");
                                msg.set_details(format!("{err:?}"));
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                                return;
                            }
                        };

                        let mut graph = (**graph).clone();
                        let parent = graph
                            .parent(&rid)
                            .expect("parent `Container` not found")
                            .expect("could not get `Container` parent")
                            .clone();

                        match graph.insert_tree(&parent, dup) {
                            Ok(_) => graph_state.dispatch(GraphStateAction::SetGraph(graph)),
                            Err(_) => {
                                app_state.dispatch(AppStateAction::AddMessage(Message::error(
                                    "Could not duplicate tree.",
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
                                panic!("{err}");
                            }
                        }
                    });
                }
            }
        }
    });

    let close_create_asset = use_callback((), {
        let show_create_assets = show_create_assets.setter();
        move |_, _| {
            show_create_assets.set(false);
        }
    });

    // --------------
    // --- assets ---
    // --------------

    let onclick_asset = use_callback(
        (
            app_state.clone(),
            canvas_state.clone(),
            graph_state.clone(),
            multiple_selected.clone(),
        ),
        move |(asset, e), (app_state, canvas_state, graph_state, multiple_selected)| {
            let Some(asset) = get_asset(&asset, graph_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::error("Could not load asset."),
                    MESSAGE_TIMEOUT,
                    app_state.clone(),
                ));
                return;
            };

            let rid = asset.rid.clone();
            let selected = canvas_state.selected.contains(&rid);
            match selection_action(selected, *multiple_selected, e) {
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
        },
    );

    let ondblclick_asset = use_callback(
        (props.rid.clone(), app_state.clone(), graph_state.clone()),
        move |(asset, e): (ResourceId, MouseEvent), (rid, app_state, graph_state)| {
            e.stop_propagation();
            let rid = rid.clone();
            let Some(asset) = get_asset(&asset, graph_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                    Message::error("Could not load asset."),
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
                        let msg = Message::error("Could not get container path.");
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                        return;
                    }
                };

                path.push(asset.path.clone());
                match open_file(path).await {
                    Ok(_) => {}
                    Err(err) => {
                        let mut msg = Message::error("Could not open file.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                };
            });
        },
    );

    let onclick_asset_remove = use_callback((), {
        let graph_state = graph_state.dispatcher();
        let app_state = app_state.dispatcher();

        move |rid: ResourceId, _| {
            let graph_state = graph_state.clone();
            let app_state = app_state.clone();

            spawn_local(async move {
                match remove_asset(rid.clone()).await {
                    Ok(_) => {
                        // NOTE: File removal should be picked up by file system watcher
                        //  which will trigger an event taking care of remvong from grpah.
                        //  May want to add graph state removal here anyways in case file removal not picked up.
                        // graph_state.dispatch(GraphStateAction::RemoveAsset(rid.clone()));
                    }

                    Err(err) => {
                        tracing::debug!(?err);
                        match err {
                            RemoveResource::ZMQ(err) => {
                                let mut msg = Message::error("Could not remove asset.");
                                msg.set_details(err);
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }

                            RemoveResource::Trash(TrashError::NotFound) => {
                                let msg = Message::warning("Asset file was not found.");
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                                graph_state.dispatch(GraphStateAction::RemoveAsset(rid.clone()));
                            }

                            RemoveResource::Trash(TrashError::PermissionDenied) => {
                                let mut msg = Message::warning("Asset file could not be removed.");
                                msg.set_details("This is often because the file is being used by another program.");
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                                graph_state.dispatch(GraphStateAction::RemoveAsset(rid.clone()));
                            }

                            RemoveResource::Trash(TrashError::Other(err)) => {
                                let mut msg = Message::error("Could not remove asset.");
                                msg.set_details(err);
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }

                            RemoveResource::Database(err) => {
                                let mut msg = Message::error("Could not remove asset.");
                                msg.set_details(err);
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }
                        }
                    }
                };
            });
        }
    });

    let onadd_assets = use_callback((), {
        let show_create_assets = show_create_assets.setter();
        move |_, _| {
            show_create_assets.set(false);
        }
    });

    // ---------------
    // --- scripts ---
    // ---------------

    let onchange_analysis_association = use_callback((props.rid.clone(), graph_state.clone()), {
        let app_state = app_state.dispatcher();
        move |assoc: AnalysisAssociation, (container_id, graph_state)| {
            let Some(container) = graph_state.graph.get(container_id) else {
                let mut msg = Message::error("Could not add Script association.");
                msg.set_details("Container not found in graph.");
                app_state.dispatch(AppStateAction::AddMessage(msg));
                return;
            };

            let mut associations = container.analyses.clone();
            associations.insert(assoc.analysis.clone(), assoc.into());
            let app_state = app_state.clone();
            let graph_state = graph_state.dispatcher();
            let container_id = container_id.clone();
            spawn_local(async move {
                match update_analysis_associations(container_id.clone(), associations.clone()).await
                {
                    Ok(_) => {
                        graph_state.dispatch(
                            GraphStateAction::UpdateContainerAnalysisAssociations(
                                UpdateAnalysisAssociationsArgs {
                                    rid: container_id.clone(),
                                    associations,
                                },
                            ),
                        );
                    }

                    Err(err) => {
                        let mut msg = Message::error("Could not add analysis association.");
                        msg.set_details(format!("{err}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        }
    });

    let onremove_analysis_association = use_callback((props.rid.clone(), graph_state.clone()), {
        let app_state = app_state.dispatcher();
        move |analysis: ResourceId, (container_id, graph_state)| {
            let Some(container) = graph_state.graph.get(container_id) else {
                let mut msg = Message::error("Could not remove analysis association.");
                msg.set_details("Container not found in graph.");
                app_state.dispatch(AppStateAction::AddMessage(msg));
                return;
            };

            let mut associations = container.analyses.clone();
            associations.remove(&analysis);
            let app_state = app_state.clone();
            let graph_state = graph_state.dispatcher();
            let container_id = container_id.clone();
            spawn_local(async move {
                match update_analysis_associations(container_id.clone(), associations.clone()).await
                {
                    Ok(_) => {
                        graph_state.dispatch(
                            GraphStateAction::UpdateContainerAnalysisAssociations(
                                UpdateAnalysisAssociationsArgs {
                                    rid: container_id.clone(),
                                    associations,
                                },
                            ),
                        );
                    }

                    Err(err) => {
                        let mut msg = Message::error("Could not add Script association.");
                        msg.set_details(format!("{err}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        }
    });

    // ----------------------
    // --- on drop events ---
    // ----------------------

    let ondrop = use_callback((props.rid.clone(), graph_state.graph.clone()), {
        let graph_state = graph_state.dispatcher();
        move |e: DragEvent, (container_id, graph)| {
            let drop_data = e.data_transfer().unwrap();
            let action = drop_data.get_data("application/json").unwrap();

            let action: Option<ContainerAction> = serde_json::from_str(&action).ok();
            if let Some(action) = action {
                match action {
                    ContainerAction::AddScriptAssociation(script) => 'add_script: {
                        let container = graph.get(&container_id).unwrap();
                        if container.analyses.contains_key(&script) {
                            break 'add_script;
                        }

                        let mut associations = container.analyses.clone();
                        associations.insert(script, RunParameters::new());

                        let app_state = app_state.dispatcher();
                        let graph_state = graph_state.clone();
                        let container_id = container_id.clone();
                        spawn_local(async move {
                            match update_analysis_associations(
                                container_id.clone(),
                                associations.clone(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    graph_state.dispatch(
                                        GraphStateAction::UpdateContainerAnalysisAssociations(
                                            UpdateAnalysisAssociationsArgs {
                                                rid: container_id,
                                                associations,
                                            },
                                        ),
                                    );
                                }

                                Err(err) => {
                                    let mut msg =
                                        Message::error("Could not add Script association.");

                                    msg.set_details(format!("{err}"));
                                    app_state.dispatch(AppStateAction::AddMessage(msg));
                                }
                            }
                        });
                    }
                }
            }

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
                        match add_asset_from_contents(container_id.clone(), name, contents).await {
                            Ok(_) => {}
                            Err(err) => {
                                panic!("{err}");
                            }
                        }
                    });
                });

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
            }
        }
    });

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

    let container_flags = canvas_state
        .flags
        .get(&props.rid)
        .cloned()
        .unwrap_or_default();

    let mut asset_flags = HashMap::new();
    for asset in container.assets.keys() {
        if let Some(a_flags) = canvas_state.flags.get(asset) {
            asset_flags.insert(asset.clone(), a_flags.clone());
        }
    }

    let flags = Flags {
        container: container_flags,
        assets: asset_flags,
    };

    let c_props = props! {
            ContainerUiProps {
                r#ref: props.r#ref.clone(),
                class,
                visible: canvas_state.is_visible(&container.rid),
                is_root,
                rid: props.rid.clone(),
                properties: container.properties.clone(),
                assets: container.assets.clone(),
                flags,
                active_assets: canvas_state.selected.clone(),
                analyses: container.analyses.clone(),
                analysis_names,
                preview: canvas_state.preview.clone(),
                onmousedown,
                onclick,
                onclick_asset,
                ondblclick_asset,
                onclick_asset_remove,
                onchange_analysis_association,
                onremove_analysis_association,
                onadd_child: props.onadd_child.clone(),
                on_menu_event,
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
fn get_asset(rid: &ResourceId, graph_state: GraphStateReducer) -> Option<Asset> {
    let Some(container) = graph_state.asset_map.get(&rid) else {
        return None;
    };

    let container = graph_state
        .graph
        .get(&container)
        .expect("`Container` not found");

    container.assets.get(&rid).cloned()
}
