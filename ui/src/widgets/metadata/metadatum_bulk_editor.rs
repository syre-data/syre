//! Inline editor for a single metadatum.
use super::MetadatumBulkValueEditor;
use serde_json::Value as JsValue;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct MetadatumBulkEditorProps {
    pub name: String,

    pub value: Vec<JsValue>,

    #[prop_or_default]
    pub onchange: Callback<JsValue>,
}

#[tracing::instrument]
#[function_component(MetadatumBulkEditor)]
pub fn metadatum_bulk_editor(props: &MetadatumBulkEditorProps) -> Html {
    let error = use_state(|| None);
    let onerror = {
        let error = error.clone();

        Callback::from(move |message: String| {
            error.set(Some(message));
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
        <div class={classes!("syre-ui-metadatum")}>
            <div class={classes!("metadatum-fields")}>
                <span class={classes!("metadatum-key")}>
                    { &props.name }
                </span>

                <MetadatumBulkValueEditor
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
