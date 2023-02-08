//! Inline editor for a single metadatum.
use serde_json::{Result as JsResult, Value as JsValue};
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

// ****************
// *** Metadata ***
// ****************

pub type Metadatum = (Option<String>, JsValue);

/// Types a metadatum value can assume.
#[derive(PartialEq, Clone)]
pub enum MetadatumType {
    String,
    Bool,
    Number,
    Array,
    Object,
}

impl Into<String> for MetadatumType {
    fn into(self) -> String {
        match self {
            MetadatumType::String => "String".to_string(),
            MetadatumType::Number => "Number".to_string(),
            MetadatumType::Bool => "Boolean".to_string(),
            MetadatumType::Array => "Array".to_string(),
            MetadatumType::Object => "Object".to_string(),
        }
    }
}

impl Into<AttrValue> for MetadatumType {
    fn into(self) -> AttrValue {
        Into::<String>::into(self).into()
    }
}

impl IntoPropValue<Option<AttrValue>> for MetadatumType {
    fn into_prop_value(self) -> Option<AttrValue> {
        Some(self.into())
    }
}

/// Returns the type the string represents.
fn type_from_string(s: &str) -> Option<MetadatumType> {
    match s {
        "String" => Some(MetadatumType::String),
        "Number" => Some(MetadatumType::Number),
        "Boolean" => Some(MetadatumType::Bool),
        "Array" => Some(MetadatumType::Array),
        "Object" => Some(MetadatumType::Object),
        _ => None,
    }
}

/// Returns the type of the value.
fn type_of_value(value: &JsValue) -> Option<MetadatumType> {
    match value {
        JsValue::Null => None,
        JsValue::String(_) => Some(MetadatumType::String),
        JsValue::Number(_) => Some(MetadatumType::Number),
        JsValue::Bool(_) => Some(MetadatumType::Bool),
        JsValue::Array(_) => Some(MetadatumType::Array),
        JsValue::Object(_) => Some(MetadatumType::Object),
    }
}

// *****************
// *** Component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct MetadatumEditorProps {
    #[prop_or_default]
    pub name: Option<String>,

    #[prop_or(JsValue::Null)]
    pub value: JsValue,

    /// Initial active state of the controller.
    #[prop_or(false)]
    pub active: bool,

    /// Triggered when either the key or value change.
    /// # Fields
    /// 1. New value
    /// 2. Old value
    #[prop_or_default]
    pub onchange: Option<Callback<Metadatum>>,
}

#[function_component(MetadatumEditor)]
pub fn metadatum_editor(props: &MetadatumEditorProps) -> Html {
    // @note: `kind` and `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let active = use_state(|| props.active);
    let kind = use_state(|| type_of_value(&props.value));
    let value = use_state(|| serde_json::to_string_pretty(&props.value).unwrap_or("".to_string()));
    let error = use_state(|| None);

    let key_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let value_ref = use_node_ref();

    let activate = {
        let active = active.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            active.set(true);
        })
    };

    let on_type_change = {
        let kind = kind.clone();
        let kind_ref = kind_ref.clone();

        Callback::from(move |_: web_sys::Event| {
            let type_input = kind_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast node ref to select");

            kind.set(type_from_string(&type_input.value()));
        })
    };

    let onsave = {
        let active = active.clone();
        let key_ref = key_ref.clone();
        let kind_ref = kind_ref.clone();
        let value_ref = value_ref.clone();
        let error = error.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            active.set(false);

            // get key
            let key = key_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast key node ref into input");

            let key = key.value().trim().to_owned();
            let key = if key.is_empty() { None } else { Some(key) };

            // get kind
            let kind = kind_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast kind node ref into select");

            let Some(kind) = type_from_string(&kind.value()) else {
                // @unreachble
                error.set(Some("Invalid data type"));
                active.set(true);
                return;
            };

            // get value
            if let Ok(value) = value_from_input(value_ref.clone(), kind) {
                if let Some(onchange) = onchange.clone() {
                    onchange.emit((key, value));
                }
            } else {
                // invalid input for type
                error.set(Some("Invalid value"));
                active.set(true);
            };
        })
    };

    let oncancel = {
        let active = active.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            active.set(false);
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
        <div class={classes!("metadatum")} ondblclick={activate}>
            if *active {
                <input
                    ref={key_ref}
                    placeholder="Name"
                    value={props.name.clone().unwrap_or("".to_string())} />

                <select ref={kind_ref} onchange={on_type_change}>
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

                <button onclick={onsave}>{ "Save" }</button>
                <button onclick={oncancel}>{ "Cancel" }</button>

                if let Some(msg) = *error {
                    { msg }
                }
            } else {
                <span class={classes!("metadatum-key")}>{
                    if let Some(name) = &props.name {
                        name
                    } else {
                        { "(no key)" }
                    }
                }</span>

                if let Some(kind) = (*kind).clone() {
                    <span class={classes!("metadatum-kind")}>{ Into::<String>::into(kind) }</span>
                }

                <span class={classes!("metadatum-value")}>
                    if props.value == JsValue::Null {
                        { "(no value)" }
                    } else {
                        { value.to_string() }
                    }
                </span>
            }
        </div>
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
#[path = "./metadatum_editor_test.rs"]
mod metadatum_editor_test;
