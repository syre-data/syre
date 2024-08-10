pub mod kind {
    use crate::components::form::debounced::InputText;
    use leptos::*;

    #[component]
    pub fn Editor(
        #[prop(into)] value: MaybeSignal<Option<String>>,
        #[prop(into)] oninput: Callback<Option<String>>,
        #[prop(into)] debounce: MaybeSignal<f64>,
    ) -> impl IntoView {
        let (processed_value, set_processed_value) = create_signal(value());

        let input_value = move || value.with(|value| value.clone().unwrap_or(String::new()));

        let oninput_text = {
            move |value: String| {
                let value = value.trim();
                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };

                set_processed_value(value);
            }
        };

        create_effect(move |_| {
            oninput(processed_value());
        });

        view! { <InputText value=Signal::derive(input_value) oninput=oninput_text debounce/> }
    }
}

pub mod description {
    use crate::components::form::debounced::TextArea;
    use leptos::*;

    #[component]
    pub fn Editor(
        #[prop(into)] value: MaybeSignal<Option<String>>,
        #[prop(into)] oninput: Callback<Option<String>>,
        #[prop(into)] debounce: MaybeSignal<f64>,
    ) -> impl IntoView {
        let (processed_value, set_processed_value) = create_signal(value());

        let input_value = move || value.with(|value| value.clone().unwrap_or(String::new()));

        let oninput_text = {
            move |value: String| {
                let value = value.trim();
                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };

                set_processed_value(value);
            }
        };

        create_effect(move |_| {
            oninput(processed_value());
        });

        view! { <TextArea value=Signal::derive(input_value) oninput=oninput_text debounce/> }
    }
}

pub mod tags {
    use crate::components::form::debounced::InputText;
    use leptos::*;

    #[component]
    pub fn Editor(
        #[prop(into)] value: MaybeSignal<Vec<String>>,
        #[prop(into)] oninput: Callback<Vec<String>>,
        #[prop(into)] debounce: MaybeSignal<f64>,
    ) -> impl IntoView {
        let (processed_value, set_processed_value) = create_signal(value());

        let input_value = move || value.with(|value| value.join(", "));

        let oninput_text = {
            move |value: String| {
                let tags = value
                    .split(",")
                    .filter_map(|tag| {
                        let tag = tag.trim();
                        if tag.is_empty() {
                            None
                        } else {
                            Some(tag.to_string())
                        }
                    })
                    .collect::<Vec<_>>();

                let mut tags_unique = Vec::with_capacity(tags.len());
                for tag in tags {
                    if !tags_unique.contains(&tag) {
                        tags_unique.push(tag);
                    }
                }

                set_processed_value(tags_unique)
            }
        };

        create_effect(move |_| {
            oninput(processed_value());
        });

        view! { <InputText value=Signal::derive(input_value) oninput=oninput_text debounce/> }
    }
}

pub mod metadata {
    //! Common components for editing metadata
    use super::super::INPUT_DEBOUNCE;
    use crate::components::form::InputNumber;
    use leptos::*;
    use syre_core::types::{data::ValueKind, Value};

    #[component]
    pub fn AddDatum(
        #[prop(into)] keys: MaybeSignal<Vec<String>>,
        #[prop(into)] onadd: Callback<(String, Value)>,
    ) -> impl IntoView {
        let (key, set_key) = create_signal("".to_string());
        let key = leptos_use::signal_debounced(key, INPUT_DEBOUNCE);
        let (value, set_value) = create_signal(Value::String("".to_string()));

        let invalid_key = {
            let keys = keys.clone();
            move || {
                key.with(|key| {
                    let key = key.trim();
                    keys.with(|keys| keys.iter().any(|k| k == key))
                })
            }
        };

        let onadd_datum = {
            let keys = keys.clone();
            move |_| {
                if keys
                    .with_untracked(|keys| key.with_untracked(|key| keys.iter().any(|k| k == key)))
                {
                    return;
                }

                let key = key.with_untracked(|key| key.trim().to_string());
                if key.is_empty() {
                    return;
                }

                let value = value.with_untracked(|value| match value {
                    Value::String(value) => Value::String(value.trim().to_string()),
                    Value::Quantity { magnitude, unit } => Value::Quantity {
                        magnitude: magnitude.clone(),
                        unit: unit.trim().to_string(),
                    },
                    Value::Bool(_) | Value::Number(_) | Value::Array(_) | Value::Map(_) => {
                        value.clone()
                    }
                    Value::Null => unreachable!(),
                });

                set_key.update(|key| key.clear());
                set_value(Value::String(String::new()));
                onadd((key, value));
            }
        };

        view! {
            <div>
                <input
                    name="key"
                    class=(["border-red-600", "border-solid", "border-2"], invalid_key.clone())
                    prop:value=key
                    minlength="1"
                    on:input=move |e| set_key(event_target_value(&e))
                />
                <ValueEditor value set_value/>
                <button type="button" on:mousedown=onadd_datum>
                    "+"
                </button>
            </div>
        }
    }

