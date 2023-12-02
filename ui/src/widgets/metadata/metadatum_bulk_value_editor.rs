//! Editor for a `Metadatum` value.
use super::common;
use super::{type_from_string, type_of_value, MetadatumType};
use serde_json::Value as JsValue;
use std::rc::Rc;
use yew::prelude::*;

const PLACEHOLDER: &'static str = "(mixed)";

#[derive(PartialEq, Clone)]
enum BulkValue {
    MixedType,
    MixedValue(MetadatumType),
    EqualValue(JsValue),
}

// ***************
// *** reducer ***
// ***************

enum MetadatumStateAction {
    New(Vec<JsValue>),
    Set(BulkValue),
}

#[derive(PartialEq, Clone)]
struct MetadatumState {
    value: BulkValue,
    dirty: bool,
}

impl MetadatumState {
    pub fn new(values: &Vec<JsValue>) -> Self {
        let mut vals = values.clone();
        vals.dedup();
        if vals.len() == 1 {
            return Self {
                value: BulkValue::EqualValue(vals[0].clone()),
                dirty: false,
            };
        }

        // check if value types are equal
        let mut kinds = values.iter().map(|v| type_of_value(v)).collect::<Vec<_>>();
        kinds.dedup();
        if kinds.len() == 1 {
            let kind = kinds[0].clone().expect("invalid metadatum type");
            return Self {
                value: BulkValue::MixedValue(kind),
                dirty: false,
            };
        }

        Self {
            value: BulkValue::MixedType,
            dirty: false,
        }
    }

    pub fn value(&self) -> &BulkValue {
        &self.value
    }

    pub fn kind(&self) -> Option<MetadatumType> {
        match &self.value {
            BulkValue::MixedType => None,
            BulkValue::MixedValue(kind) => Some(kind.clone()),
            BulkValue::EqualValue(value) => type_of_value(&value),
        }
    }
}

impl Reducible for MetadatumState {
    type Action = MetadatumStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MetadatumStateAction::New(value) => Self::new(&value).into(),

            MetadatumStateAction::Set(value) => {
                let current = Self { value, dirty: true };
                current.into()
            }
        }
    }
}

// *****************
// *** component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct MetadatumBulkValueEditorProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub value: Vec<JsValue>,

    #[prop_or_default]
    pub onchange: Callback<JsValue>,

    #[prop_or_default]
    pub onerror: Callback<String>,
}

#[function_component(MetadatumBulkValueEditor)]
pub fn metadatum_bulk_value_editor(props: &MetadatumBulkValueEditorProps) -> Html {
    assert!(props.value.len() > 0, "at least one value must be assigned");

    // NOTE `value` are set to default values if they can not be
    // interpreted correctly. It may be better to return an error instead,
    // although this situation should likely never arise due to their types.
    let state = use_reducer(|| MetadatumState::new(&props.value));
    let kind_ref = use_node_ref();
    let value_ref = use_node_ref();

    {
        // update states if prop value changes
        let state = state.clone();
        use_effect_with(props.value.clone(), move |value| {
            state.dispatch(MetadatumStateAction::New(value.clone()));
        });
    }

    {
        let state = state.clone();
        let value_ref = value_ref.clone();

        use_effect_with((state, value_ref), move |(state, value_ref)| {
            if state.kind() != Some(MetadatumType::Bool) {
                return;
            }
        
            match state.value() {
                &BulkValue::EqualValue(_) => {}
                _ => {
                    let input = value_ref
                        .cast::<web_sys::HtmlInputElement>()
                        .expect("could not cast node ref to input element");
        
                    input.set_indeterminate(true);
                }
            };
        });
    }

    {
        // emit changes
        let onchange = props.onchange.clone();
        let state = state.clone();
        use_effect_with(state, move |state| {
            if let BulkValue::EqualValue(value) = state.value() {
                onchange.emit(value.clone());
            }
        });
    }

    let onchange_kind = {
        let state = state.clone();
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

            state.dispatch(MetadatumStateAction::Set(BulkValue::EqualValue(
                common::convert_value(JsValue::Null, &kind_val),
            )));
        })
    };

    let onchange_value = {
        let state = state.clone();
        let value_ref = value_ref.clone();
        let onerror = props.onerror.clone();

        Callback::from(move |_: Event| {
            if let Ok(val) = common::value_from_input(
                value_ref.clone(),
                &state.kind().expect("invalid metadatum type"),
            ) {
                state.dispatch(MetadatumStateAction::Set(BulkValue::EqualValue(val)));
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

    let kind = match state.value().clone() {
        BulkValue::MixedType => None,
        BulkValue::MixedValue(kind) => Some(kind),
        BulkValue::EqualValue(value) => type_of_value(&value),
    };

    let kind_opts = html! {
        <>
        if kind.is_none() {
            <option selected={true} disabled={true}>{ "(mixed)" }</option>
        }

        { kind_opts.into_iter().map(|k| { html! {
                <option
                    value={k.clone()}
                    selected={Some(k.clone()) == kind}>

                    { Into::<String>::into(k) }
                </option>
            }}).collect::<Html>()
        }
        </>
    };

    // ui
    let class = classes!("thot-ui-metadatum-value-editor", props.class.clone());

    html! {
        <span {class}>
            <select ref={kind_ref} onchange={onchange_kind.clone()}>
                { kind_opts }
            </select>

            if let BulkValue::MixedValue(kind) = state.value() {
                { match kind {
                    MetadatumType::String => html! {
                        <input
                            ref={value_ref.clone()}
                            value={""}
                            {PLACEHOLDER}
                            onchange={onchange_value.clone()} />
                    },

                    MetadatumType::Number => html! {
                        <input
                            ref={value_ref.clone()}
                            type={"number"}
                            {PLACEHOLDER}
                            value={""}
                            onchange={onchange_value.clone()} />
                    },

                    MetadatumType::Bool => html! {
                        <input
                            ref={value_ref.clone()}
                            type={"checkbox"}
                            checked={true}
                            onchange={onchange_value.clone()} />
                    },

                    MetadatumType::Array => html! {
                        <textarea
                            ref={value_ref.clone()}
                            {PLACEHOLDER}
                            value={""}
                            onchange={onchange_value.clone()}>
                        </textarea>
                    },

                    MetadatumType::Object => html! {
                        <textarea
                            ref={value_ref.clone()}
                            {PLACEHOLDER}
                            value={""}
                            onchange={onchange_value.clone()}>
                        </textarea>
                    },
                }}
            }

            if let BulkValue::EqualValue(value) = state.value() {
                { match value {
                    JsValue::String(value) => html! {
                        <input
                            ref={value_ref}
                            value={value.clone()}
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
                            checked={value.clone()}
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
            }
        </span>
    }
}
