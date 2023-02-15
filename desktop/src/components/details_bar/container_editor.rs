//! Container editor widget.
use crate::commands::common::{UpdatePropertiesArgs, UpdatePropertiesStringArgs};
use crate::common::invoke;
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use crate::hooks::use_container;
use std::sync::{Arc, Mutex};
use thot_core::project::{Container as CoreContainer, StandardProperties};
use thot_core::types::ResourceId;
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
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeReducer` context not found");

    let container = use_container(props.rid.clone());
    let properties = use_state(|| container_properties(container.as_ref()));

    let onchange = {
        let properties = properties.clone();

        Callback::from(move |update: StandardProperties| {
            properties.set(update);
        })
    };

    let onsave = {
        let rid = props.rid.clone();
        let onsave = props.onsave.clone();
        let tree_state = tree_state.clone();
        let properties = properties.clone();

        Callback::from(move |_: MouseEvent| {
            let rid = rid.clone();
            let onsave = onsave.clone();
            let tree_state = tree_state.clone();
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
                let _res = invoke("update_container_properties", update_str)
                    .await
                    .expect("could not invoke `update_container_properties`");

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(update));
                onsave.emit(());
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
