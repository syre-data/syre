//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
//! Acts as a wrapper around a [`thot_ui::widgets::container::container_tree::Container`].
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::common::{PathBufArgs, ResourceIdArgs};
use crate::common::invoke;
use crate::components::asset::CreateAssets;
use crate::components::canvas::{
    CanvasStateAction, CanvasStateReducer, ContainerTreeStateAction, ContainerTreeStateReducer,
};
use crate::components::details_bar::DetailsBarWidget;
use crate::constants::{MESSAGE_TIMEOUT, SCRIPT_DISPLAY_NAME_MAX_LENGTH};
use crate::hooks::use_container;
use serde_wasm_bindgen as swb;
use std::path::PathBuf;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::{ResourceId, ResourceMap};
use thot_ui::components::ShadowBox;
use thot_ui::types::Message;
use thot_ui::widgets::container::container_tree::{
    container::ContainerProps as ContainerUiProps, container::ContainerSettingsMenuEvent,
    Container as ContainerUi,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;

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
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeReducer` context not found");

    let show_create_asset = use_state(|| false);
    let container = use_container(props.rid.clone());
    let Some(container) = container.as_ref() else {
        panic!("`Container` not loaded");
    };

    let container_id = {
        let container = container.lock().expect("could not lock `Container`");
        container.rid.clone()
    };

    let selected = canvas_state.selected.contains(&container_id);
    let multiple_selected = canvas_state.selected.len() > 1;

    let script_names = projects_state
        .project_scripts
        .get(&canvas_state.project)
        .expect("project's state not found")
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

    let onclick = {
        let canvas_state = canvas_state.clone();
        let container_id = container_id.clone();
        let selected = selected.clone();
        let multiple_selected = multiple_selected.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let container_id = container_id.clone();
            match selection_action(selected, multiple_selected, e) {
                SelectionAction::SelectOnly => {
                    canvas_state.dispatch(CanvasStateAction::ClearSelected);
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(container_id));
                }

                SelectionAction::Select => {
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(container_id));
                }

                SelectionAction::Unselect => {
                    canvas_state.dispatch(CanvasStateAction::Unselect(container_id));
                }
            }
        })
    };

    // ----------------------------
    // --- settings menu events ---
    // ----------------------------

    let on_settings_event = {
        let show_create_asset = show_create_asset.clone();
        let container_id = container_id.clone();

        Callback::from(move |event: ContainerSettingsMenuEvent| match event {
            ContainerSettingsMenuEvent::AddAssets => show_create_asset.set(true),

            ContainerSettingsMenuEvent::OpenFolder => {
                let container_id = container_id.clone();

                spawn_local(async move {
                    let path = invoke(
                        "get_container_path",
                        ResourceIdArgs {
                            rid: container_id.clone(),
                        },
                    )
                    .await
                    .expect("could not get `Container` path");

                    let path: PathBuf = swb::from_value(path)
                        .expect("could not convert result of `get_container_path` to `PathBuf`");

                    invoke("open_file", PathBufArgs { path })
                        .await
                        .expect("could not open file");
                });
            }
            ContainerSettingsMenuEvent::DuplicateTree => {
                let container_id = container_id.clone();

                spawn_local(async move {
                    let rid = invoke(
                        "duplicate_container_tree",
                        ResourceIdArgs {
                            rid: container_id.clone(),
                        },
                    )
                    .await
                    .expect("could not invoke `duplicate_container_tree`");

                    let rid: ResourceId = swb::from_value(rid).expect(
                        "could not convert result of `duplicate_container_tree` to `ResourceId`",
                    );

                    // let tree = invoke(
                    //     "load_container_tree",
                });
            }
        })
    };

    let close_create_asset = {
        let show_create_asset = show_create_asset.clone();

        Callback::from(move |_: MouseEvent| {
            show_create_asset.set(false);
        })
    };

    // --------------
    // --- assets ---
    // --------------

    let onclick_asset = {
        let app_state = app_state.clone();
        let canvas_state = canvas_state.clone();
        let tree_state = tree_state.clone();
        let selected = selected.clone();
        let multiple_selected = multiple_selected.clone();

        Callback::from(move |(asset, e): (ResourceId, MouseEvent)| {
            let Some(asset) = get_asset(&asset, tree_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(Message::error("Could not load asset".to_string()), MESSAGE_TIMEOUT, app_state.clone()));
                return;
            };

            let rid = asset.rid.clone();
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
        let tree_state = tree_state.clone();
        let container_id = container_id.clone();

        Callback::from(move |(asset, e): (ResourceId, MouseEvent)| {
            e.stop_propagation();
            let container_id = container_id.clone();

            let Some(asset) = get_asset(&asset, tree_state.clone()) else {
                app_state.dispatch(AppStateAction::AddMessageWithTimeout(Message::error("Could not load asset".to_string()), MESSAGE_TIMEOUT, app_state.clone()));
                return;
            };

            spawn_local(async move {
                let path = invoke(
                    "get_container_path",
                    ResourceIdArgs {
                        rid: container_id.clone(),
                    },
                )
                .await
                .expect("could not get `Container` path");

                let mut path: PathBuf = swb::from_value(path)
                    .expect("could not convert result of `get_container_path` to `PathBuf`");

                path.push(asset.path.clone());
                invoke("open_file", PathBufArgs { path })
                    .await
                    .expect("could not open file");
            });
        })
    };

    let onadd_assets = {
        let show_create_asset = show_create_asset.clone();

        Callback::from(move |_: ()| {
            show_create_asset.set(false);
        })
    };

    // ---------------
    // --- scripts ---
    // ---------------

    let onclick_edit_scripts = {
        let app_state = app_state.clone();
        let canvas_state = canvas_state.clone();

        Callback::from(move |container: ResourceId| {
            let onsave = {
                let app_state = app_state.clone();
                let canvas_state = canvas_state.clone();

                Callback::from(move |_: ()| {
                    canvas_state.dispatch(CanvasStateAction::ClearDetailsBar);
                    app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                        Message::success("Resource saved".to_string()),
                        MESSAGE_TIMEOUT,
                        app_state.clone(),
                    ));
                })
            };

            canvas_state.dispatch(CanvasStateAction::SetDetailsBarWidget(
                DetailsBarWidget::ScriptsAssociationsEditor(container.clone(), Some(onsave)),
            ));
        })
    };

    // ---------------
    // --- scripts ---
    // ---------------

    let onclick_toggle_visibility = {
        let canvas_state = canvas_state.clone();
        let container_id = container_id.clone();

        Callback::from(move |_| {
            canvas_state.dispatch(CanvasStateAction::SetVisibility(
                container_id.clone(),
                !canvas_state.is_visible(&container_id),
            ));
        })
    };

    // ----------------------
    // --- on drop events ---
    // ----------------------

    let ondragenter = {
        let tree_state = tree_state.clone();
        let container_id = container_id.clone();

        Callback::from(move |_: web_sys::DragEvent| {
            tree_state.dispatch(ContainerTreeStateAction::SetDragOverContainer(
                container_id.clone(),
            ));
        })
    };

    let ondragleave = {
        let tree_state = tree_state.clone();

        Callback::from(move |_: web_sys::DragEvent| {
            tree_state.dispatch(ContainerTreeStateAction::ClearDragOverContainer);
        })
    };

    // ----------
    // --- ui ---
    // ----------

    // props
    let mut class = Classes::new();
    if selected {
        class.push("selected");
    }

    let container_val = container.lock().expect("could not lock container").clone();
    let c_props = {
        let c = container_val.clone();

        props! {
            ContainerUiProps {
                r#ref: props.r#ref.clone(),
                class,
                visible: canvas_state.is_visible(&container_id),
                rid: c.rid,
                properties: c.properties,
                assets: c.assets,
                active_assets: canvas_state.selected.clone(),
                scripts: c.scripts,
                script_names,
                preview: tree_state.preview.clone(),
                onclick,
                onclick_asset,
                ondblclick_asset,
                onadd_child: props.onadd_child.clone(),
                on_settings_event,
                ondragenter,
                ondragleave,
                onclick_edit_scripts,
                onclick_toggle_visibility,
            }
        }
    };

    let container_name = match container_val.properties.name.clone() {
        None => "(no name)".to_string(),
        Some(name) => name,
    };

    Ok(html! {
        <>
        <ContainerUi ..c_props />
        if *show_create_asset {
            // @todo: Use portal.
            <ShadowBox
                title={format!("Add Asset to {container_name}")}
                onclose={close_create_asset}>

                <CreateAssets
                    container={container_val.rid.clone()}
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
    if e.ctrl_key() {
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

fn get_asset(rid: &ResourceId, tree_state: ContainerTreeStateReducer) -> Option<CoreAsset> {
    let Some(container) = tree_state
                .asset_map
                .get(&rid) else {
                    return None;
                };

    let container = tree_state
        .containers
        .get(container)
        .expect("`Container` not found")
        .as_ref()
        .expect("`Container` not set")
        .lock()
        .expect("could not lock `Container`");

    container.assets.get(&rid).cloned()
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
