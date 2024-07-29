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

/// Common components for editing metadata
pub mod metadata {
    use super::super::INPUT_DEBOUNCE;
    use crate::components::form::InputNumber;
    use leptos::*;
    use syre_core::types::{data::ValueKind, Value};

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

    fn value_to_kind(value: &Value) -> Option<ValueKind> {
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

    fn value_to_kind_str(value: &Value) -> Option<&'static str> {
        value_to_kind(value).map(|kind| kind_to_str(&kind))
    }

    fn kind_to_str(kind: &ValueKind) -> &'static str {
        match kind {
            ValueKind::Bool => "bool",
            ValueKind::String => "string",
            ValueKind::Number => "number",
            ValueKind::Quantity => "quantity",
            ValueKind::Array => "array",
            ValueKind::Map => "map",
        }
    }

    fn string_to_kind(s: impl AsRef<str>) -> Option<ValueKind> {
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
    fn convert_value_kind(value: Value, target: &ValueKind) -> Value {
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

pub mod bulk {
    //! Types for bulk editing.

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
}
