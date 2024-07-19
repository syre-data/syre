use crate::{invoke::invoke_result, pages::project::state};
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::*;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_local_database as db;
use tags::Editor as Tags;

const INPUT_DEBOUNCE: f64 = 350.0;

#[component]
pub fn Editor(container: state::Container) -> impl IntoView {
    let db::state::DataResource::Ok(properties) = container.properties().get_untracked() else {
        panic!("invalid state");
    };

    view! {
        <div>
            <form on:submit=|e| e.prevent_default()>
                <div>
                    <label>
                        "Name"
                        <Name
                            value=properties.name().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Type"
                        <Kind
                            value=properties.kind().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Description"
                        <Description
                            value=properties.description().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Tags"
                        <Tags
                            value=properties.tags().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Metadata"
                        <AddDatum
                            container=properties.rid().read_only()
                            metadata=properties.metadata().read_only()
                        />
                        <Metadata
                            value=properties.metadata().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
            </form>

        </div>
    }
}

mod name {
    use super::INPUT_DEBOUNCE;
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use serde::Serialize;
    use std::{ffi::OsString, path::PathBuf};
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        /// Initial value.
        value: ReadSignal<String>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        #[derive(derive_more::Deref, Clone, Debug)]
        struct ValueState {
            /// Source of the value.
            source: Source,

            #[deref]
            value: String,
        }

        /// Source of current value.
        #[derive(PartialEq, Clone, Debug)]
        enum Source {
            /// Value state.
            State,

            /// User input.
            Input,
        }

        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let (input_value, set_input_value) = create_signal(ValueState {
            source: Source::State,
            value: value(),
        });
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);
        let (error, set_error) = create_signal(false);

        create_effect(move |_| {
            value.with(|value| {
                set_input_value(ValueState {
                    source: Source::State,
                    value: value.clone(),
                });
            })
        });

        create_effect({
            let project = project.clone();
            let graph = graph.clone();
            let container = container.clone();
            let messages = messages.write_only();
            move |_| {
                if input_value.with(|value| value.source == Source::State) {
                    return;
                }

                set_error(false);
                let name = input_value.with(|value| value.value.clone());
                if name.is_empty() {
                    set_error(true);
                    return;
                }

                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();

                let project = project.rid().get_untracked();
                let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = rename_container(project, path, name).await {
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        });

        view! {
            <input
                name="name"
                class=("border-red", error)

                prop:value=move || input_value.with(|value| value.value.clone())
                on:input=move |e| {
                    set_input_value(ValueState {
                        source: Source::Input,
                        value: event_target_value(&e),
                    });
                }
            />
        }
    }

    async fn rename_container(
        project: ResourceId,
        container: impl Into<PathBuf>,
        name: impl Into<OsString>,
    ) -> Result<(), lib::command::container::error::Rename> {
        #[derive(Serialize)]
        struct RenameContainerArgs {
            project: ResourceId,
            container: PathBuf,
            #[serde(with = "db::serde_os_string")]
            name: OsString,
        }

        crate::invoke::invoke_result(
            "container_rename",
            RenameContainerArgs {
                project,
                container: container.into(),
                name: name.into(),
            },
        )
        .await
    }
}

mod kind {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        value: ReadSignal<Option<String>>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let (input_value, set_input_value) = create_signal(value());
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        create_effect(move |_| {
            value.with(|value| {
                input_value.with(|input_value| {
                    if value != input_value {
                        set_input_value(value.clone());
                    }
                })
            })
        });

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            // let messages = messages.write_only();
            move |e| {
                let kind = event_target_value(&e);
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.kind = if kind.is_empty() { None } else { Some(kind) };

                let project = project.rid().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <input prop:value=value on:input=oninput/> }
    }
}

mod description {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        /// Initial value.
        value: ReadSignal<Option<String>>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            // let messages = messages.write_only();
            move |e| {
                let description = event_target_value(&e);
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.description = if description.is_empty() {
                    None
                } else {
                    Some(description)
                };

                let project = project.rid().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! {
            <input
                prop:value=move || value.with(|value| value.clone().unwrap_or(String::new()))
                on:input=oninput
            />
        }
    }
}

mod tags {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        /// Initial value.
        value: ReadSignal<Vec<String>>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            // let messages = messages.write_only();
            move |e| {
                let tags = event_target_value(&e);
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });

                let tags = tags
                    .split(",")
                    .filter_map(|tag| {
                        if tag.trim().is_empty() {
                            None
                        } else {
                            Some(tag.trim().to_string())
                        }
                    })
                    .collect::<Vec<_>>();

                let mut tags_unique = Vec::with_capacity(tags.len());
                for tag in tags {
                    if !tags_unique.contains(&tag) {
                        tags_unique.push(tag);
                    }
                }

                properties.tags = tags_unique;
                let project = project.rid().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <input prop:value=move || value.with(|value| value.join(", ")) on:input=oninput/> }
    }
}

