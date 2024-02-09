//! Inline metadata editor.
use super::{MetadatumBuilder, MetadatumEditor};
use std::collections::HashSet;
use syre_core::project::Metadata;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct MetadataEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Displayed if inactive and no value.
    #[prop_or_default]
    pub children: Children,

    #[prop_or(Metadata::new())]
    pub value: Metadata,

    #[prop_or_default]
    pub onchange: Callback<Metadata>,
}

#[function_component(MetadataEditor)]
pub fn metadata_editor(props: &MetadataEditorProps) -> Html {
    let add_metadatum_visible = use_state(|| false);

    let show_add_metadatum = use_callback((), {
        let add_metadatum_visible = add_metadatum_visible.setter();

        move |_: MouseEvent, _| {
            add_metadatum_visible.set(true);
        }
    });

    let add_metadatum = use_callback((props.value.clone(), props.onchange.clone()), {
        let add_metadatum_visible = add_metadatum_visible.setter();

        move |(key, value), (metadata, onchange)| {
            let mut metadata = metadata.clone();
            metadata.insert(key, value);
            onchange.emit(metadata);
            add_metadatum_visible.set(false);
        }
    });

    let oncancel_add_metadatum = use_callback((), {
        let add_metadatum_visible = add_metadatum_visible.setter();
        move |_, _| {
            add_metadatum_visible.set(false);
        }
    });

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
    let class = classes!("syre-ui-metadata-editor", props.class.clone());

    html! {
        <div {class}>
            <div class={"metadata-header"}>
                <h3>{ "Metadata" }</h3>
                <button class={"add-button"} type={"button"} onclick={show_add_metadatum}>
                    <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                        icon_id={IconId::HeroiconsSolidPlus}/>
                </button>
            </div>
            <div class={"add-metadatum-controls"}>
                if *add_metadatum_visible {
                    <MetadatumBuilder
                        {name_filter}
                        onsave={add_metadatum}
                        oncancel={oncancel_add_metadatum} />
                }
            </div>
            <ol class={"metadata-editor"}>
                { props.value.clone().into_iter().map(|(name, value)| html! {
                    <li key={name.clone()}>
                        <MetadatumEditor
                            name={name.clone()}
                            {value}
                            onchange={onchange(name.clone())}/>

                        <button class={"remove-button"} type={"button"} onclick={remove_metadatum(name)}>
                            <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                                icon_id={IconId::HeroiconsSolidMinus}/>
                        </button>
                    </li>
                }).collect::<Html>() }
            </ol>
        </div>
    }
}