    #[component]
    pub fn ValueEditor(
        #[prop(into)] value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let value_kind = create_memo(move |_| value.with(|value| value.kind()));

        let value_editor = move || {
            value_kind.with(|kind| match kind {
                ValueKind::Bool => {
                    view! { <BoolEditor value set_value/> }
                }
                ValueKind::String => {
                    view! { <StringEditor value set_value/> }
                }
                ValueKind::Number => {
                    view! { <NumberEditor value set_value/> }
                }
                ValueKind::Quantity => {
                    view! { <QuantityEditor value set_value/> }
                }
                ValueKind::Array => {
                    view! { <ArrayEditor value set_value/> }
                }
                ValueKind::Map => {
                    view! { <MapEditor value set_value/> }
                }
            })
        };

        view! {
            <KindSelect value set_value/>
            {value_editor}
        }
    }

    #[component]
    fn KindSelect(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let change = move |e| {
            let kind = string_to_kind(event_target_value(&e)).unwrap();
            set_value(convert_value_kind(value.get(), &kind));
        };

        view! {
            <select
                prop:value=move || {
                    value
                        .with(|value| {
                            value_to_kind_str(value).unwrap_or(kind_to_str(&ValueKind::String))
                        })
                }

                on:change=change
            >
                <option value=kind_to_str(&ValueKind::String)>"String"</option>
                <option value=kind_to_str(&ValueKind::Number)>"Number"</option>
                <option value=kind_to_str(&ValueKind::Quantity)>"Quantity"</option>
                <option value=kind_to_str(&ValueKind::Bool)>"Boolean"</option>
                <option value=kind_to_str(&ValueKind::Array)>"Array"</option>
                <option value=kind_to_str(&ValueKind::Map)>"Map"</option>
            </select>
        }
    }

    #[component]
    fn BoolEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let checked = move || {
            value.with(|value| {
                let Value::Bool(value) = value else {
                    panic!("invalid value");
                };

                *value
            })
        };