mod metadata {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{
        components::{form::InputNumber, message::Builder as Message},
        pages::project::state,
        types::Messages,
    };
    use leptos::*;
    use std::str::FromStr;
    use syre_core::types::{data::ValueKind, ResourceId, Value};
    use syre_local_database as db;

    #[derive(Clone, derive_more::Deref)]
    struct ActiveContainer(ReadSignal<ResourceId>);

    #[component]
    pub fn Editor(
        /// Initial value.
        value: ReadSignal<state::Metadata>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        provide_context(ActiveContainer(container));

        view! {
            <For each=value key=|(key, _)| key.clone() let:datum>
                <DatumEditor key=datum.0 value=datum.1.read_only()/>
            </For>
        }
    }

    #[component]
    pub fn AddDatum(
        container: ReadSignal<ResourceId>,
        metadata: ReadSignal<state::Metadata>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let (key, set_key) = create_signal("".to_string());
        let key = leptos_use::signal_debounced(key, INPUT_DEBOUNCE);
        let value = create_rw_signal(Value::String("".to_string()));

        let invalid_key = move || {
            key.with(|key| metadata.with(|metadata| metadata.iter().any(|(k, _)| k == key)))
        };

        let add_metadatum = move |_| {
            if metadata.with(|metadata| key.with(|key| metadata.iter().any(|(k, _)| k == key))) {
                return;
            }

            let node = container.with(|rid| graph.find_by_id(rid).unwrap());
            let path = graph.path(&node).unwrap();
            let mut properties = node.properties().with_untracked(|properties| {
                let db::state::DataResource::Ok(properties) = properties else {
                    panic!("invalid state");
                };

                properties.as_properties()
            });

            let mut metadata = metadata.with(|metadata| metadata.as_properties());
            metadata.insert(key(), value());
            properties.metadata = metadata;

            let project = project.rid().get_untracked();
            spawn_local(async move {
                if let Err(err) = update_properties(project, path, properties).await {
                    tracing::error!(?err);
                    todo!()
                }

                set_key.update(|key| key.clear());
                value.set(Value::String(String::new()));
            });
        };

        view! {
            <div>
                <input
                    name="key"
                    class=("error", invalid_key)
                    prop:value=key
                    minlength="1"
                    on:input=move |e| set_key(event_target_value(&e))
                />
                <ValueEditor value/>
                <button on:click=add_metadatum>"Add"</button>
            </div>
        }
    }

    #[component]
    pub fn DatumEditor(key: String, value: ReadSignal<Value>) -> impl IntoView {
        assert!(!key.is_empty());
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let container = expect_context::<ActiveContainer>();
        let messages = expect_context::<Messages>();
        let input_value = create_rw_signal(value.get_untracked());

        // create_effect(move |_| {
        //     input_value.set(value());
        // });

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        // create_effect({
        //     let key = key.clone();
        //     move |container_id| -> ResourceId {
        //         // let messages = messages.write_only();
        //         if container.with(|rid| {
        //             if let Some(container_id) = container_id {
        //                 *rid != container_id
        //             } else {
        //                 false
        //             }
        //         }) {
        //             return container.get();
        //         }

        //         let node = container.with(|rid| graph.find_by_id(rid).unwrap());
        //         let path = graph.path(&node).unwrap();
        //         let mut properties = node.properties().with_untracked(|properties| {
        //             let db::state::DataResource::Ok(properties) = properties else {
        //                 panic!("invalid state");
        //             };

        //             properties.as_properties()
        //         });

        //         properties.metadata.insert(key.clone(), input_value.get());
        //         let project = project.rid().get_untracked();
        //         // let messages = messages.clone();
        //         spawn_local(async move {
        //             if let Err(err) = update_properties(project, path, properties).await {
        //                 tracing::error!(?err);
        //                 let mut msg = Message::error("Could not save container");
        //                 msg.body(format!("{err:?}"));
        //                 // messages.update(|messages| messages.push(msg.build()));
        //             }
        //         });

        //         container.get()
        //     }
        // });

        view! {
            <div>
                <span>{key}</span>
                <ValueEditor value=input_value/>
            </div>
        }
    }

