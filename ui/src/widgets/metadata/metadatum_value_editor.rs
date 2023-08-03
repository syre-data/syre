//! Editor for a `Metadatum` value.
use super::common;
use super::{type_from_string, type_of_value, MetadatumType};
use serde_json::Value as JsValue;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct MetadatumValueEditorProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or(JsValue::Null)]
    pub value: JsValue,

    #[prop_or_default]
    pub onchange: Callback<JsValue>,

    #[prop_or_default]
    pub onerror: Callback<String>,
}

#[tracing::instrument]
#[function_component(MetadatumValueEditor)]
pub fn metadatum_value_editor(props: &MetadatumValueEditorProps) -> Html {
    // NOTE `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let value = use_state(|| props.value.clone());
    let kind_ref = use_node_ref();
    let value_ref = use_node_ref();

    {
        // update states if prop value changes
        let value = value.clone();

        use_effect_with_deps(
            move |val| {
                value.set(val.clone());
            },
            props.value.clone(),
        );
    }

    {
        // call onchange whenever the value has changed
        let onchange = props.onchange.clone();
        let value = value.clone();

        use_effect_with_deps(
            move |value| {
                onchange.emit((**value).clone());
            },
            value,
        );
    }

    let onchange_kind = {
        let value = value.clone();
        let kind_ref = kind_ref.clone();
        let onerror = props.onerror.clone();

        Callback::from(move |_: Event| {
            // get kind
            let kind_val = kind_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast kind node ref into select");

            let Some(kind_val) = type_from_string(&kind_val.value()) else {
                // @unreachble
                onerror.emit("Invalid data type".to_string());
                return;
            };

            value.set(common::convert_value((*value).clone(), &kind_val));
        })
    };

    let onchange_value = {
        let value = value.clone();
        let value_ref = value_ref.clone();
        let onerror = props.onerror.clone();

        Callback::from(move |_: Event| {
            let Some(kind) = type_of_value(&*value) else {
                onerror.emit("Invalid data type".to_string());
                return;
            };

            // get value
            if let Ok(val) = common::value_from_input(value_ref.clone(), &kind) {
                if kind == MetadatumType::Number && val == JsValue::Null {
                    // invalid number input
                    onerror.emit("Invalid number".to_string());
                    return;
                }

                value.set(common::convert_value(val, &kind));
            } else {
                // invalid input for type
                onerror.emit("Invalid value".to_string());
            };
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

    let kind = type_of_value(&*value).unwrap_or_default();
    let kind_opts = html! {
        { kind_opts.into_iter().map(|k| { html! {
                <option
                    value={k.clone()}
                    selected={k.clone() == kind}>

                    { Into::<String>::into(k) }
                </option>
            }}).collect::<Html>()
        }
    };

    // ui
    let class = classes!("thot-ui-metadatum-value-editor", props.class.clone());

    html! {
        <span {class}>
            <select ref={kind_ref} onchange={onchange_kind.clone()}>
                { kind_opts }
            </select>

            { match (*value).clone() {
                JsValue::String(value) => html! {
                    <input
                        ref={value_ref}
                        {value}
                        placeholder={"Value"}
                        onchange={onchange_value.clone()} />
                },

                JsValue::Number(value) => html! {
                    <input
                        ref={value_ref}
                        type={"number"}
                        value={value.to_string()}
                        onchange={onchange_value.clone()} />
                },

                JsValue::Bool(value) => html! {
                    <input
                        ref={value_ref}
                        type={"checkbox"}
                        checked={value}
                        onchange={onchange_value.clone()} />
                },

                JsValue::Array(value) => html! {
                    <textarea
                        ref={value_ref}
                        value={serde_json::to_string_pretty(&value).unwrap_or(String::default())}
                        onchange={onchange_value.clone()}>
                    </textarea>
                },

                JsValue::Object(value) => html! {
                    <textarea
                        ref={value_ref}
                        value={serde_json::to_string_pretty(&value).unwrap_or(String::default())}
                        onchange={onchange_value.clone()}>
                    </textarea>
                },

                JsValue::Null => html! {}
            }}
        </span>
    }
}

#[cfg(test)]
#[path = "./metadatum_value_editor_test.rs"]
mod metadatum_value_editor_test;
