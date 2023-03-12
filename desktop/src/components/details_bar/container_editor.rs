//! Container editor widget.
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::common::{UpdatePropertiesArgs, UpdatePropertiesStringArgs};
use crate::common::invoke;
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::constants::MESSAGE_TIMEOUT;
use serde_wasm_bindgen as swb;
use std::sync::{Arc, Mutex};
use thot_core::project::{Container as CoreContainer, StandardProperties};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::Result as LibResult;
use thot_ui::types::Message;
use thot_ui::widgets::StandardPropertiesEditor;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerEditorProps {
    pub rid: ResourceId,

    #[prop_or_default]
    pub onsave: Callback<()>,
}

#[function_component(ContainerEditor)]
pub fn container_editor(props: &ContainerEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let graph_state = use_context::<GraphStateReducer>().expect("`GraphReducer` context not found");

    let container = graph_state
        .graph
        .get(&props.rid)
        .expect("`Container` not found");

    let properties = use_state(|| container.properties.clone());
    {
        let container = container.clone();
        let properties = properties.clone();

        use_effect_with_deps(
            move |container| {
                properties.set(container.properties.clone());
            },
            container.clone(),
        );
    }

    let onchange = {
        let properties = properties.clone();

        Callback::from(move |update: StandardProperties| {
            properties.set(update);
        })
    };

    let onsave = {
        let rid = props.rid.clone();
        let onsave = props.onsave.clone();
        let graph_state = graph_state.clone();
        let properties = properties.clone();

        Callback::from(move |_: MouseEvent| {
            let rid = rid.clone();
            let onsave = onsave.clone();
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let properties = (*properties).clone();

            spawn_local(async move {
                // @todo: Issue with serializing `HashMap` of `metadata`. perform manually.
                // See: https://github.com/tauri-apps/tauri/issues/6078
                let properties_str = serde_json::to_string(&properties)
                    .expect("could not serialize `StandardProperties`");

                let update_str = UpdatePropertiesStringArgs {
                    rid: rid.clone(),
                    properties: properties_str,
                };

                let update = UpdatePropertiesArgs { rid, properties };
                web_sys::console::log_1(&2.into());
                let res = invoke("update_container_properties", update_str)
                    .await
                    .expect("could not invoke `update_container_properties`");

                web_sys::console::log_1(&0.into());
                let res: LibResult =
                    swb::from_value(res).expect("could not convert result for JsValue");

                web_sys::console::log_1(&1.into());
                match res {
                    Err(err) => {
                        web_sys::console::debug_1(&format!("{err:?}").into());
                        app_state.dispatch(AppStateAction::AddMessage(Message::error(
                            "Could not save resource",
                        )));
                    }

                    Ok(_) => {
                        graph_state.dispatch(GraphStateAction::UpdateContainerProperties(update));
                        app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                            Message::success("Resource saved"),
                            MESSAGE_TIMEOUT,
                            app_state.clone(),
                        ));
                        onsave.emit(());
                    }
                }
            });
        })
    };

    html! {
        <div>
            <StandardPropertiesEditor
                properties={(*properties).clone()}
                onchange={onchange} />

            <div>
                <button onclick={onsave}>{ "Save" }</button>
            </div>
            // @todo: <AssetDropZone />
        </div>
    }
}

// ***************
// *** helpers ***
// ***************

fn container_properties(container: Option<&Arc<Mutex<CoreContainer>>>) -> StandardProperties {
    let Some(container) = container else {
            panic!("`Container` not loaded");
        };

    let container = container.lock().expect("could not lock `Container`");
    container.properties.clone()
}

#[cfg(test)]
#[path = "./container_editor_test.rs"]
mod container_editor_test;
