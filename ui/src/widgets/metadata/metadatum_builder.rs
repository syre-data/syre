//! Inline editor for a single metadatum.
use super::{type_from_string, type_of_value, Metadatum, MetadatumType};
use serde_json::{Result as JsResult, Value as JsValue};
use std::collections::HashSet;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MetadatumBuilderProps {
    /// Initial name.
    #[prop_or_default]
    pub name: String,

    /// Initial value.
    #[prop_or(JsValue::Null)]
    pub value: JsValue,

    /// Already existing names to validate against.
    #[prop_or_default]
    pub name_filter: HashSet<String>,

    #[prop_or_default]
    pub onsave: Callback<Metadatum>,

    #[prop_or_default]
    pub oncancel: Callback<()>,
}

#[function_component(MetadatumBuilder)]
pub fn metadatum_builder(props: &MetadatumBuilderProps) -> Html {
    // @note: `kind` and `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let key = use_state(|| props.name.clone());
    let kind = use_state(|| type_of_value(&props.value));
    let value = use_state(|| serde_json::to_string_pretty(&props.value).unwrap_or("".to_string()));
    let error = use_state(|| None);

    let key_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let value_ref = use_node_ref();

    let onchange_key = {
        let key = key.clone();
        let key_ref = key_ref.clone();

        Callback::from(move |_: Event| {
            let key_input = key_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref to input");

            key.set(key_input.value());
        })
    };

    let onchange_type = {
        let kind = kind.clone();
        let kind_ref = kind_ref.clone();

        Callback::from(move |_: Event| {
            let type_input = kind_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast node ref to select");

            kind.set(type_from_string(&type_input.value()));
        })
    };

    let onsubmit = {
        let name_filter = props.name_filter.clone();
        let onsave = props.onsave.clone();
        let value_ref = value_ref.clone();
        let key = key.clone();
        let kind = kind.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // get key
            if key.is_empty() {
                // @unreachble
                error.set(Some("Key can not be empty"));
                return;
            }

            if name_filter.contains(&*key) {
                error.set(Some("Key already exists"));
                return;
            }

            // get kind
            let Some(kind) = (*kind).clone() else {
                // @unreachble
                error.set(Some("Invalid data type"));
                return;
            };

            // get value
            let Ok(value) = value_from_input(value_ref.clone(), kind) else {
                // invalid input for type
                error.set(Some("Invalid value"));
                return;
            };

            onsave.emit(((*key).clone(), value));
        })
    };

    let oncancel = {
        let oncancel = props.oncancel.clone();

        Callback::from(move |_: MouseEvent| {
            oncancel.emit(());
        })
    };

    // create <options> for `kind` <select>
    let kind_opts = [
        MetadatumType::String,
        MetadatumType::Number,
        MetadatumType::Bool,
        MetadatumType::Array,
        MetadatumType::Object,
    ];

    let kind_opts = html! {
        { kind_opts.into_iter().map(|k| { html! {
                <option
                    value={k.clone()}
                    selected={Some(k.clone()) == *kind}>{
                    Into::<String>::into(k)
                }</option>
            }}).collect::<Html>()
        }
    };

    html! {
        <form class={classes!("thot-ui-metadatum-builder")} {onsubmit}>
            <input
                ref={key_ref}
                placeholder="Name"
                value={(*key).clone()}
                minlength="1"
                onchange={onchange_key}
                required={true} />

            <select ref={kind_ref} onchange={onchange_type}>
                { kind_opts }
            </select>

            if let Some(kind) = &*kind {
                { match kind {
                    MetadatumType::String => html! { <input ref={value_ref} value={(*value).clone()} /> },
                    MetadatumType::Number => html! { <input ref={value_ref} type={"number"} /> },
                    MetadatumType::Bool => html! { <input ref={value_ref} type={"checkbox"} /> },
                    MetadatumType::Array => html! { <textarea ref={value_ref}></textarea> },
                    MetadatumType::Object => html! { <textarea ref={value_ref}></textarea> },
                }}
            } else {
                <input ref={value_ref} />
            }

            <button>{ "Add" }</button>
            <button type="button" onclick={oncancel}>{ "Cancel" }</button>

            <div class={classes!("error")}>
                if let Some(msg) = *error {
                    { msg }
                }
            </div>
        </form>
    }
}

fn value_from_input(value_ref: NodeRef, kind: MetadatumType) -> JsResult<JsValue> {
    let value = match kind {
        MetadatumType::String => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => JsValue::String(val),
            }
        }
        MetadatumType::Number => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            let val = v_in.value_as_number();
            match val.is_nan() {
                true => JsValue::Null,
                false => JsValue::from(val),
            }
        }
        MetadatumType::Bool => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            JsValue::Bool(v_in.checked())
        }
        MetadatumType::Array => {
            let v_in = value_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast value node ref as textarea");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => serde_json::from_str(&val)?,
            }
        }
        MetadatumType::Object => {
            let v_in = value_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast value node ref as textarea");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => serde_json::from_str(&val)?,
            }
        }
    };

    Ok(value)
}

#[cfg(test)]
#[path = "./metadatum_builder_test.rs"]
mod metadatum_builder_test;