        view! {
            <input
                type="checkbox"
                on:input=move |e| set_value(Value::Bool(event_target_checked(&e)))
                checked=checked
            />
        }
    }

    #[component]
    fn StringEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let input_value = move || {
            value.with(|value| {
                let Value::String(value) = value else {
                    panic!("invalid value");
                };

                value.clone()
            })
        };

        view! {
            <input
                type="text"
                prop:value=input_value
                on:input=move |e| set_value(Value::String(event_target_value(&e)))
            />
        }
    }

    #[component]
    fn NumberEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let input_value = move || {
            value.with(|value| {
                let Value::Number(value) = value else {
                    panic!("invalid value");
                };

                value.to_string()
            })
        };

        let oninput = move |value: String| {
            let Ok(value) = serde_json::from_str(&value) else {
                return;
            };

            set_value(Value::Number(value));
        };

        view! { <InputNumber value=Signal::derive(input_value) oninput/> }
    }

    #[component]
    fn QuantityEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let value_magnitude = move || {
            value.with(|value| {
                let Value::Quantity { magnitude, .. } = value else {
                    panic!("invalid value");
                };

                magnitude.to_string()
            })
        };

        let value_unit = move || {
            value.with(|value| {
                let Value::Quantity { unit, .. } = value else {
                    panic!("invalid value");
                };

                unit.clone()
            })
        };

        let oninput_magnitude = move |value: String| {
            let Ok(mag) = value.parse::<f64>() else {
                return;
            };

            set_value.update(move |value| {
                let Value::Quantity { magnitude, .. } = value else {
                    panic!("invalid value");
                };

                *magnitude = mag;
            });
        };

        let oninput_unit = move |e| {
            set_value.update(move |value| {
                let Value::Quantity { unit, .. } = value else {
                    panic!("invalid value");
                };

                *unit = event_target_value(&e).trim().to_string();
            });
        };

        view! {
            <div>
                <InputNumber value=Signal::derive(value_magnitude) oninput=oninput_magnitude/>
                <input prop:value=value_unit minlength=1 on:input=oninput_unit/>
            </div>
        }
    }

    #[component]
    fn ArrayEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        let (input_value, set_input_value) = create_signal(value.with_untracked(|value| {
            let Value::Array(value) = value else {
                panic!("invalid value kind");
            };

            value
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        }));
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        create_effect(move |_| {
            let val = value.with(|value| {
                let Value::Array(value) = value else {
                    panic!("invalid value kind");
                };

                value
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            });

            set_input_value(val);
        });

        create_effect(move |_| {
            let val = input_value.with(|value| {
                value
                    .split([',', '\n', ';'])
                    .filter_map(|elm| {
                        let value = elm.trim();
                        if value.is_empty() {
                            None
                        } else {
                            todo!();
                        }
                    })
                    .collect::<Vec<Value>>()
            });

            set_value(Value::Array(val));
        });

        view! {
            <textarea on:input=move |e| set_input_value(
                event_target_value(&e),
            )>{input_value}</textarea>
        }
    }

    #[component]
    fn MapEditor(
        /// Read signal.
        value: Signal<Value>,
        set_value: WriteSignal<Value>,
    ) -> impl IntoView {
        view! {}
    }

    pub(super) fn value_to_kind(value: &Value) -> Option<ValueKind> {
        match value {
            Value::Null => None,
            Value::Bool(_) => Some(ValueKind::Bool),
            Value::String(_) => Some(ValueKind::String),
            Value::Number(_) => Some(ValueKind::Number),
            Value::Quantity { .. } => Some(ValueKind::Quantity),
            Value::Array(_) => Some(ValueKind::Array),
            Value::Map(_) => Some(ValueKind::Map),
        }
    }

    pub(super) fn value_to_kind_str(value: &Value) -> Option<&'static str> {
        value_to_kind(value).map(|kind| kind_to_str(&kind))
    }

    pub(super) fn kind_to_str(kind: &ValueKind) -> &'static str {
        match kind {
            ValueKind::Bool => "bool",
            ValueKind::String => "string",
            ValueKind::Number => "number",
            ValueKind::Quantity => "quantity",
            ValueKind::Array => "array",
            ValueKind::Map => "map",
        }
    }

    pub(super) fn string_to_kind(s: impl AsRef<str>) -> Option<ValueKind> {
        let s = s.as_ref();
        match s {
            "bool" => Some(ValueKind::Bool),
            "string" => Some(ValueKind::String),
            "number" => Some(ValueKind::Number),
            "quantity" => Some(ValueKind::Quantity),
            "array" => Some(ValueKind::Array),
            "map" => Some(ValueKind::Map),
            _ => None,
        }
    }

    /// Converts [`Value`]s between types.
    /// If a reasonable conversion can not be made, the default value for that type is returned.
    pub(super) fn convert_value_kind(value: Value, target: &ValueKind) -> Value {
        let v = (value, target);
        match v {
            (Value::String(_), ValueKind::String)
            | (Value::Number(_), ValueKind::Number)
            | (Value::Quantity { .. }, ValueKind::Quantity)
            | (Value::Bool(_), ValueKind::Bool)
            | (Value::Array(_), ValueKind::Array)
            | (Value::Map(_), ValueKind::Map) => v.0,

            (Value::Null, _) => match target {
                ValueKind::Bool => Value::Bool(Default::default()),
                ValueKind::String => Value::String(Default::default()),
                ValueKind::Number => Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                ValueKind::Quantity => Value::Quantity {
                    magnitude: 0.0,
                    unit: Default::default(),
                },
                ValueKind::Array => Value::Array(Default::default()),
                ValueKind::Map => Value::Map(Default::default()),
            },

            (Value::String(value), ValueKind::Number) => match str_to_number(&value) {
                Ok(val) => val,
                Err(_) => Value::from(0 as u64),
            },

            (Value::Number(value), ValueKind::String) => value.to_string().into(),

            (Value::String(unit), ValueKind::Quantity) => Value::Quantity {
                magnitude: 0.0,
                unit,
            },

            (Value::Number(magnitude), ValueKind::Quantity) => Value::Quantity {
                magnitude: magnitude.as_f64().unwrap(),
                unit: String::default(),
            },

            (Value::Array(value), ValueKind::String) => serde_json::to_string_pretty(&value)
                .unwrap_or(String::default())
                .into(),

            (Value::Map(value), ValueKind::String) => serde_json::to_string_pretty(&value)
                .unwrap_or(String::default())
                .into(),

            (Value::String(value), ValueKind::Array) => {
                let value = serde_json::to_value(value).unwrap_or_default();
                if value.is_array() {
                    value.into()
                } else {
                    Value::Array(Vec::default())
                }
            }

            (Value::String(value), ValueKind::Map) => {
                let value = serde_json::to_value(value).unwrap_or_default();
                if value.is_object() {
                    value.into()
                } else {
                    Value::Map(syre_core::types::data::Map::default())
                }
            }

            (_, ValueKind::String) => Value::String(String::default()),
            (_, ValueKind::Number) => Value::Number(0.into()),
            (_, ValueKind::Quantity) => Value::Quantity {
                magnitude: 0.0,
                unit: "".to_string(),
            },
            (_, ValueKind::Bool) => Value::Bool(false),
            (_, ValueKind::Array) => Value::Array(Vec::default()),
            (_, ValueKind::Map) => Value::Map(syre_core::types::data::Map::default()),
        }
    }

    /// Converts a string to a number.
    /// Is restrictive as possible in conversion.
    /// i.e. First tries to convert to `u64`, then `i64`, then `f64`.
    ///
    /// # Returns
    /// A [`serde_json::Value`] that is a
    /// + [`Number`](serde_json::value::Number) if the value is finite and parsed correctly.
    /// + `Null` if the value is parsed correclty but `nan`.
    /// + 0 if the value is empty. (This also occurs if the string is an invalid number.)
    ///
    /// # Errors
    /// + If the value can not be parsed as a number.
    fn str_to_number(input: &str) -> Result<Value, ()> {
        fn parse_as_int(input: &str) -> Option<Value> {
            if let Ok(val) = input.parse::<u64>() {
                return Some(Value::from(val));
            }

            if let Ok(val) = input.parse::<i64>() {
                return Some(Value::from(val));
            }

            None
        }

        if input.is_empty() {
            return Ok(Value::from(0 as u64));
        }

        match input.split_once('.') {
            None => match parse_as_int(input) {
                Some(val) => Ok(val),
                None => Err(()),
            },

            Some((_, decs)) => {
                if decs.is_empty() {
                    match parse_as_int(input) {
                        Some(val) => Ok(val),
                        None => Err(()),
                    }
                } else {
                    let Ok(val) = input.parse::<f64>() else {
                        return Err(());
                    };

                    match val.is_nan() {
                        true => Ok(Value::Null),
                        false => Ok(Value::from(val)),
                    }
                }
            }
        }
    }
}

