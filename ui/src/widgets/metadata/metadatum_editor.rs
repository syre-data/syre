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

#[tracing::instrument]
#[function_component(MetadatumEditor)]
pub fn metadatum_editor(props: &MetadatumEditorProps) -> Html {
    let error = use_state(|| None);
    let onerror = {
        let error = error.clone();
        Callback::from(move |message: String| {
            error.set(message.into());
        })
    };

    let onchange = {
        let onchange = props.onchange.clone();
        let error = error.clone();

        Callback::from(move |value| {
            error.set(None);
            onchange.emit(value);
        })
    };

    // ui
    html! {
        <div class={classes!("thot-ui-metadatum")}>
            <div class={classes!("metadatum-fields")}>
                <span class={classes!("metadatum-key")}
                    title={props.name.clone()}>
                    { &props.name }
                </span>

                <MetadatumValueEditor
                    class={classes!("metadatum-value")}
                    value={props.value.clone()}
                    {onchange}
                    {onerror} />
            </div>

            if let Some(msg) = error.as_ref() {
                <span class={classes!("error")}>{ msg }</span>
            }
        </div>
    }
}
