use super::{common, INPUT_DEBOUNCE};
use crate::pages::project::state;
use analysis_associations::{AddAssociation, Editor as AnalysisAssociations};
use description::Editor as Description;
use has_id::HasId;
use kind::Editor as Kind;
use leptos::*;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local as local;
use syre_local_database as db;
use tags::Editor as Tags;

#[component]
pub fn Editor(container: state::Container) -> impl IntoView {
    let project = expect_context::<state::Project>();

    let db::state::DataResource::Ok(properties) = container.properties().get_untracked() else {
        panic!("invalid state");
    };

    let db::state::DataResource::Ok(analysis_associations) = container.analyses().get_untracked()
    else {
        panic!("invalid state");
    };

    let available_analyses = move || {
        let db::state::DataResource::Ok(analyses) = project.analyses().get() else {
            return vec![];
        };

        analyses.with(|analyses| {
            analyses
                .iter()
                .filter_map(move |analysis| {
                    if analysis_associations.with(|associations| {
                        !associations.iter().any(|association| {
                            analysis
                                .properties()
                                .with(|properties| association.analysis() == properties.id())
                        })
                    }) {
                        analysis.properties().with(|properties| match properties {
                            local::types::AnalysisKind::Script(script) => {
                                let title = if let Some(name) = script.name.as_ref() {
                                    name.clone()
                                } else {
                                    script.path.to_string_lossy().to_string()
                                };

                                Some(common::analysis_associations::AnalysisInfo::script(
                                    script.rid().clone(),
                                    title,
                                ))
                            }

                            local::types::AnalysisKind::ExcelTemplate(template) => {
                                let title = if let Some(name) = template.name.as_ref() {
                                    name.clone()
                                } else {
                                    template.template.path.to_string_lossy().to_string()
                                };

                                Some(common::analysis_associations::AnalysisInfo::excel_template(
                                    template.rid().clone(),
                                    title,
                                ))
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
    };

    view! {
        <div>
            <div>
                <h3>"Container"</h3>
            </div>
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
                            metadata=properties.metadata().read_only()
                            container=properties.rid().read_only()
                        />
                        <Metadata
                            value=properties.metadata().read_only()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Analyses"
                        <AddAssociation
                            available_analyses=Signal::derive(available_analyses)
                            container=properties.rid().read_only()
                        /> <AnalysisAssociations associations=analysis_associations.read_only()/>
                    </label>
                </div>
            </form>

        </div>
    }
}

mod name {
    use super::INPUT_DEBOUNCE;
    use crate::{
        components::{form::debounced::value, message::Builder as Message},
        pages::project::state,
        types::Messages,
    };
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
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let (input_value, set_input_value) = create_signal(value::State::set_from_state(value()));
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);
        let (error, set_error) = create_signal(false);

        create_effect(move |_| {
            value.with(|value| {
                set_input_value(value::State::set_from_state(value.clone()));
            })
        });

        create_effect({
            let project = project.clone();
            let graph = graph.clone();
            let container = container.clone();
            let messages = messages.write_only();
            move |_| {
                if input_value.with(|value| value.was_set_from_state()) {
                    return;
                }

                set_error(false);
                let name = input_value.with(|value| value.trim().to_string());
                if name.is_empty() {
                    set_error(true);
                    return;
                }

                spawn_local({
                    let project = project.rid().get_untracked();
                    let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                    let path = graph.path(&node).unwrap();
                    let messages = messages.clone();

                    async move {
                        if let Err(err) = rename_container(project, path, name).await {
                            let mut msg = Message::error("Could not save container");
                            msg.body(format!("{err:?}"));
                            messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        });

        view! {
            <input
                name="name"
                class=("border-red", error)
                prop:value=move || input_value.with(|value| value.value().clone())
                minlength="1"
                on:input=move |e| {
                    set_input_value(value::State::set_from_input(event_target_value(&e)));
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
        struct Args {
            project: ResourceId,
            container: PathBuf,
            #[serde(with = "db::serde_os_string")]
            name: OsString,
        }

        tauri_sys::core::invoke_result(
            "container_rename",
            Args {
                project,
                container: container.into(),
                name: name.into(),
            },
        )
        .await
    }
}

mod kind {
    use super::{super::common::kind::Editor as KindEditor, update_properties, INPUT_DEBOUNCE};
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

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = move |value: Option<String>| {
            // let messages = messages.write_only();
            let node = container.with_untracked(|rid| graph.find_by_id(rid).unwrap());
            let mut properties = node.properties().with_untracked(|properties| {
                let db::state::DataResource::Ok(properties) = properties else {
                    panic!("invalid state");
                };

                properties.as_properties()
            });
            properties.kind = value;

            spawn_local({
                let project = project.rid().get_untracked();
                let path = graph.path(&node).unwrap();
                // let messages = messages.clone();

                async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                }
            });
        };

        view! { <KindEditor value oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod description {
    use super::{
        super::common::description::Editor as DescriptionEditor, update_properties, INPUT_DEBOUNCE,
    };
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
            move |value: Option<String>| {
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.description = value;

                spawn_local({
                    let project = project.rid().get_untracked();
                    let path = graph.path(&node).unwrap();
                    // let messages = messages.clone();

                    async move {
                        if let Err(err) = update_properties(project, path, properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container");
                            msg.body(format!("{err:?}"));
                            // messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        };

        view! { <DescriptionEditor value oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod tags {
    use super::{super::common::tags::Editor as TagsEditor, update_properties, INPUT_DEBOUNCE};
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
            move |value: Vec<String>| {
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.tags = value;

                spawn_local({
                    let project = project.rid().get_untracked();
                    let path = graph.path(&node).unwrap();
                    // let messages = messages.clone();

                    async move {
                        if let Err(err) = update_properties(project, path, properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container");
                            msg.body(format!("{err:?}"));
                            // messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        };

        view! { <TagsEditor value oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod metadata {
    use super::{
        super::common::metadata::{AddDatum as AddDatumEditor, ValueEditor},
        update_properties, INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::{ResourceId, Value};
    use syre_local_database as db;

    #[derive(Clone, derive_more::Deref)]
    struct ActiveResource(ReadSignal<ResourceId>);

    #[component]
    pub fn Editor(
        /// Initial value.
        value: ReadSignal<state::Metadata>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        provide_context(ActiveResource(container));

        let remove_datum = {
            move |key| {
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.metadata.retain(|k, _| k != &key);

                spawn_local({
                    let project = project.rid().get_untracked();
                    let path = graph.path(&node).unwrap();
                    // let messages = messages.clone();

                    async move {
                        if let Err(err) = update_properties(project, path, properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container");
                            msg.body(format!("{err:?}"));
                            // messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        };

        view! {
            <For each=value key=|(key, _)| key.clone() let:datum>
                <div>
                    <DatumEditor key=datum.0.clone() value=datum.1.read_only()/>
                    <button
                        type="button"
                        on:mousedown={
                            let key = datum.0.clone();
                            let remove_datum = remove_datum.clone();
                            move |_| remove_datum(key.clone())
                        }
                    >

                        "X"
                    </button>
                </div>
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
        let keys = move || {
            metadata.with(|metadata| {
                metadata
                    .iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>()
            })
        };

        let onadd = move |(key, value): (String, Value)| {
            assert!(!key.is_empty());
            assert!(!metadata.with(|metadata| metadata.iter().any(|(k, _)| *k == key)));

            let node = container.with(|rid| graph.find_by_id(rid).unwrap());
            let path = graph.path(&node).unwrap();
            let mut properties = node.properties().with_untracked(|properties| {
                let db::state::DataResource::Ok(properties) = properties else {
                    panic!("invalid state");
                };

                properties.as_properties()
            });

            let mut metadata = metadata.with(|metadata| metadata.as_properties());
            metadata.insert(key, value);
            properties.metadata = metadata;

            spawn_local({
                let project = project.rid().get_untracked();

                async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        todo!()
                    }
                }
            });
        };

        view! { <AddDatumEditor keys=Signal::derive(keys) onadd/> }
    }

    #[component]
    pub fn DatumEditor(key: String, value: ReadSignal<Value>) -> impl IntoView {
        assert!(!key.is_empty());
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let container = expect_context::<ActiveResource>();
        let messages = expect_context::<Messages>();
        let (input_value, set_input_value) = create_signal(value.get_untracked());
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        create_effect(move |_| {
            set_input_value(value());
        });

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        create_effect({
            let key = key.clone();
            move |container_id| -> ResourceId {
                // let messages = messages.write_only();
                if container.with(|rid| {
                    if let Some(container_id) = container_id {
                        *rid != container_id
                    } else {
                        false
                    }
                }) {
                    return container.get();
                }

                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });

                let value = input_value.with(|value| match value {
                    Value::String(value) => Value::String(value.trim().to_string()),
                    Value::Quantity { magnitude, unit } => Value::Quantity {
                        magnitude: magnitude.clone(),
                        unit: unit.trim().to_string(),
                    },
                    Value::Null
                    | Value::Bool(_)
                    | Value::Number(_)
                    | Value::Array(_)
                    | Value::Map(_) => value.clone(),
                });
                properties.metadata.insert(key.clone(), value);

                spawn_local({
                    let project = project.rid().get_untracked();
                    // let messages = messages.clone();

                    async move {
                        if let Err(err) = update_properties(project, path, properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container");
                            msg.body(format!("{err:?}"));
                            // messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });

                // return the current id to track if the container changed
                container.get()
            }
        });

        view! {
            <div>
                <span>{key}</span>
                <ValueEditor value=input_value set_value=set_input_value/>
            </div>
        }
    }
}

mod analysis_associations {
    use super::super::{
        common::analysis_associations::{
            AddAssociation as AddAssociationEditor, AnalysisInfo, Editor as AssociationsEditor,
        },
        state,
    };
    use crate::{components::message::Builder as Message, types::Messages};
    use leptos::*;
    use serde::Serialize;
    use std::path::PathBuf;
    use syre_core::{project::AnalysisAssociation, types::ResourceId};
    use syre_desktop_lib as lib;
    use syre_local_database as db;

    #[component]
    pub fn AddAssociation(
        available_analyses: Signal<Vec<AnalysisInfo>>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();

        let onadd = move |association| {
            #[derive(Serialize)]
            struct Args {
                project: ResourceId,
                container: PathBuf,
                associations: Vec<AnalysisAssociation>,
            }

            let node = container.with(|rid| graph.find_by_id(rid).unwrap());
            let mut associations = node.analyses().with_untracked(|associations| {
                let db::state::DataResource::Ok(associations) = associations else {
                    panic!("invalid state");
                };

                associations.with_untracked(|associations| {
                    associations
                        .iter()
                        .map(|association| association.as_association())
                        .collect::<Vec<_>>()
                })
            });
            associations.push(association);

            spawn_local({
                let project = project.rid().get_untracked();
                let container_path = graph.path(&node).unwrap();

                async move {
                    if let Err(err) = tauri_sys::core::invoke_result::<
                        (),
                        lib::command::container::error::Update,
                    >(
                        "container_analysis_associations_update",
                        Args {
                            project,
                            container: container_path,
                            associations,
                        },
                    )
                    .await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    };
                }
            });
        };

        let oncancel = move |_| {
            tracing::debug!("cancel");
        };

        view! {
            <div>
                <AddAssociationEditor available_analyses onadd oncancel/>
            </div>
        }
    }

    #[component]
    pub fn Editor(
        #[prop(into)] associations: Signal<Vec<state::AnalysisAssociation>>,
    ) -> impl IntoView {
        view! {
            <div>
                <AssociationsEditor associations/>
            </div>
        }
    }
}

async fn update_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    properties: syre_core::project::ContainerProperties,
) -> Result<(), lib::command::container::error::Update> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
        properties: syre_core::project::ContainerProperties,
    }

    tauri_sys::core::invoke_result(
        "container_properties_update",
        Args {
            project,
            container: container.into(),
            properties,
        },
    )
    .await
}
