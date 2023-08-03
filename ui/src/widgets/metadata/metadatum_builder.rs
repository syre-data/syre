//! Inline editor for a single metadatum.
use super::{Metadatum, MetadatumValueEditor};
use serde_json::Value as JsValue;
use std::collections::HashSet;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct MetadatumBuilderProps {
    /// Initial name.
    #[prop_or_default]
    pub name: String,

    /// Initial value.
    #[prop_or(JsValue::String(String::default()))]
    pub value: JsValue,

    /// Already existing names to validate against.
    #[prop_or_default]
    pub name_filter: HashSet<String>,

    #[prop_or_default]
    pub onsave: Callback<Metadatum>,

    #[prop_or_default]
    pub oncancel: Callback<()>,
}

#[tracing::instrument]
#[function_component(MetadatumBuilder)]
pub fn metadatum_builder(props: &MetadatumBuilderProps) -> Html {
    // @note: `kind` and `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let value = use_state(|| props.value.clone());
    let error = use_state(|| None);
    let key = use_state(|| props.name.clone());
    let key_ref = use_node_ref();

    let onchange_key = {
        let key = key.clone();
        let key_ref = key_ref.clone();

        Callback::from(move |_: Event| {
            let key_input = key_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref to input");

            key.set(key_input.value().trim().into());
        })
    };

    let onchange_value = {
        let value = value.clone();
        Callback::from(move |val| {
            value.set(val);
        })
    };

    let onerror = {
        let error = error.clone();
        Callback::from(move |message: String| {
            error.set(Some(message));
        })
    };

    let onsubmit = {
        let name_filter = props.name_filter.clone();
        let onsave = props.onsave.clone();
        let key = key.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // get key
            if key.is_empty() {
                // @unreachble
                error.set(Some("Key can not be empty".to_string()));
                return;
            }

            if name_filter.contains(&*key) {
                error.set(Some("Key already exists".to_string()));
                return;
            }

            if (*error).is_some() {
                return;
            }

            onsave.emit(((*key).clone(), (*value).clone()));
        })
    };

    let oncancel = {
        let oncancel = props.oncancel.clone();
        Callback::from(move |_: MouseEvent| {
            oncancel.emit(());
        })
    };

    html! {
        <div class={classes!("thot-ui-metadatum-builder")}>
            <form {onsubmit}>
                <div class={classes!("form-fields")}>
                    <input
                        ref={key_ref}
                        placeholder="Name"
                        value={(*key).clone()}
                        minlength="1"
                        onchange={onchange_key}
                        required={true} />

                    <MetadatumValueEditor
                        class={classes!("metadatum-value")}
                        value={props.value.clone()}
                        onchange={onchange_value}
                        {onerror} />

                    <div class={classes!("form-controls")}>
                        <button>{ "Add" }</button>
                        <button type="button" onclick={oncancel}>{ "Cancel" }</button>
                    </div>
                </div>
            </form>

            <div class={classes!("error")}>
                if let Some(msg) = error.as_ref() {
                    { msg }
                }
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./metadatum_builder_test.rs"]
mod metadatum_builder_test;
