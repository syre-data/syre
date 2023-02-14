//! Container editor widget.
use crate::commands::common::UpdatePropertiesArgs;
use crate::common::invoke;
use crate::components::canvas::{ContainerTreeStateAction, ContainerTreeStateReducer};
use crate::hooks::use_container;
use std::sync::{Arc, Mutex};
use thot_core::project::{Container as CoreContainer, Metadata, StandardProperties};
use thot_core::types::ResourceId;
use thot_ui::widgets::{MetadataEditor, StandardPropertiesEditor};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerEditorProps {
    pub rid: ResourceId,

    #[prop_or_default]
    pub onsave: Option<Callback<()>>,
}

#[function_component(ContainerEditor)]
pub fn container_editor(props: &ContainerEditorProps) -> Html {
    let tree_state = use_context::<ContainerTreeStateReducer>()
        .expect("`ContainerTreeReducer` context not found");

    let container = use_container(props.rid.clone());
    let properties = use_state(|| container_properties(container.as_ref()));

    //     {
    //         let properties = properties.clone();
    //         let metadata = metadata.clone();

    //         let container = container.clone();
    //         let Some(container) = container.as_ref() else {
    //             panic!("`Container` not loaded");
    //         };

    //         let container_lock = container.lock().expect("could not lock `Container`");
    //         let container = container_lock.clone();
    //         Mutex::unlock(container_lock);

    //         use_effect_with_deps(
    //             move |container| {
    //                 properties.set(container_properties(container.as_ref()));
    //                 metadata.set(container_metadata(container.as_ref()));
    //             },
    //             container,
    //         );
    //     }

    let onchange_properties = {
        let properties = properties.clone();

        Callback::from(move |mut update: StandardProperties| {
            update.metadata = properties.metadata.clone();
            properties.set(update);
        })
    };

    let onchange_metadata = {
        let properties = properties.clone();

        Callback::from(move |metadata: Metadata| {
            let mut props = (*properties).clone();
            props.metadata = metadata;
            properties.set(props);
        })
    };

    let onsave = {
        let rid = props.rid.clone();
        let onsave = props.onsave.clone();
        let tree_state = tree_state.clone();
        let properties = properties.clone();

        Callback::from(move |_: MouseEvent| {
            let onsave = onsave.clone();
            let tree_state = tree_state.clone();

            let update = UpdatePropertiesArgs {
                rid: rid.clone(),
                properties: (*properties).clone(),
            };

            spawn_local(async move {
                let _res = invoke("update_container_properties", update.clone())
                    .await
                    .expect("could not invoke `update_container_properties`");

                tree_state.dispatch(ContainerTreeStateAction::UpdateContainerProperties(update));
                if let Some(onsave) = onsave {
                    onsave.emit(());
                }
            });
        })
    };

    html! {
        <div>
            <StandardPropertiesEditor
                properties={(*properties).clone()}
                onchange={onchange_properties} />

            <MetadataEditor
                value={properties.metadata.clone()}
                onchange={onchange_metadata} />

            <button onclick={onsave}>{ "Save" }</button>
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
