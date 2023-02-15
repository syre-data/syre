//! Inline metadata editor.
use super::{MetadatumBuilder, MetadatumEditor};
use std::collections::HashSet;
use thot_core::project::Metadata;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MetadataEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Displayed if inactive and no value.
    #[prop_or_default]
    pub children: Children,

    #[prop_or(Metadata::new())]
    pub value: Metadata,

    /// Callback triggered when the value of a single `Metadatum` is changed
    /// or a new `Metadatum` is added.
    ///
    /// # Fields
    /// 1. Current value
    #[prop_or_default]
    pub onchange: Callback<Metadata>,
}

#[function_component(MetadataEditor)]
pub fn metadata_editor(props: &MetadataEditorProps) -> Html {
    let add_metadatum_visible = use_state(|| false);

    let show_add_metadatum = {
        let add_metadatum_visible = add_metadatum_visible.clone();

        Callback::from(move |_: MouseEvent| {
            add_metadatum_visible.set(true);
        })
    };

    let add_metadatum = {
        let metadata = props.value.clone();
        let onchange = props.onchange.clone();
        let add_metadatum_visible = add_metadatum_visible.clone();

        Callback::from(move |(key, value)| {
            let mut metadata = metadata.clone();
            metadata.insert(key, value);
            onchange.emit(metadata);
            add_metadatum_visible.set(false);
        })
    };

    let oncancel_add_metadatum = {
        let add_metadatum_visible = add_metadatum_visible.clone();

        Callback::from(move |_| {
            add_metadatum_visible.set(false);
        })
    };

    let remove_metadatum = move |key: String| {
        let metadata = props.value.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |_: MouseEvent| {
            let mut metadata = metadata.clone();

            metadata.remove(&key);
            onchange.emit(metadata);
        })
    };

    let onchange = {
        let onchange = props.onchange.clone();
        let metadata = props.value.clone();

        move |key: String| {
            let onchange = onchange.clone();
            let metadata = metadata.clone();

            Callback::from(move |value| {
                let mut metadata = metadata.clone();
                metadata.insert(key.clone(), value);
                onchange.emit(metadata);
            })
        }
    };

    let name_filter = props.value.clone().into_keys().collect::<HashSet<String>>();
    let class = classes!("thot-ui-metadata-editor", props.class.clone());

    html! {
        <div {class}>
            <div class={classes!("add-metadatum-controls")}>
                if *add_metadatum_visible {
                    <MetadatumBuilder
                        {name_filter}
                        onsave={add_metadatum}
                        oncancel={oncancel_add_metadatum} />
                } else {
                    <button onclick={show_add_metadatum}>{ "+" }</button>
                }
            </div>
            <ol class={classes!("metadata-editor")}>
                { props.value.clone().into_iter().map(|(name, value)| html! {
                    <li key={name.clone()}>
                        <MetadatumEditor
                            name={name.clone()}
                            {value}
                            onchange={onchange(name.clone())}/>

                        <button onclick={remove_metadatum(name)}>{ "X" }</button>
                    </li>
                }).collect::<Html>() }
            </ol>
        </div>
    }
}

#[cfg(test)]
#[path = "./metadata_editor_test.rs"]
mod metadata_editor_test;
