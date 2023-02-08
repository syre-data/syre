//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
//! Acts as a wrapper around a [`thot_ui::widgets::container::container_tree::Container`].
use crate::commands::container::UpdatePropertiesArgs as UpdateContainerPropertiesArgs;
use crate::common::invoke;
use crate::components::asset::CreateAssets;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use crate::components::details_bar::DetailsBarWidget;
use crate::hooks::use_container;
use std::sync::{Arc, Mutex};
use thot_core::project::{Container as CoreContainer, Metadata};
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

    // -----------------------
    // --- onchange events ---
    // -----------------------

    let onchange_name = create_update_container_string_property_callback(
        tree_state.clone(),
        (*container).clone(),
        ContainerStringProperty::Name,
    );

    let onchange_kind = create_update_container_string_property_callback(
        tree_state.clone(),
        (*container).clone(),
        ContainerStringProperty::Kind,
    );

    let onchange_description = create_update_container_string_property_callback(
        tree_state.clone(),
        (*container).clone(),
        ContainerStringProperty::Description,
    );

    let onchange_tags = {
        let tree_state = tree_state.clone();
        let container = container.clone();

        Callback::from(move |tags| {
            let container = container.lock().expect("could not lock `Container`");
            let rid = container.rid.clone();
            let mut properties = container.properties.clone();
            Mutex::unlock(container);

            properties.tags = tags;
            let tree_state = tree_state.clone();

            spawn_local(async move {
                let update = UpdateContainerPropertiesArgs { rid, properties };
                let _res = invoke("update_container_properties", update.clone())
                    .await
                    .expect("could not invoke `update_container_properties`");

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(update));
            });
        })
    };

    let onchange_metadata = {
        let tree_state = tree_state.clone();
        let container = container.clone();

        Callback::from(move |metadata: Metadata| {
            let container = container.lock().expect("could not lock `Container`");
            let rid = container.rid.clone();
            let mut properties = container.properties.clone();
            Mutex::unlock(container);

            properties.metadata = metadata;
            let tree_state = tree_state.clone();

            spawn_local(async move {
                let update = UpdateContainerPropertiesArgs { rid, properties };
                let _res = invoke("update_container_properties", update.clone())
                    .await
                    .expect("could not invoke `update_container_properties`");

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(update));
            });
        })
    };

    // ----------------------------
    // --- settings menu events ---
    // ----------------------------

    let on_settings_event = {
        let canvas_state = canvas_state.clone();
        // let show_container_editor = show_container_editor.clone();
        let show_create_asset = show_create_asset.clone();

        Callback::from(
            move |(rid, event): (ResourceId, ContainerSettingsMenuEvent)| match event {
                // ContainerSettingsMenuEvent::Edit => show_container_editor.set(true),
                ContainerSettingsMenuEvent::Edit => canvas_state.dispatch(
                    CanvasStateAction::SetDetailsBarWidget(DetailsBarWidget::ContainerEditor(rid)),
                ),
                ContainerSettingsMenuEvent::AddAsset => show_create_asset.set(true),
                ContainerSettingsMenuEvent::Analyze => {}
            },
        )
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

                Callback::from(move |asset| {
                    tree_state.dispatch(ContainerTreeStateAction::UpdateAsset(asset));
                    canvas_state.dispatch(CanvasStateAction::CloseDetailsBar);
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
                    canvas_state.dispatch(CanvasStateAction::CloseDetailsBar);
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
    let container_val = container.lock().expect("could not lock container").clone();
    let c_props = {
        let c = container_val.clone();

        props! {
            ContainerUiProps {
                rid: c.rid,
                properties: c.properties,
                assets: c.assets,
                scripts: c.scripts,
                preview: tree_state.preview.clone(),
                onchange_name,
                onchange_kind,
                onchange_tags,
                onchange_description,
                onchange_metadata,
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
        // if *show_container_editor {
        //     <ShadowBox
        //         title={container_name.clone()}
        //         onclose={close_container_editor}>

        //         // @todo: Open in details bar.
        //         <ContainerEditor container={container_val.clone()}
        //             onchange_properties={onchange_properties.clone()} />
        //     </ShadowBox>
        // }
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

// ************************
// *** helper functions ***
// ************************

/// List of [`Container`](CoreContainer) properties that can contain a `String`.
enum ContainerStringProperty {
    Name,
    Kind,
    Description,
}

/// Creates a callback that receives a `String` and updates the corresponding
/// [`Container`](CoreContainer) property accordingly.
///
/// The recieved `String` is `trim`med, and
/// if it is empty after, is set to `None`.
fn create_update_container_string_property_callback(
    tree_state: ContainerTreeStateReducer,
    container: Arc<Mutex<CoreContainer>>,
    property: ContainerStringProperty,
) -> Callback<String> {
    Callback::from(move |value: String| {
        let value = value.trim();
        let value = if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        };

        let container = container.lock().expect("could not lock `Container`");
        let rid = container.rid.clone();
        let mut properties = container.properties.clone();
        Mutex::unlock(container);

        match property {
            ContainerStringProperty::Name => properties.name = value,
            ContainerStringProperty::Kind => properties.kind = value,
            ContainerStringProperty::Description => properties.description = value,
        }

        {
            let tree_state = tree_state.clone();

            spawn_local(async move {
                let update = UpdateContainerPropertiesArgs { rid, properties };
                let _res = invoke("update_container_properties", update.clone())
                    .await
                    .expect("could not invoke `update_container_properties`");

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(update));
            });
        }
    })
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
