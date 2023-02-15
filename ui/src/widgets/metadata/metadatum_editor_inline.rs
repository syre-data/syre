//! Inline editor for a single metadatum.
use super::MetadatumEditor;
use serde_json::Value as JsValue;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InlineMetadatumEditorProps {
    pub name: String,

    #[prop_or(JsValue::Null)]
    pub value: JsValue,

    #[prop_or_default]
    pub onchange: Option<Callback<JsValue>>,

    /// Initial active state of the controller.
    /// Only relevant if `onchange` is provided.
    #[prop_or(false)]
    pub active: bool,
}

#[function_component(InlineMetadatumEditor)]
pub fn inline_metadatum_editor(props: &InlineMetadatumEditorProps) -> Html {
    // @note: `kind` and `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let active = use_state(|| props.active);
    let value = use_state(|| props.value.clone());

    let activate = {
        let onchange = props.onchange.clone();
        let active = active.clone();

        Callback::from(move |_: MouseEvent| {
            if onchange.is_some() {
                active.set(true);
            }
        })
    };

    let onchange = {
        let value = value.clone();

        Callback::from(move |val| {
            value.set(val);
        })
    };

    let onsave = {
        let onchange = props.onchange.clone();
        let value = value.clone();
        let active = active.clone();

        Callback::from(move |_: MouseEvent| {
            active.set(false);
            if let Some(onchange) = onchange.as_ref() {
                onchange.emit((*value).clone());
            }
        })
    };

    let oncancel = {
        let active = active.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            active.set(false);
        })
    };

    html! {
        <div class={classes!("metadatum")} ondblclick={activate}>
            if *active {
                <MetadatumEditor
                    name={props.name.clone()}
                    value={(*value).clone()}
                    {onchange} />

                <button onclick={onsave}>{ "Save" }</button>
                <button onclick={oncancel}>{ "Cancel" }</button>
            } else {
                <span class={classes!("metadatum-key")}>
                        { &props.name }
                </span>

                <span class={classes!("metadatum-value")}>
                    if value.is_null() {
                        { "(no value)" }
                    } else {
                        { value.to_string() }
                    }
                </span>
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./metadatum_editor_inline_test.rs"]
mod metadatum_editor_inline_test;
