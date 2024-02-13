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
    // NOTE `kind` and `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let value = use_state(|| props.value.clone());
    let error = use_state(|| None);
    let key = use_state(|| props.name.clone());
    let key_ref = use_node_ref();

    let onchange_key = use_callback((), {
        let key = key.setter();
        let key_ref = key_ref.clone();

        move |_: Event, _| {
            let key_input = key_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref to input");

            key.set(key_input.value().trim().into());
        }
    });

    let onchange_value = use_callback((), {
        let value = value.setter();
        move |val, _| {
            value.set(val);
        }
    });

    let onerror = use_callback((), {
        let error = error.setter();
        move |message: String, _| {
            error.set(message.into());
        }
    });

    let onsubmit = use_callback(
        (
            key.clone(),
            value.clone(),
            error.clone(),
            props.name_filter.clone(),
            props.onsave.clone(),
        ),
        move |e: SubmitEvent, (key, value, error, name_filter, onsave)| {
            e.prevent_default();

            // get key
            if key.is_empty() {
                // @unreachble
                error.set(Some("Key can not be empty".to_string()));
                return;
            }

            if name_filter.contains(&**key) {
                error.set(Some("Key already exists".to_string()));
                return;
            }

            if (*error).is_some() {
                return;
            }

            onsave.emit(((**key).clone(), (**value).clone()));
        },
    );

    let oncancel = use_callback(props.oncancel.clone(), move |_: MouseEvent, oncancel| {
        oncancel.emit(());
    });

    html! {
        <div class={"syre-ui-metadatum-builder"}>
            <form {onsubmit}>
                <div class={"form-fields"}>
                    <div class={"metadatum-fields"}>
                        <span class={"metadatum-name"}>
                            <input
                                ref={key_ref}
                                placeholder={"Name"}
                                value={(*key).clone()}
                                minlength={"1"}
                                onchange={onchange_key}
                                required={true} />
                        </span>

                        <MetadatumValueEditor
                            value={(*value).clone()}
                            onchange={onchange_value}
                            {onerror} />
                    </div>
                    <div class={"form-controls"}>
                        <button>{ "Add" }</button>
                        <button type={"button"} onclick={oncancel}>{ "Cancel" }</button>
                    </div>
                </div>
            </form>

            <div class={"error"}>
                if let Some(msg) = error.as_ref() {
                    { msg }
                }
            </div>
        </div>
    }
}
