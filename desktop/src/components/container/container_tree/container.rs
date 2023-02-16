//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
//! Acts as a wrapper around a [`thot_ui::widgets::container::container_tree::Container`].
use crate::commands::common::UpdatePropertiesArgs;
use crate::common::invoke;
use crate::components::asset::CreateAssets;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use crate::components::details_bar::DetailsBarWidget;
use crate::hooks::use_container;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use thot_ui::components::ShadowBox;
use thot_ui::widgets::container::container_tree::{
    container::ContainerProps as ContainerUiProps, container::ContainerSettingsMenuEvent,
    Container as ContainerUi,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::props;

// *****************
// *** Container ***
// *****************

// @remove
// ContainerSettingsMenuEvent::Edit => canvas_state.dispatch(
//     CanvasStateAction::SetDetailsBarWidget(DetailsBarWidget::ContainerEditor(rid)),
// ),

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub rid: ResourceId,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,
}

#[function_component(Container)]
pub fn container(props: &ContainerProps) -> HtmlResult {
    // -------------
    // --- setup ---
    // -------------

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

    // -------------------
    // --- interaction ---
    // -------------------

    let onclick = {
        let canvas_state = canvas_state.clone();
        let container_id = container_id.clone();
        let selected = selected.clone();

        Callback::from(move |_| {
            let container_id = container_id.clone();
            let action = if selected {
                CanvasStateAction::Unselect(container_id)
            } else {
                CanvasStateAction::Select(container_id)
            };

            canvas_state.dispatch(action);
        })
    };

    // ----------------------------
    // --- settings menu events ---
    // ----------------------------

    let on_settings_event = {
        let show_create_asset = show_create_asset.clone();

        Callback::from(move |event: ContainerSettingsMenuEvent| match event {
            ContainerSettingsMenuEvent::AddAsset => show_create_asset.set(true),
            ContainerSettingsMenuEvent::Analyze => {}
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

    let ondblclick_asset = {
        let canvas_state = canvas_state.clone();
        let tree_state = tree_state.clone();

        Callback::from(move |asset: ResourceId| {
            let container = tree_state
                .asset_map
                .get(&asset)
                .expect("`Asset`'s `Container` not found");

            let container = tree_state
                .containers
                .get(container)
                .expect("`Container` not found")
                .as_ref()
                .expect("`Container` not set")
                .lock()
                .expect("could not lock `Container`");

            let asset = container.assets.get(&asset).expect("`Asset` not found");
            let onsave = {
                let canvas_state = canvas_state.clone();
                let tree_state = tree_state.clone();

                Callback::from(move |asset: CoreAsset| {
                    let tree_state = tree_state.clone();
                    let canvas_state = canvas_state.clone();

                    spawn_local(async move {
                        let res = invoke(
                            "update_asset_properties",
                            &UpdatePropertiesArgs {
                                rid: asset.rid.clone(),
                                properties: asset.properties.clone(),
                            },
                        )
                        .await
                        .expect("could not invoke `update_asset_properties`");

                        tree_state.dispatch(ContainerTreeStateAction::UpdateAsset(asset));
                        canvas_state.dispatch(CanvasStateAction::ClearDetailsBar);
                    });
                })
            };

            canvas_state.dispatch(CanvasStateAction::SetDetailsBarWidget(
                DetailsBarWidget::AssetEditor(asset.clone(), onsave),
            ));
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
        let canvas_state = canvas_state.clone();

        Callback::from(move |container: ResourceId| {
            let onsave = {
                let canvas_state = canvas_state.clone();

                Callback::from(move |_: ()| {
                    canvas_state.dispatch(CanvasStateAction::ClearDetailsBar);
                })
            };

            canvas_state.dispatch(CanvasStateAction::SetDetailsBarWidget(
                DetailsBarWidget::ScriptsAssociationsEditor(container.clone(), Some(onsave)),
            ));
        })
    };

    // ----------------------
    // --- on drop events ---
    // ----------------------

    let ondrop = {
        Callback::from(move |e: web_sys::DragEvent| {
            e.prevent_default();
            web_sys::console::log_1(&format!("{e:#?}").into()); // @remove
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
                class,
                rid: c.rid,
                properties: c.properties,
                assets: c.assets,
                scripts: c.scripts,
                preview: tree_state.preview.clone(),
                onclick,
                ondblclick_asset,
                onadd_child: props.onadd_child.clone(),
                on_settings_event,
                ondrop,
                onclick_edit_scripts,
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

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
