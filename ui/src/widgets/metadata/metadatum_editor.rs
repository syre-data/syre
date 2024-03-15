//! Inline editor for a single metadatum.
use super::MetadatumValueEditor;
use serde_json::Value as JsValue;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct MetadatumEditorProps {
    pub name: String,

    #[prop_or(JsValue::Null)]
    pub value: JsValue,

    #[prop_or_default]
    pub onchange: Callback<JsValue>,
}

#[function_component(MetadatumEditor)]
pub fn metadatum_editor(props: &MetadatumEditorProps) -> Html {
    let error = use_state(|| None);
    let onerror = use_callback((), {
        let error = error.setter();
        move |message: String, _| {
            error.set(message.into());
        }
    });

    let onchange = use_callback(props.onchange.clone(), {
        let error = error.setter();
        move |value, onchange| {
            error.set(None);
            onchange.emit(value);
        }
    });

    html! {
        <div class={"syre-ui-metadatum"}>
            <div class={"metadatum-fields"}>
                <span class={"metadatum-key"}
                    title={props.name.clone()}>
                    { &props.name }
                </span>

                <MetadatumValueEditor
                    value={props.value.clone()}
                    {onchange}
                    {onerror} />
            </div>

            if let Some(msg) = error.as_ref() {
                <span class={"error"}>{ msg }</span>
            }
        </div>
    }
}