    #[component]
    fn ValueEditor(value: RwSignal<Value>) -> impl IntoView {
        let value_editor = move || {
            value.with(|val| match val {
                Value::Null => unreachable!(),
                // Value::Bool(_) => {
                //     view! { <BoolEditor value/> }
                // }
                Value::String(_) => {
                    view! { <StringEditor value/> }
                }
                // Value::Number(_) => {
                //     view! { <NumberEditor value/> }
                // }
                // Value::Quantity { .. } => {
                //     view! { <QuantityEditor value/> }
                // }
                // Value::Array(_) => {
                //     view! { <ArrayEditor value/> }
                // }
                // Value::Map(_) => {
                //     view! { <MapEditor value/> }
                // }
                _ => todo!(),
            })
        };

        view! {
            <KindSelect value/>
            {value_editor}
        }
    }

    #[component]
    fn KindSelect(value: RwSignal<Value>) -> impl IntoView {
        let change = move |e| {
            let kind = string_to_kind(event_target_value(&e)).unwrap();
            // value.set(convert_value_kind(value.get(), &kind));
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
    fn BoolEditor(value: RwSignal<Value>) -> impl IntoView {
        // view! { <input type="checkbox" value=value.read_only() set_value=value.write_only()/> }
        view! {}
    }

    #[component]
    fn StringEditor(value: RwSignal<Value>) -> impl IntoView {
        let input_value = Signal::derive(move || {
            value.with(|value| {
                let Value::String(s) = value else {
                    panic!("invalid value kind");
                };

                s.clone()
            })
        });

        let oninput = move |val| value.set(Value::String(val));
        // view! { <InputTextDebounced debounce=INPUT_DEBOUNCE value=input_value oninput/> }
        // view! {
        //     <input
        //         type="text"
        //         prop:value=value.read_only()
        //         minlength="1"
        //         on:input=move |e| value.set(event_target_value(&e))
        //         required="required"
        //     />
        // }
        view! {}
    }

    #[component]
    fn NumberEditor(value: RwSignal<Value>) -> impl IntoView {
        let input_value = move || {
            value.with(|value| {
                let Value::Number(n) = value else {
                    panic!("invalid value kind");
                };

                n.as_f64().unwrap()
            })
        };

        let oninput = move |val| {
            value.set(Value::Number(serde_json::Number::from_f64(val).unwrap()));
        };

        // view! { <InputNumber value=Signal::derive(input_value) oninput/> }
        view! {}
    }

    #[component]
    fn QuantityEditor(value: RwSignal<Value>) -> impl IntoView {
        let (input_value, set_input_value) = create_signal(value.with_untracked(|value| {
            let Value::Quantity { magnitude, unit } = value.clone() else {
                panic!("invalid value kind");
            };

            (magnitude, unit)
        }));
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        let input_magnitude = move |value: f64| {
            set_input_value.update(|(magnitude, _unit)| *magnitude = value);
        };

        let input_unit = move |e| {
            let value = event_target_value(&e);
            set_input_value.update(|(_magnitude, unit)| *unit = value);
        };

        create_effect(move |_| {
            let Value::Quantity { magnitude, unit } = value() else {
                panic!("invalid value kind");
            };

            set_input_value((magnitude, unit));
        });

        create_effect(move |_| {
            let (magnitude, unit) = input_value();
            value.set(Value::Quantity { magnitude, unit });
        });

        view! {
            <div>
                // <InputNumber
                // debounce=INPUT_DEBOUNCE
                // value=Signal::derive(move || {
                // input_value.with(|(magnitude, _)| magnitude.clone())
                // })

                // oninput=input_magnitude
                // />

                <input
                    prop:value=Signal::derive(move || input_value.with(|(_, unit)| unit.clone()))
                    minlength=1
                    on:input=input_unit
                />
            </div>
        }
    }

    #[component]
    fn ArrayEditor(value: RwSignal<Value>) -> impl IntoView {
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

            value.set(Value::Array(val));
        });

        view! {
            <textarea on:input=move |e| set_input_value(
                event_target_value(&e),
            )>{input_value}</textarea>
        }
    }

    #[component]
    fn MapEditor(value: RwSignal<Value>) -> impl IntoView {
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
    pub fn convert_value_kind(value: Value, target: &ValueKind) -> Value {
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
    pub fn str_to_number(input: &str) -> Result<Value, ()> {
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

async fn update_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    properties: syre_core::project::ContainerProperties,
) -> Result<(), ()> {
    #[derive(Serialize)]
    struct UpdateContainerPropertiesArgs {
        project: ResourceId,
        container: PathBuf,
        properties: syre_core::project::ContainerProperties,
    }

    invoke_result(
        "container_properties_update",
        UpdateContainerPropertiesArgs {
            project,
            container: container.into(),
            properties,
        },
    )
    .await
}
