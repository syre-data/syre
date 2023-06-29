//! Bulk metadata editor.
use super::{MetadataBulk, Metadatum, MetadatumBuilder, MetadatumBulkEditor};
use serde_json::Value as JsValue;
use std::collections::HashSet;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct MetadataBulkEditorProps {
    pub value: MetadataBulk,

    /// Called when a metadatum is added.
    #[prop_or_default]
    pub onadd: Option<Callback<Metadatum>>,

    /// Called when a metadatum is removed.
    #[prop_or_default]
    pub onremove: Option<Callback<String>>,

    /// Called when the value of a metadatum is changed.
    #[prop_or_default]
    pub onchange: Option<Callback<Metadatum>>,
}

#[tracing::instrument(skip(props))]
#[function_component(MetadataBulkEditor)]
pub fn metadata_bulk_editor(props: &MetadataBulkEditorProps) -> Html {
    let add_metadatum_visible = use_state(|| false);

    let show_add_metadatum = {
        let add_metadatum_visible = add_metadatum_visible.clone();

        Callback::from(move |_: MouseEvent| {
            add_metadatum_visible.set(true);
        })
    };

    let add_metadatum = {
        let onadd = props.onadd.clone();
        let add_metadatum_visible = add_metadatum_visible.clone();

        Callback::from(move |metadatum: Metadatum| {
            if let Some(onadd) = onadd.as_ref() {
                onadd.emit(metadatum.clone());
            }
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
        let onremove = props.onremove.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(onremove) = onremove.as_ref() {
                onremove.emit(key.clone());
            }
        })
    };

    let onchange = move |key: String| {
        let onchange = props.onchange.clone();
        Callback::from(move |value: JsValue| {
            if let Some(onchange) = onchange.as_ref() {
                onchange.emit((key.clone(), value.clone()));
            }
        })
    };

    let name_filter = props.value.clone().into_keys().collect::<HashSet<String>>();
    let mut value = props.value.clone().into_iter().collect::<Vec<_>>();
    value.sort_by_key(|v| v.0.clone());

    html! {
        <div class={classes!("thot-ui-metadata-editor")}>
            <div class={classes!("metadata-header")}>
                <h3>{ "Metadata" }</h3>
                <button class={classes!("add-button")} type="button" onclick={show_add_metadatum}>
                    <Icon class={classes!("thot-ui-add-remove-icon")} icon_id={ IconId::HeroiconsSolidPlus }/>
                </button>
            </div>
            <div class={classes!("add-metadatum-controls")}>
                if *add_metadatum_visible {
                    <MetadatumBuilder
                        {name_filter}
                        onsave={add_metadatum}
                        oncancel={oncancel_add_metadatum} />
                }
            </div>
            <ol class={classes!("metadata-editor")}>
                { value.into_iter().map(|(name, value)| html! {
                    <li key={name.clone()}>
                        <MetadatumBulkEditor
                            name={name.clone()}
                            {value}
                            onchange={onchange(name.clone())}/>

                        <button class={classes!("remove-button")} type="button" onclick={remove_metadatum(name)}>
                            <Icon class={classes!("thot-ui-add-remove-icon")} icon_id={IconId::HeroiconsSolidMinus}/>
                        </button>
                    </li>
                }).collect::<Html>() }
            </ol>
        </div>

    }
}