pub mod analysis_associations {
    use leptos::*;
    use std::str::FromStr;
    use syre_core::{self as core, types::ResourceId};

    /// Indicates the kind of the analysis.
    /// Represents a stripped version of [`syre_local::types::analysis::AnalysisKind`].
    #[derive(Clone, Debug)]
    pub enum AnalysisKind {
        Script,
        ExcelTemplate,
    }

    #[derive(Clone, Debug)]
    pub struct AnalysisInfo {
        rid: ResourceId,
        title: String,
        kind: AnalysisKind,
    }

    impl AnalysisInfo {
        pub fn script(rid: ResourceId, title: impl Into<String>) -> Self {
            Self {
                rid,
                title: title.into(),
                kind: AnalysisKind::Script,
            }
        }

        pub fn excel_template(rid: ResourceId, title: impl Into<String>) -> Self {
            Self {
                rid,
                title: title.into(),
                kind: AnalysisKind::ExcelTemplate,
            }
        }
    }

    #[component]
    pub fn AddAssociation(
        #[prop(into)] available_analyses: Signal<Vec<AnalysisInfo>>,
        #[prop(into)] onadd: Callback<core::project::AnalysisAssociation>,
        #[prop(into)] oncancel: Callback<()>,
    ) -> impl IntoView {
        let analysis_node = create_node_ref::<html::Select>();
        let priority_node = create_node_ref::<html::Input>();
        let autorun_node = create_node_ref::<html::Input>();

        let add = move |_| {
            let analysis = analysis_node.get().unwrap();
            let analysis = ResourceId::from_str(&analysis.value()).unwrap();

            let priority = priority_node.get().unwrap();
            let priority =
                priority.value_as_number() as core::project::analysis_association::Priority;

            let autorun = autorun_node.get().unwrap();
            let autorun = autorun.checked();

            let association =
                core::project::AnalysisAssociation::with_params(analysis, autorun, priority);

            onadd(association);
        };

        let cancel = move |_| {
            oncancel(());
        };

        view! {
            <div>
                <div>
                    <select ref=analysis_node>
                        <Show
                            when=move || available_analyses.with(|analyses| !analyses.is_empty())
                            fallback=move || {
                                view! {
                                    <option value="" disabled=true>
                                        "(no analyses available)"
                                    </option>
                                }
                            }
                        >

                            <For
                                each=available_analyses
                                key=|analysis| analysis.rid.clone()
                                let:analysis
                            >
                                <option value=analysis.rid.to_string()>{analysis.title}</option>
                            </For>
                        </Show>
                    </select>
                    <input ref=priority_node type="number" name="priority" value="0"/>
                    <input ref=autorun_node type="checkbox" name="autorun" checked=true/>
                </div>
                <div>
                    <button type="button" on:mousedown=add>
                        "+"
                    </button>
                    <button type="button" on:mousedown=cancel>
                        "Cancel"
                    </button>
                </div>
            </div>
        }
    }
}

