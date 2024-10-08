pub mod kind {
    use crate::components::form::debounced::InputText;
    use leptos::*;

    #[component]
    pub fn Editor(
        #[prop(into)] value: MaybeSignal<Option<String>>,
        #[prop(into)] oninput: Callback<Option<String>>,
        #[prop(into)] debounce: MaybeSignal<f64>,
        #[prop(into, optional)] class: MaybeProp<String>,
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

        view! { <InputText value=Signal::derive(input_value) oninput=oninput_text debounce class /> }
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
        #[prop(optional, into)] class: MaybeProp<String>,
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

        view! { <TextArea value=Signal::derive(input_value) oninput=oninput_text debounce class /> }
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
        #[prop(optional, into)] class: MaybeProp<String>,
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

        view! { <InputText value=Signal::derive(input_value) oninput=oninput_text debounce class /> }
    }
}

pub mod metadata {
    //! Common components for editing metadata
    use super::super::INPUT_DEBOUNCE;
    use crate::components::form::InputNumber;
    use leptos::{ev::SubmitEvent, *};
    use leptos_icons::Icon;
    use syre_core::types::{data::ValueKind, Value};

    #[component]
    pub fn AddDatum(
        #[prop(into)] keys: MaybeSignal<Vec<String>>,
        #[prop(into)] onadd: Callback<(String, Value)>,
        /// Reset the state of the form.
        #[prop(optional, into)]
        reset: Option<ReadSignal<()>>,
        #[prop(optional, into)] id: MaybeProp<String>,
        #[prop(optional, into)] class: MaybeProp<String>,
    ) -> impl IntoView {
        let (key, set_key) = create_signal("".to_string());
        let key = leptos_use::signal_debounced(key, INPUT_DEBOUNCE);
        let (value, set_value) = create_signal(Value::Number(serde_json::Number::from(0)));

        if let Some(reset) = reset {
            let _ = watch(
                reset,
                move |_, _, _| set_value(Value::Number(serde_json::Number::from(0))),
                false,
            );
        }

        let invalid_key = {
            let keys = keys.clone();
            move || {
                key.with(|key| {
                    let key = key.trim();
                    keys.with(|keys| keys.iter().any(|k| k == key))
                })
            }
        };

        let submit = {
            let keys = keys.clone();
            move |e: SubmitEvent| {
                e.prevent_default();

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
                    Value::Bool(_) | Value::Number(_) | Value::Array(_) => value.clone(),
                    Value::Null => unreachable!(),
                });

                set_key.update(|key| key.clear());
                set_value(Value::Number(serde_json::Number::from(0)));
                onadd((key, value));
            }
        };

        view! {
            <form on:submit=submit id=id class=class>
                <div class="pb-1">
                    <input
                        name="key"
                        on:input=move |e| set_key(event_target_value(&e))
                        prop:value=key
                        placeholder="Name"
                        minlength="1"
                        class=(["border-red-600", "border-solid", "border-2"], invalid_key.clone())
                        class="input-compact w-full"
                    />
                </div>
                <ValueEditor value set_value />
                <div class="py-1 flex justify-center">
                    <button class="rounded-sm hover:bg-primary-400 dark:hover:bg-primary-700">
                        <Icon icon=icondata::AiPlusOutlined />
                    </button>
                </div>
            </form>
        }
    }

    #[component]
    pub fn ValueEditor(
        #[prop(into)] value: Signal<Value>,
        set_value: WriteSignal<Value>,
        #[prop(optional, into)] class: MaybeProp<String>,
    ) -> impl IntoView {
        let value_kind = create_memo(move |_| value.with(|value| value.kind()));
        let value_editor = move || {
            value_kind.with(|kind| match kind {
                ValueKind::Bool => {
                    view! { <BoolEditor value set_value /> }
                }
                ValueKind::String => {
                    view! { <StringEditor value set_value /> }
                }
                ValueKind::Number => {
                    view! { <NumberEditor value set_value /> }
                }
                ValueKind::Quantity => {
                    view! { <QuantityEditor value set_value /> }
                }
                ValueKind::Array => {
                    view! { <ArrayEditor value set_value /> }
                }
            })
        };

        let class = move || {
            let mut class = class.get().unwrap_or("".to_string());
            class.push_str("flex flex-wrap gap-2");
            class
        };

        view! {
            <div class=class>
                <KindSelect value set_value />
                {value_editor}
            </div>
        }
    }

    #[component]
    fn KindSelect(value: Signal<Value>, set_value: WriteSignal<Value>) -> impl IntoView {
        let input_node = NodeRef::<html::Select>::new();

        create_effect(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };

            value.with(|value| {
                if let Some(value) = value_to_kind_str(value) {
                    input.set_value(value);
                }
            });
        });

        let change = move |e| {
            let kind = string_to_kind(event_target_value(&e)).unwrap();
            set_value(convert_value_kind(value.get(), &kind));
        };

        view! {
            <select
                ref=input_node
                prop:value=move || {
                    value
                        .with(|value| {
                            value_to_kind_str(value).unwrap_or(kind_to_str(&ValueKind::Quantity))
                        })
                }

                on:change=change
                class="input-compact pr-4 w-full"
            >
                <option value=kind_to_str(&ValueKind::Number)>"Number"</option>
                <option value=kind_to_str(&ValueKind::Quantity)>"Quantity"</option>
                <option value=kind_to_str(&ValueKind::Bool)>"Boolean"</option>
                <option value=kind_to_str(&ValueKind::String)>"String"</option>
                <option value=kind_to_str(&ValueKind::Array)>"List"</option>
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
                placeholder="Value"
                class="input-compact w-full"
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

        view! {
            <InputNumber
                value=Signal::derive(input_value)
                oninput
                placeholder="Value"
                class="input-compact w-full"
            />
        }
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
            <div class="flex flex-wrap w-full">
                <InputNumber
                    value=Signal::derive(value_magnitude)
                    oninput=oninput_magnitude
                    placeholder="Magnitude"
                    class="input-compact"
                />

                <input
                    prop:value=value_unit
                    minlength=1
                    on:input=oninput_unit
                    placeholder="Units"
                    class="input-compact"
                />
            </div>
        }
    }

    #[component]
    fn ArrayEditor(value: Signal<Value>, set_value: WriteSignal<Value>) -> impl IntoView {
        let (error, set_error) = create_signal(None);
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
            set_error(None);
            let val = input_value.with(|value| {
                value
                    .split([',', '\n', ';'])
                    .filter_map(|elm| {
                        let value = elm.trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(serde_json::from_str::<Value>(elm))
                        }
                    })
                    .collect::<serde_json::Result<Vec<_>>>()
            });

            match val {
                Ok(val) => {
                    let val = Value::Array(val);
                    if value.with_untracked(|value| *value != val) {
                        set_value(val);
                    }
                }
                Err(err) => set_error(Some(err)),
            }
        });

        view! {
            <textarea
                on:input=move |e| set_input_value(event_target_value(&e))
                placeholder="Separate values by comma, semicolon, or new line."
                class=(
                    ["border-2", "!border-syre-red-600", "focus:ring-syre-red-600"],
                    move || error.with(|error| error.is_some()),
                )
                class="input-compact align-top overflow-auto scrollbar-thin"
                title="Separate values by comma, semicolon, or new line."
            >
                {input_value}
            </textarea>
        }
    }

    pub(super) fn value_to_kind(value: &Value) -> Option<ValueKind> {
        match value {
            Value::Null => None,
            Value::Bool(_) => Some(ValueKind::Bool),
            Value::String(_) => Some(ValueKind::String),
            Value::Number(_) => Some(ValueKind::Number),
            Value::Quantity { .. } => Some(ValueKind::Quantity),
            Value::Array(_) => Some(ValueKind::Array),
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
            | (Value::Array(_), ValueKind::Array) => v.0,

            (Value::Null, _) => match target {
                ValueKind::Bool => Value::Bool(Default::default()),
                ValueKind::String => Value::String(Default::default()),
                ValueKind::Number => Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                ValueKind::Quantity => Value::Quantity {
                    magnitude: 0.0,
                    unit: Default::default(),
                },
                ValueKind::Array => Value::Array(Default::default()),
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

            (Value::String(value), ValueKind::Array) => {
                let value = serde_json::to_value(value).unwrap_or_default();
                if value.is_array() {
                    value.into()
                } else {
                    Value::Array(Vec::default())
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
    use leptos_icons::Icon;
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
        #[prop(optional, into)] class: MaybeProp<String>,
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

        view! {
            <div class=class>
                <div>
                    <div class="pb-1">
                        <select ref=analysis_node class="input-compact w-full">
                            <Show
                                when=move || {
                                    available_analyses.with(|analyses| !analyses.is_empty())
                                }

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
                    </div>
                    <div class="flex gap-1">
                        <input
                            ref=priority_node
                            type="number"
                            name="priority"
                            value="0"
                            // TODO: May not want to use hard coded width
                            class="input-compact min-w-14"
                        />
                        <input
                            ref=autorun_node
                            type="checkbox"
                            name="autorun"
                            checked=true
                            class="input-compact"
                        />
                    </div>
                </div>
                <div class="py-1 flex justify-center">
                    <button
                        type="button"
                        on:mousedown=add
                        class="hover:bg-primary-400 dark:hover:bg-primary-700 rounded-sm"
                    >
                        <Icon icon=icondata::AiPlusOutlined />
                    </button>
                </div>
            </div>
        }
    }
}

pub mod bulk {
    //! Types for bulk editing.
    pub use metadata::{Metadata, Metadatum};

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

        /// Converts from `Value<T>` to `Option<T>``.
        pub fn equal(self) -> Option<T> {
            match self {
                Value::Equal(value) => Some(value),
                Value::Mixed => None,
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
                        Value::Mixed => Some("(mixed)".to_string()),
                        Value::Equal(_) => Some("(empty)".to_string()),
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
                    placeholder=MaybeProp::derive(placeholder)
                    class="input-compact"
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
            #[prop(optional, into)] class: MaybeProp<String>,
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
                        Value::Mixed => Some("(mixed)".to_string()),
                        Value::Equal(_) => Some("(empty)".to_string()),
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
                    placeholder=MaybeProp::derive(placeholder)
                    class
                />
            }
        }
    }

    pub mod tags {
        use crate::types;
        use leptos::{
            ev::{MouseEvent, SubmitEvent},
            *,
        };
        use leptos_icons::Icon;
        use wasm_bindgen::JsCast;

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Vec<String>>,
            #[prop(into)] onremove: Callback<String>,

            /// Classes applied to outer container.
            #[prop(optional, into)]
            class: MaybeProp<String>,

            /// Classes applied to individual tags.
            #[prop(optional, into)]
            tag_class: MaybeProp<String>,
        ) -> impl IntoView {
            let tag_class = move || {
                let mut class = tag_class.get().unwrap_or("".to_string());
                class.push_str(" flex pr-2 rounded-full border border-secondary-900 dark:border-secondary-200 dark:bg-secondary-700");
                class
            };

            let remove = move |tag: String| {
                move |e: MouseEvent| {
                    if e.button() == types::MouseButton::Primary {
                        onremove(tag.clone());
                    }
                }
            };

            view! {
                <div class=class>
                    <ul class="flex gap-2 flex-wrap">
                        {move || {
                            value
                                .with(|tags| {
                                    tags.iter()
                                        .map(|tag| {
                                            view! {
                                                <li class=tag_class.clone()>
                                                    <span class="px-2">{tag.clone()}</span>
                                                    <button
                                                        type="button"
                                                        on:mousedown=remove(tag.clone())
                                                        class="aspect-square h-full rounded-full hover:bg-secondary-200 dark:hover:bg-secondary-600"
                                                    >

                                                        <Icon icon=icondata::AiMinusOutlined class="inline-block" />
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

        #[component]
        pub fn AddTags(
            #[prop(into)] onadd: Callback<Vec<String>>,
            /// Reset the state of the form.
            #[prop(optional, into)]
            reset: Option<ReadSignal<()>>,
            #[prop(optional, into)] class: MaybeProp<String>,
        ) -> impl IntoView {
            let input_ref = create_node_ref::<html::Input>();

            if let Some(reset) = reset {
                let _ = watch(
                    reset,
                    move |_, _, _| {
                        let input = input_ref.get_untracked().unwrap();
                        let input = input.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                        input.set_value("");
                    },
                    false,
                );
            }

            let add_tags = move |e: SubmitEvent| {
                e.prevent_default();

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
                <form on:submit=add_tags class=class>
                    <input
                        ref=input_ref
                        type="text"
                        placeholder="Add tags"
                        class="input-compact w-full"
                    />
                    <div class="py-1 flex justify-center">
                        <button
                            type="button"
                            class="rounded-sm hover:bg-primary-400 dark:hover:bg-primary-700"
                        >
                            <Icon icon=icondata::AiPlusOutlined />
                        </button>
                    </div>
                </form>
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
        use leptos_icons::Icon;
        use serde::Deserialize;
        use syre_core::types::data;

        #[derive(PartialEq, Clone, Debug)]
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

        #[derive(Clone, Debug)]
        pub struct Metadatum {
            key: String,
            values: Vec<ReadSignal<syre_core::types::Value>>,
        }

        impl Metadatum {
            pub fn new(key: String, values: Vec<ReadSignal<syre_core::types::Value>>) -> Self {
                assert!(values.len() > 1);
                Self { key, values }
            }

            pub fn key(&self) -> &String {
                &self.key
            }

            pub fn value(&self) -> Signal<Value> {
                Signal::derive({
                    let values = self.values.clone();
                    move || {
                        let mut values = values.iter();
                        let value = values.next().unwrap();
                        let value = Value::Equal(value.get());
                        values
                            .try_fold(value, |value, datum| match value {
                                Value::MixedKind => unreachable!(),
                                Value::EqualKind(ref value_kind) => {
                                    if datum.with(|datum| datum.kind() != *value_kind) {
                                        None
                                    } else {
                                        Some(value)
                                    }
                                }
                                Value::Equal(ref val) => {
                                    if datum.with(|datum| datum.kind() != val.kind()) {
                                        None
                                    } else if datum.with(|datum| datum != val) {
                                        Some(Value::EqualKind(val.kind()))
                                    } else {
                                        Some(value)
                                    }
                                }
                            })
                            .unwrap_or(Value::MixedKind)
                    }
                })
            }
        }

        #[component]
        pub fn Editor(
            #[prop(into)] value: MaybeSignal<Metadata>,
            #[prop(into)] onremove: Callback<String>,
            #[prop(into)] onmodify: Callback<(String, data::Value)>,
        ) -> impl IntoView {
            let value_sorted = move || {
                let mut value = value.get();
                value.sort_by_key(|datum| datum.key().to_lowercase());
                value
            };

            view! {
                <div class="flex flex-col gap-2">
                    <For each=value_sorted key=|datum| datum.key().clone() let:datum>
                        <DatumEditor
                            key=datum.key().clone()
                            value=datum.value()
                            oninput={
                                let key = datum.key().clone();
                                Callback::new(move |value| onmodify((key.clone(), value)))
                            }

                            onremove=Callback::new({
                                let key = datum.key().clone();
                                move |_| onremove(key.clone())
                            })
                        />

                    </For>
                </div>
            }
        }

        #[component]
        fn DatumEditor(
            key: String,
            value: Signal<Value>,
            #[prop(into)] oninput: Callback<data::Value>,
            #[prop(into)] onremove: Callback<()>,
            #[prop(optional, into)] class: MaybeProp<String>,
        ) -> impl IntoView {
            view! {
                <div class=class>
                    <div class="flex">
                        <span class="grow">{key}</span>

                        <button
                            type="button"
                            on:mousedown=move |_| onremove(())
                            class="aspect-square h-full rounded-sm hover:bg-secondary-200 dark:hover:bg-secondary-700"
                        >

                            <Icon icon=icondata::AiMinusOutlined />
                        </button>
                    </div>
                    <ValueEditor value oninput />
                </div>
            }
        }

        #[component]
        pub fn ValueEditor(
            value: Signal<Value>,
            #[prop(into)] oninput: Callback<data::Value>,
        ) -> impl IntoView {
            let value_editor = {
                let oninput = oninput.clone();
                move || {
                    value.with(|val| match val {
                        Value::MixedKind => view! {}.into_view(),
                        Value::EqualKind(data::ValueKind::Bool)
                        | Value::Equal(data::Value::Bool(_)) => {
                            view! { <BoolEditor value oninput /> }.into_view()
                        }
                        Value::EqualKind(data::ValueKind::String)
                        | Value::Equal(data::Value::String(_)) => {
                            view! { <StringEditor value oninput /> }.into_view()
                        }
                        Value::EqualKind(data::ValueKind::Number)
                        | Value::Equal(data::Value::Number(_)) => {
                            view! { <NumberEditor value oninput /> }.into_view()
                        }
                        Value::EqualKind(data::ValueKind::Quantity)
                        | Value::Equal(data::Value::Quantity { .. }) => {
                            view! { <QuantityEditor value oninput /> }.into_view()
                        }
                        Value::EqualKind(data::ValueKind::Array)
                        | Value::Equal(data::Value::Array(_)) => {
                            view! { <ArrayEditor value oninput /> }.into_view()
                        }
                        Value::Equal(data::Value::Null) => unreachable!(),
                    })
                }
            };

            view! {
                <div class="flex flex-wrap">
                    <KindSelect value onchange=oninput />
                    {value_editor}
                </div>
            }
        }

        #[component]
        fn KindSelect(
            /// Read signal.
            value: Signal<Value>,
            onchange: Callback<data::Value>,
        ) -> impl IntoView {
            let input_node = NodeRef::<html::Select>::new();
            create_effect(move |_| {
                let Some(input) = input_node.get() else {
                    return;
                };

                value.with(|value| match &value {
                    Value::Equal(value) => {
                        if let Some(value) = value_to_kind_str(value) {
                            input.set_value(value);
                        }
                    }
                    _ => {}
                })
            });

            let change = move |e| {
                let kind = string_to_kind(event_target_value(&e)).unwrap();
                value.with(|value| {
                    if let Value::Equal(value) = value {
                        onchange(convert_value_kind(value.clone(), &kind));
                    } else {
                        onchange(convert_value_kind(data::Value::Null, &kind));
                    }
                })
            };

            view! {
                <select
                    ref=input_node
                    prop:value=move || {
                        value
                            .with(|value| match value {
                                Value::Equal(ref value) => {
                                    value_to_kind_str(&value)
                                        .unwrap_or(kind_to_str(&data::ValueKind::Number))
                                }
                                Value::EqualKind(ref kind) => kind_to_str(&kind),
                                Value::MixedKind => "",
                            })
                    }

                    on:change=change
                    class="input-compact pr-4"
                >
                    {move || {
                        value
                            .with(|value| {
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
                            })
                    }}

                    <option value=kind_to_str(&data::ValueKind::Number)>"Number"</option>
                    <option value=kind_to_str(&data::ValueKind::Quantity)>"Quantity"</option>
                    <option value=kind_to_str(&data::ValueKind::Bool)>"Boolean"</option>
                    <option value=kind_to_str(&data::ValueKind::String)>"String"</option>
                    <option value=kind_to_str(&data::ValueKind::Array)>"List"</option>
                </select>
            }
        }

        #[component]
        fn BoolEditor(value: Signal<Value>, oninput: Callback<data::Value>) -> impl IntoView {
            let checked = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => false,
                    Value::Equal(data::Value::Bool(value)) => *value,
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
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
        fn StringEditor(value: Signal<Value>, oninput: Callback<data::Value>) -> impl IntoView {
            let input_value = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::String(ref value)) => value.clone(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
            };

            let placeholder = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => "(mixed)",
                    Value::Equal(data::Value::String(_)) => "",
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
            };

            view! {
                <input
                    type="text"
                    prop:value=input_value
                    on:input=move |e| oninput(data::Value::String(event_target_value(&e)))
                    placeholder=placeholder
                    class="input-compact"
                />
            }
        }

        #[component]
        fn NumberEditor(value: Signal<Value>, oninput: Callback<data::Value>) -> impl IntoView {
            let input_value = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Number(ref value)) => value.to_string(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
            };

            let placeholder = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => Some("(mixed)".to_string()),
                    Value::Equal(data::Value::Number(_)) => None,
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
            };

            let oninput_text = move |value: String| {
                let Ok(value) = serde_json::from_str(&value) else {
                    return;
                };

                oninput(data::Value::Number(value));
            };

            view! {
                <InputNumber
                    value=Signal::derive(input_value)
                    oninput=Callback::new(oninput_text)
                    placeholder=MaybeProp::derive(placeholder)
                    class="input-compact"
                />
            }
        }

        #[component]
        fn QuantityEditor(value: Signal<Value>, oninput: Callback<data::Value>) -> impl IntoView {
            let (magnitude, set_magnitude) = create_signal({
                value.with_untracked(|value| match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Quantity { ref magnitude, .. }) => {
                        magnitude.to_string()
                    }
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
            });

            let (unit, set_unit) = create_signal({
                value.with_untracked(|value| match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Quantity { ref unit, .. }) => unit.clone(),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
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
                <div class="flex">
                    <InputNumber
                        value=magnitude
                        oninput=oninput_magnitude
                        placeholder="Magnitude"
                        class="input-compact"
                    />
                    <input
                        prop:value=unit
                        minlength="1"
                        on:input=oninput_unit
                        placeholder="Unit"
                        class="input-compact"
                    />
                </div>
            }
        }

        #[component]
        fn ArrayEditor(value: Signal<Value>, oninput: Callback<data::Value>) -> impl IntoView {
            let (error, set_error) = create_signal(None);
            let (input_value, set_input_value) = create_signal(value.with_untracked(|value| {
                match value {
                    Value::EqualKind(_) => "".to_string(),
                    Value::Equal(data::Value::Array(ref value)) => value
                        .iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                }
            }));
            let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

            let placeholder = move || {
                value.with(|value| match value {
                    Value::EqualKind(_) => "(mixed)",
                    Value::Equal(data::Value::Array(_)) => {
                        "Separate values by comma, semicolon, or new line."
                    }
                    Value::MixedKind | Value::Equal(_) => unreachable!(),
                })
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
                                Some(serde_json::from_str::<data::Value>(elm))
                            }
                        })
                        .collect::<serde_json::Result<Vec<_>>>()
                });

                match val {
                    Ok(val) => oninput(data::Value::Array(val)),
                    Err(err) => set_error(Some(err)),
                }
            });

            view! {
                <textarea
                    on:input=move |e| set_input_value(event_target_value(&e))
                    placeholder=placeholder
                    class=(
                        ["border-2", "!border-syre-red-600", "focus:ring-syre-red-600"],
                        move || error.with(|error| error.is_some()),
                    )
                    class="input-compact align-top overflow-auto scrollbar-thin"
                    title="Separate values by comma, semicolon, or new line."
                >
                    {input_value}
                </textarea>
            }
        }
    }
}