pub mod bulk {
    //! Types for bulk editing.
    pub use metadata::Metadata;

    #[derive(Clone, PartialEq, Debug)]
    pub enum Value<T> {
        Equal(T),
        Mixed,
    }

    impl<T> Value<T> {
        pub fn is_equal(&self) -> bool {
            match self {
                Self::Equal(_) => true,
                Self::Mixed => false,
            }
        }

        pub fn is_mixed(&self) -> bool {
            !self.is_equal()
        }

        pub fn unwrap(self) -> T {
            match self {
                Value::Equal(value) => value,
                Value::Mixed => panic!("unwrapped `Mixed` value"),
            }
        }

        pub fn unwrap_or(self, or: T) -> T {
            match self {
                Value::Equal(value) => value,
                Value::Mixed => or,
            }
        }
    }

    pub mod kind {
        use super::Value;
        use crate::components::form::debounced::InputText;
        use leptos::*;

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Value<Option<String>>>,
            #[prop(into)] oninput: Callback<Option<String>>,
            #[prop(into)] debounce: MaybeSignal<f64>,
        ) -> impl IntoView {
            let (processed_value, set_processed_value) = create_signal({
                value.with_untracked(|value| match value {
                    Value::Mixed | Value::Equal(None) => None,
                    Value::Equal(Some(value)) => Some(value.clone()),
                })
            });

            let input_value = {
                let value = value.clone();
                move || {
                    value.with(|value| match value {
                        Value::Mixed | Value::Equal(None) => String::new(),
                        Value::Equal(Some(value)) => value.clone(),
                    })
                }
            };

            let oninput_text = {
                move |value: String| {
                    let value = value.trim();
                    let value = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    };

                    set_processed_value(value);
                }
            };

            let placeholder = {
                let value = value.clone();
                move || {
                    value.with(|value| match value {
                        Value::Mixed => "(mixed)".to_string(),
                        Value::Equal(_) => "(empty)".to_string(),
                    })
                }
            };

            let _ = watch(
                processed_value,
                move |processed_value, _, _| {
                    oninput(processed_value.clone());
                },
                false,
            );

            view! {
                <InputText
                    value=Signal::derive(input_value)
                    oninput=oninput_text
                    debounce
                    placeholder=Signal::derive(placeholder)
                />
            }
        }
    }

    pub mod description {
        use super::Value;
        use crate::components::form::debounced::TextArea;
        use leptos::*;

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Value<Option<String>>>,
            #[prop(into)] oninput: Callback<Option<String>>,
            #[prop(into)] debounce: MaybeSignal<f64>,
        ) -> impl IntoView {
            let (processed_value, set_processed_value) = create_signal({
                value.with_untracked(|value| match value {
                    Value::Mixed | Value::Equal(None) => None,
                    Value::Equal(Some(value)) => Some(value.clone()),
                })
            });

            let input_value = {
                let value = value.clone();
                move || {
                    value.with(|value| match value {
                        Value::Mixed | Value::Equal(None) => String::new(),
                        Value::Equal(Some(value)) => value.clone(),
                    })
                }
            };

            let oninput_text = {
                move |value: String| {
                    let value = value.trim();
                    let value = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    };

                    set_processed_value(value);
                }
            };

            let placeholder = {
                let value = value.clone();
                move || {
                    value.with(|value| match value {
                        Value::Mixed => "(mixed)".to_string(),
                        Value::Equal(_) => "(empty)".to_string(),
                    })
                }
            };

            let _ = watch(
                processed_value,
                move |processed_value, _, _| {
                    oninput(processed_value.clone());
                },
                false,
            );

            view! {
                <TextArea
                    value=Signal::derive(input_value)
                    oninput=oninput_text
                    debounce
                    placeholder=Signal::derive(placeholder)
                />
            }
        }
    }

    pub mod tags {
        use leptos::*;

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Vec<String>>,
            #[prop(into)] onadd: Callback<Vec<String>>,
            #[prop(into)] onremove: Callback<String>,
        ) -> impl IntoView {
            let input_ref = create_node_ref::<html::Input>();
            let add_tags = move |e| {
                let input = input_ref.get_untracked().unwrap();
                let input_value = input.value();
                if input_value.trim().is_empty() {
                    return;
                }

                input.set_value("");
                let mut tags = input_value
                    .split(",")
                    .filter_map(|tag| {
                        let tag = tag.trim();
                        if tag.is_empty() {
                            None
                        } else {
                            Some(tag.to_string())
                        }
                    })
                    .collect::<Vec<_>>();

                tags.sort();
                tags.dedup();
                onadd(tags);
            };

            view! {
                <div>
                    <div>
                        <input ref=input_ref type="text" placeholder="Add tags"/>
                        <button type="button" on:mousedown=add_tags>
                            "+"
                        </button>
                    </div>
                    <TagsList value onremove/>
                </div>
            }
        }

        #[component]
        fn TagsList(value: MaybeSignal<Vec<String>>, onremove: Callback<String>) -> impl IntoView {
            view! {
                <div>
                    <ul>
                        {move || {
                            value
                                .with(|tags| {
                                    tags.iter()
                                        .map(|tag| {
                                            view! {
                                                <li>
                                                    {tag.clone()}
                                                    <button
                                                        type="button"
                                                        on:mousedown={
                                                            let tag = tag.clone();
                                                            move |_| onremove(tag.clone())
                                                        }
                                                    >

                                                        "-"
                                                    </button>
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                })
                        }}

                    </ul>
                </div>
            }
        }
    }

    pub mod metadata {
        use super::super::{
            super::INPUT_DEBOUNCE,
            metadata::{convert_value_kind, kind_to_str, string_to_kind, value_to_kind_str},
        };
        use crate::components::form::InputNumber;
        use leptos::*;
        use syre_core::types::data;

        #[derive(Clone, Debug)]
        pub enum Value {
            /// Values have mixed kinds.
            MixedKind,

            /// Values have equal kinds but mixed values.
            EqualKind(data::ValueKind),

            /// Equal kind and value.
            Equal(data::Value),
        }

        impl Value {
            pub fn is_mixed_kind(&self) -> bool {
                matches!(self, Self::MixedKind)
            }
        }

        pub type Metadata = Vec<Metadatum>;
        pub type Metadatum = (String, Value);

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Metadata>,
            #[prop(into)] onremove: Callback<String>,
            #[prop(into)] onmodify: Callback<(String, data::Value)>,
        ) -> impl IntoView {
            // TODO: This signal with the watch is a work around to allow
            // `containers` signal in the callback function.
            // See https://github.com/leptos-rs/leptos/issues/2041.
            let (modified, set_modified) = create_signal(("".to_string(), data::Value::Null));
            let _ = watch(
                modified,
                move |modified, _, _| {
                    onmodify(modified.clone());
                },
                false,
            );

            view! {
                <div>
                    {move || {
                        value
                            .with(|value| {
                                value
                                    .iter()
                                    .map(|(key, value)| {
                                        view! {
                                            <DatumEditor
                                                key=key.clone()
                                                value=value.clone()
                                                oninput={
                                                    let key = key.clone();
                                                    move |value| set_modified((key.clone(), value))
                                                }
                                            />

                                            <button
                                                type="button"
                                                on:mousedown={
                                                    let key = key.clone();
                                                    move |_| onremove(key.clone())
                                                }
                                            >

                                                "-"
                                            </button>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                    }}

                </div>
            }
        }

        #[component]
        fn DatumEditor(
            key: String,
            value: Value,
            #[prop(into)] oninput: Callback<data::Value>,
        ) -> impl IntoView {
            view! {
                <div>
                    <span>{key}</span>
                    <ValueEditor value oninput/>
                </div>
            }
        }

        #[component]
        pub fn ValueEditor(
            value: Value,
            #[prop(into)] oninput: Callback<data::Value>,
        ) -> impl IntoView {
            let value_editor = {
                let value = value.clone();
                let oninput = oninput.clone();
                move || match value {
                    Value::MixedKind => view! {}.into_view(),
                    Value::EqualKind(data::ValueKind::Bool)
                    | Value::Equal(data::Value::Bool(_)) => {
                        view! { <BoolEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::EqualKind(data::ValueKind::String)
                    | Value::Equal(data::Value::String(_)) => {
                        view! { <StringEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::EqualKind(data::ValueKind::Number)
                    | Value::Equal(data::Value::Number(_)) => {
                        view! { <NumberEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::EqualKind(data::ValueKind::Quantity)
                    | Value::Equal(data::Value::Quantity { .. }) => {
                        view! { <QuantityEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::EqualKind(data::ValueKind::Array)
                    | Value::Equal(data::Value::Array(_)) => {
                        view! { <ArrayEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::EqualKind(data::ValueKind::Map) | Value::Equal(data::Value::Map(_)) => {
                        view! { <MapEditor value=value.clone() oninput/> }.into_view()
                    }
                    Value::Equal(data::Value::Null) => unreachable!(),
                }
            };

            view! {
                <KindSelect value onchange=oninput/>
                {value_editor}
            }
        }

        #[component]
        fn KindSelect(
            /// Read signal.
            value: Value,
            onchange: Callback<data::Value>,
        ) -> impl IntoView {
            let change = {
                let value = value.clone();
                move |e| {
                    let kind = string_to_kind(event_target_value(&e)).unwrap();
                    if let Value::Equal(ref value) = value {
                        onchange(convert_value_kind(value.clone(), &kind));
                    } else {
                        onchange(convert_value_kind(data::Value::Null, &kind));
                    }
                }
            };

            view! {
                <select
                    prop:value={
                        let value = value.clone();
                        move || match value {
                            Value::Equal(ref value) => {
                                value_to_kind_str(&value)
                                    .unwrap_or(kind_to_str(&data::ValueKind::String))
                            }
                            Value::EqualKind(ref kind) => kind_to_str(&kind),
                            Value::MixedKind => "",
                        }
                    }

                    on:change=change
                >
                    {move || {
                        if value.is_mixed_kind() {
                            view! {
                                <option value="" disabled=true selected>
                                    "(mixed)"
                                </option>
                            }
                                .into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    <option value=kind_to_str(&data::ValueKind::String)>"String"</option>
                    <option value=kind_to_str(&data::ValueKind::Number)>"Number"</option>
                    <option value=kind_to_str(&data::ValueKind::Quantity)>"Quantity"</option>
                    <option value=kind_to_str(&data::ValueKind::Bool)>"Boolean"</option>
                    <option value=kind_to_str(&data::ValueKind::Array)>"Array"</option>
                    <option value=kind_to_str(&data::ValueKind::Map)>"Map"</option>
                </select>
            }
        }

        #[component]
        fn BoolEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            let checked = move || match value {
                Value::EqualKind(_) => false,
                Value::Equal(data::Value::Bool(value)) => value,
                Value::MixedKind | Value::Equal(_) => unreachable!(),
            };

            view! {
                <input
                    type="checkbox"
                    on:input=move |e| oninput(data::Value::Bool(event_target_checked(&e)))
                    checked=checked
                />
            }
        }

        #[component]
        fn StringEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            let input_value = {
                let value = value.clone();
                move || match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::String(ref value)) => value.clone(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            };

            let placeholder = {
                let value = value.clone();
                move || match value {
                    Value::EqualKind(_) => "(mixed)",
                    Value::Equal(data::Value::String(_)) => "",
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            };

            view! {
                <input
                    type="text"
                    prop:value=input_value
                    on:input=move |e| oninput(data::Value::String(event_target_value(&e)))
                    placeholder=placeholder
                />
            }
        }

        #[component]
        fn NumberEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            let input_value = {
                let value = value.clone();
                move || match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Number(ref value)) => value.to_string(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            };

            let placeholder = {
                let value = value.clone();
                move || match value {
                    Value::EqualKind(_) => "(mixed)".to_string(),
                    Value::Equal(data::Value::Number(_)) => "".to_string(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            };

            let oninput_text = move |value: String| {
                let Ok(value) = serde_json::from_str(&value) else {
                    return;
                };

                // oninput(data::Value::Number(value));
                tracing::debug!("{:?}", data::Value::Number(value));
            };

            view! {
                <InputNumber
                    value=Signal::derive(input_value)
                    oninput=oninput_text
                    placeholder=Signal::derive(placeholder)
                />
            }
        }

        #[component]
        fn QuantityEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            let (magnitude, set_magnitude) = create_signal({
                match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Quantity { ref magnitude, .. }) => {
                        magnitude.to_string()
                    }
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            });

            let (unit, set_unit) = create_signal({
                match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Quantity { ref unit, .. }) => unit.clone(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            });

            let oninput_magnitude = move |magnitude: String| {
                set_magnitude(magnitude);
            };

            let oninput_unit = move |e| {
                set_unit(event_target_value(&e));
            };

            let _ = watch(
                move || (magnitude, unit),
                move |(magnitude, unit), _, _| {
                    let Ok(magnitude) = magnitude.with(|magnitude| magnitude.parse::<f64>()) else {
                        return;
                    };

                    if unit.with(|unit| unit.is_empty()) {
                        return;
                    }

                    oninput(data::Value::Quantity {
                        magnitude,
                        unit: unit(),
                    });
                },
                false,
            );

            view! {
                <div>
                    <InputNumber value=magnitude oninput=oninput_magnitude/>
                    <input prop:value=unit minlength="1" on:input=oninput_unit/>
                </div>
            }
        }

        #[component]
        fn ArrayEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            let (input_value, set_input_value) = create_signal(match value {
                Value::EqualKind(_) => "".to_string(),
                Value::Equal(data::Value::Array(ref value)) => value
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(",\n"),
                Value::MixedKind | Value::Equal(_) => unreachable!(),
            });
            let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

            let placeholder = {
                let value = value.clone();
                move || match value {
                    Value::EqualKind(_) => "(mixed)",
                    Value::Equal(data::Value::Array(_)) => "",
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            };

            create_effect(move |_| {
                let val = input_value.with(|value| {
                    value
                        .split([',', '\n', ';'])
                        .filter_map(|elm| {
                            let value = elm.trim();
                            if value.is_empty() {
                                None
                            } else {
                                todo!();
                            }
                        })
                        .collect::<Vec<data::Value>>()
                });

                oninput(data::Value::Array(val));
            });

            view! {
                <textarea
                    on:input=move |e| set_input_value(event_target_value(&e))
                    placeholder=placeholder
                >
                    {input_value}
                </textarea>
            }
        }

        #[component]
        fn MapEditor(value: Value, oninput: Callback<data::Value>) -> impl IntoView {
            view! {}
        }
    }
}
