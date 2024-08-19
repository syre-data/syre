use super::INPUT_DEBOUNCE;
use crate::{pages::project, types};
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::{ev::MouseEvent, *};
use leptos_icons::Icon;
use metadata::Editor as Metadata;
use name::Editor as Name;
use serde::Serialize;
use state::{ActiveResources, State};
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use tags::Editor as Tags;

mod state {
    use super::super::common::bulk;
    use crate::pages::project::state;
    use leptos::*;
    use std::collections::HashMap;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[derive(Clone, Debug)]
    pub struct State {
        name: bulk::Value<String>,
        kind: bulk::Value<Option<String>>,
        description: bulk::Value<Option<String>>,

        /// Union of all tags.
        tags: Vec<String>,

        /// Union of all metadata.
        metadata: bulk::Metadata,
    }

    impl State {
        pub fn from_states(states: Vec<state::graph::Node>) -> Self {
            let mut names = Vec::with_capacity(states.len());
            let mut kinds = Vec::with_capacity(states.len());
            let mut descriptions = Vec::with_capacity(states.len());
            let mut tags = Vec::with_capacity(states.len());
            let mut metadata = HashMap::with_capacity(states.len());
            states
                .iter()
                .map(|state| {
                    state.properties().with(|properties| {
                        let db::state::DataResource::Ok(properties) = properties else {
                            panic!("invalid state");
                        };

                        (
                            properties.name().get_untracked(),
                            properties.kind().get_untracked(),
                            properties.description().get_untracked(),
                            properties.tags().get_untracked(),
                            properties.metadata().get_untracked(),
                        )
                    })
                })
                .fold((), |(), (name, kind, description, tag, metadatum)| {
                    names.push(name);
                    kinds.push(kind);
                    descriptions.push(description);
                    tags.extend(tag);

                    for (key, value) in metadatum {
                        let md = metadata
                            .entry(key)
                            .or_insert(Vec::with_capacity(states.len()));

                        if !value.with_untracked(|value| md.contains(value)) {
                            md.push(value.get_untracked());
                        }
                    }
                });

            names.sort();
            names.dedup();
            kinds.sort();
            kinds.dedup();
            descriptions.sort();
            descriptions.dedup();
            tags.sort();
            tags.dedup();

            let name = match &names[..] {
                [name] => bulk::Value::Equal(name.clone()),
                _ => bulk::Value::Mixed,
            };

            let kind = match &kinds[..] {
                [kind] => bulk::Value::Equal(kind.clone()),
                _ => bulk::Value::Mixed,
            };

            let description = match &descriptions[..] {
                [description] => bulk::Value::Equal(description.clone()),
                _ => bulk::Value::Mixed,
            };

            let metadata = metadata
                .into_iter()
                .map(|(key, values)| {
                    let value = if values.iter().all(|value| *value == values[0]) {
                        bulk::metadata::Value::Equal(values[0].clone())
                    } else if values.iter().all(|value| value.kind() == values[0].kind()) {
                        bulk::metadata::Value::EqualKind(values[0].kind())
                    } else {
                        bulk::metadata::Value::MixedKind
                    };

                    (key, value)
                })
                .collect();

            Self {
                name,
                kind,
                description,
                tags,
                metadata,
            }
        }
    }

    impl State {
        pub fn name(&self) -> &bulk::Value<String> {
            &self.name
        }

        pub fn kind(&self) -> &bulk::Value<Option<String>> {
            &self.kind
        }

        pub fn description(&self) -> &bulk::Value<Option<String>> {
            &self.description
        }

        pub fn tags(&self) -> &Vec<String> {
            &self.tags
        }

        pub fn metadata(&self) -> &bulk::Metadata {
            &self.metadata
        }
    }

    #[derive(derive_more::Deref, Clone)]
    pub struct ActiveResources(Signal<Vec<ResourceId>>);
    impl ActiveResources {
        pub fn new(resources: Signal<Vec<ResourceId>>) -> Self {
            Self(resources)
        }
    }
}

#[component]
pub fn Editor(containers: Signal<Vec<ResourceId>>) -> impl IntoView {
    assert!(containers.with(|containers| containers.len()) > 1);
    let graph = expect_context::<project::state::Graph>();
    let (add_metadatum_visible, set_add_metadatum_visible) = create_signal(false);

    provide_context(Signal::derive(move || {
        let states = containers.with(|containers| {
            containers
                .iter()
                .map(|rid| graph.find_by_id(rid).unwrap())
                .collect::<Vec<_>>()
        });

        State::from_states(states)
    }));

    provide_context(ActiveResources::new(containers.clone()));

    let show_add_metadatum = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary as i16 {
            set_add_metadatum_visible(true);
        }
    };

    let add_metadatum_class = move || {
        if add_metadatum_visible() {
            None
        } else {
            Some("hidden".to_string())
        }
    };

    view! {
        <div>
            <div>
                <h3>"Bulk containers"</h3>
                <small>
                    "Editing " {move || containers.with(|containers| containers.len())}
                    " containers"
                </small>
            </div>
            <form on:submit=move |e| e.prevent_default()>
                <div>
                    <label>"Name" <Name/></label>
                </div>
                <div>
                    <label>"Type" <Kind/></label>
                </div>
                <div>
                    <label>"Description" <Description/></label>
                </div>
                <div>
                    <label>"Tags" <Tags/></label>
                </div>
                <div>
                    <label>
                        <div class="flex">
                            <span class="grow">"Metadata"</span>
                            <span>
                                <button
                                    on:mousedown=show_add_metadatum
                                    class:invisible=add_metadatum_visible
                                    class="aspect-square h-full"
                                >
                                    <Icon icon=icondata::AiPlusOutlined/>
                                </button>
                            </span>
                        </div>
                        <Metadata oncancel_adddatum=move |_| set_add_metadatum_visible(false)/>
                    </label>
                </div>
            </form>
        </div>
    }
}

mod name {
    use super::{super::common::bulk::Value, ActiveResources, State, INPUT_DEBOUNCE};
    use crate::{components::message, pages::project::state, types::Messages};
    use leptos::*;
    use serde::Serialize;
    use std::{ffi::OsString, path::PathBuf};
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_local_database as db;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let (input_error, set_input_error) = create_signal(false);
        // TODO: This signal with the watch is a work around to allow
        // `containers` signal in the callback function.
        // See https://github.com/leptos-rs/leptos/issues/2041.
        let (input_value, set_input_value) = create_signal({
            state.with(|state| match state.name() {
                Value::Mixed => String::new(),
                Value::Equal(value) => value.clone(),
            })
        });
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        let _ = watch(
            input_value,
            move |input_value, _, _| {
                set_input_error(false);
                spawn_local({
                    let project = project.rid().get_untracked();
                    let containers = containers.with_untracked(|containers| {
                        containers
                            .iter()
                            .map(|container| {
                                let node = graph.find_by_id(container).unwrap();
                                graph.path(&node).unwrap()
                            })
                            .collect::<Vec<_>>()
                    });

                    let messages = messages.clone();
                    let input_value = input_value.clone();
                    async move {
                        match rename_containers(project, containers.clone(), input_value.clone())
                            .await
                        {
                            Ok(rename_results) => {
                                assert_eq!(containers.len(), rename_results.len());
                                let rename_errors = rename_results
                                    .into_iter()
                                    .enumerate()
                                    .filter_map(|(idx, result)| {
                                        if let Err(err) = result {
                                            Some((containers[idx].clone(), err))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                if rename_errors.len() > 0 {
                                    messages.update(|messages| {
                                        let mut msg = message::Builder::error(
                                            "An error ocurred when renaming container folders.",
                                        );
                                        msg.body(
                                            view! { <ErrRenameIoMessage errors=rename_errors/> },
                                        );
                                        messages.push(msg.build());
                                    });
                                }
                            }
                            Err(err) => match err {
                                lib::command::container::bulk::error::Rename::ProjectNotFound => {
                                    todo!()
                                }
                                lib::command::container::bulk::error::Rename::NameCollision(
                                    paths,
                                ) => {
                                    set_input_error(true);
                                    messages.update(|messages| {
                                        let mut msg =
                                            message::Builder::error("Could not rename containers");
                                        msg.body(view! { <ErrNameCollisionMessage paths/> });
                                        messages.push(msg.build());
                                    });
                                }
                            },
                        }
                    }
                });
            },
            false,
        );

        let placeholder = move || {
            state.with(|state| match state.name() {
                Value::Mixed => "(mixed)".to_string(),
                Value::Equal(_) => "(empty)".to_string(),
            })
        };

        view! {
            <input
                type="text"
                prop:value=Signal::derive(input_value)
                on:input=move |e| {
                    set_input_value(event_target_value(&e));
                }

                debounce=INPUT_DEBOUNCE
                placeholder=placeholder
                minlength="1"
                class=(["border-red-600", "border-solid", "border-2"], input_error)
            />
        }
    }

    async fn rename_containers(
        project: ResourceId,
        containers: Vec<PathBuf>,
        name: impl Into<OsString>,
    ) -> Result<
        Vec<Result<(), lib::command::error::IoErrorKind>>,
        lib::command::container::bulk::error::Rename,
    > {
        #[derive(Serialize)]
        struct RenameContainerArgs {
            project: ResourceId,
            containers: Vec<PathBuf>,
            #[serde(with = "db::serde_os_string")]
            name: OsString,
        }

        tauri_sys::core::invoke_result(
            "container_rename_bulk",
            RenameContainerArgs {
                project,
                containers,
                name: name.into(),
            },
        )
        .await
    }

    #[component]
    fn ErrNameCollisionMessage(paths: Vec<PathBuf>) -> impl IntoView {
        view! {
            <div>
                <p>
                    "The paths"
                    <ul>
                        {paths
                            .iter()
                            .map(|path| {
                                view! { <li>{path.to_string_lossy().to_string()}</li> }
                            })
                            .collect::<Vec<_>>()}

                    </ul> "resulted in a name collistion."
                </p>
                <p>"No containers were renamed."</p>
            </div>
        }
    }

    #[component]
    fn ErrRenameIoMessage(
        errors: Vec<(PathBuf, lib::command::error::IoErrorKind)>,
    ) -> impl IntoView {
        view! {
            <div>
                <p>
                    <ul>
                        {errors
                            .iter()
                            .map(|(path, err)| {
                                view! {
                                    <li>
                                        <strong>{path.to_string_lossy().to_string()} :</strong>
                                        {format!("{err:?}")}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}

                    </ul>
                </p>
                <p>"All other containers were renamed."</p>
            </div>
        }
    }
}

mod kind {
    use super::{
        super::common::bulk::{kind::Editor as KindEditor, Value},
        update_properties, ActiveResources, State, INPUT_DEBOUNCE,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::container::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        // TODO: This signal with the watch is a work around to allow
        // `containers` signal in the callback function.
        // See https://github.com/leptos-rs/leptos/issues/2041.
        let (input_value, set_input_value) = create_signal({
            state.with(|state| match state.kind() {
                Value::Mixed | Value::Equal(None) => None,
                Value::Equal(Some(value)) => Some(value.clone()),
            })
        });

        let _ = watch(
            input_value,
            move |input_value, _, _| {
                let containers_len = containers.with_untracked(|containers| containers.len());
                let mut update = PropertiesUpdate::default();
                let _ = update.kind.insert(input_value.clone());
                spawn_local({
                    let project = project.rid().get_untracked();
                    let containers = containers.with_untracked(|containers| {
                        containers
                            .iter()
                            .map(|container| {
                                let node = graph.find_by_id(container).unwrap();
                                graph.path(&node).unwrap()
                            })
                            .collect::<Vec<_>>()
                    });

                    async move {
                        match update_properties(project, containers, update).await {
                            Err(err) => {
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(container_results) => {
                                assert_eq!(container_results.len(), containers_len);
                                for result in container_results {
                                    if let Err(err) = result {
                                        todo!();
                                    }
                                }
                            }
                        }
                    }
                });
            },
            false,
        );

        view! {
            <KindEditor
                value=Signal::derive(move || { state.with(|state| { state.kind().clone() }) })
                oninput=move |value| {
                    set_input_value(value);
                }

                debounce=INPUT_DEBOUNCE
            />
        }
    }
}

mod description {
    use super::{
        super::common::bulk::{description::Editor as DescriptionEditor, Value},
        update_properties, ActiveResources, State, INPUT_DEBOUNCE,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::container::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        // TODO: This signal with the watch is a work around to allow
        // `containers` signal in the callback function.
        // See https://github.com/leptos-rs/leptos/issues/2041.
        let (input_value, set_input_value) = create_signal({
            state.with(|state| match state.description() {
                Value::Mixed | Value::Equal(None) => None,
                Value::Equal(Some(value)) => Some(value.clone()),
            })
        });

        let _ = watch(
            input_value,
            move |input_value, _, _| {
                let containers_len = containers.with_untracked(|containers| containers.len());
                let mut update = PropertiesUpdate::default();
                let _ = update.description.insert(input_value.clone());
                spawn_local({
                    let project = project.rid().get_untracked();
                    let containers = containers.with_untracked(|containers| {
                        containers
                            .iter()
                            .map(|container| {
                                let node = graph.find_by_id(container).unwrap();
                                graph.path(&node).unwrap()
                            })
                            .collect::<Vec<_>>()
                    });

                    async move {
                        match update_properties(project, containers, update).await {
                            Err(err) => {
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(container_results) => {
                                assert_eq!(container_results.len(), containers_len);
                                for result in container_results {
                                    if let Err(err) = result {
                                        todo!();
                                    }
                                }
                            }
                        }
                    }
                });
            },
            false,
        );

        view! {
            <DescriptionEditor
                value=Signal::derive(move || state.with(|state| state.description().clone()))
                oninput=move |value| {
                    set_input_value(value);
                }

                debounce=INPUT_DEBOUNCE
            />
        }
    }
}

mod tags {
    use super::{
        super::common::bulk::tags::Editor as TagsEditor, update_properties, ActiveResources, State,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::container::bulk::{PropertiesUpdate, TagsAction};

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        // TODO: These signals with their `watch`s is a work around to allow
        // `containers` signal in the callback function.
        // See https://github.com/leptos-rs/leptos/issues/2041.
        let (add_tags_value, set_add_tags_value) = create_signal(vec![]);
        let (remove_tag_value, set_remove_tag_value) = create_signal("".to_string());

        let _ = watch(
            add_tags_value,
            {
                let graph = graph.clone();
                let project = project.clone();
                let containers = containers.clone();
                move |value, _, _| {
                    if value.is_empty() {
                        return;
                    };

                    let containers_len = containers.with_untracked(|containers| containers.len());
                    let mut update = PropertiesUpdate::default();
                    update.tags = TagsAction {
                        insert: value.clone(),
                        remove: vec![],
                    };
                    spawn_local({
                        let project = project.rid().get_untracked();
                        let containers = containers.with_untracked(|containers| {
                            containers
                                .iter()
                                .map(|container| {
                                    let node = graph.find_by_id(container).unwrap();
                                    graph.path(&node).unwrap()
                                })
                                .collect::<Vec<_>>()
                        });

                        async move {
                            match update_properties(project, containers, update).await {
                                Err(err) => {
                                    tracing::error!(?err);
                                    todo!();
                                }

                                Ok(container_results) => {
                                    assert_eq!(container_results.len(), containers_len);
                                    for result in container_results {
                                        if let Err(err) = result {
                                            todo!();
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            },
            false,
        );

        let _ = watch(
            remove_tag_value,
            {
                let graph = graph.clone();
                let project = project.clone();
                let containers = containers.clone();
                move |value, _, _| {
                    if value.is_empty() {
                        return;
                    };

                    let containers_len = containers.with_untracked(|containers| containers.len());
                    let mut update = PropertiesUpdate::default();
                    update.tags = TagsAction {
                        insert: vec![],
                        remove: vec![value.clone()],
                    };
                    spawn_local({
                        let project = project.rid().get_untracked();
                        let containers = containers.with_untracked(|containers| {
                            containers
                                .iter()
                                .map(|container| {
                                    let node = graph.find_by_id(container).unwrap();
                                    graph.path(&node).unwrap()
                                })
                                .collect::<Vec<_>>()
                        });

                        async move {
                            match update_properties(project, containers, update).await {
                                Err(err) => {
                                    tracing::error!(?err);
                                    todo!();
                                }

                                Ok(container_results) => {
                                    assert_eq!(container_results.len(), containers_len);
                                    for result in container_results {
                                        if let Err(err) = result {
                                            todo!();
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            },
            false,
        );

        view! {
            <TagsEditor
                value=Signal::derive(move || { state.with(|state| { state.tags().clone() }) })
                onadd=move |value| set_add_tags_value(value)
                onremove=move |value| set_remove_tag_value(value)
            />
        }
    }
}

mod metadata {
    use super::{
        super::common::{bulk::metadata::Editor as MetadataEditor, metadata::AddDatum},
        update_properties, ActiveResources, State,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::data;
    use syre_desktop_lib::command::container::bulk::{MetadataAction, PropertiesUpdate};

    #[component]
    pub fn Editor(#[prop(into)] oncancel_adddatum: Callback<()>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        // TODO: These signals with their `watch`s is a work around to allow
        // `containers` signal in the callback function.
        // See https://github.com/leptos-rs/leptos/issues/2041.
        let (remove_key, set_remove_key) = create_signal("".to_string());
        let (modify_value, set_modify_value) = create_signal(("".to_string(), data::Value::Null));

        let _ = watch(
            move || remove_key(),
            {
                let project = project.clone();
                let graph = graph.clone();
                let containers = containers.clone();
                move |value: &String, _, _| {
                    let containers_len = containers.with_untracked(|containers| containers.len());
                    let mut update = PropertiesUpdate::default();
                    update.metadata = MetadataAction {
                        insert: vec![],
                        remove: vec![value.clone()],
                    };

                    spawn_local({
                        let project = project.rid().get_untracked();
                        let containers = containers.with_untracked(|containers| {
                            containers
                                .iter()
                                .map(|container| {
                                    let node = graph.find_by_id(container).unwrap();
                                    graph.path(&node).unwrap()
                                })
                                .collect::<Vec<_>>()
                        });

                        async move {
                            match update_properties(project, containers, update).await {
                                Err(err) => {
                                    tracing::error!(?err);
                                    todo!();
                                }

                                Ok(container_results) => {
                                    assert_eq!(container_results.len(), containers_len);
                                    for result in container_results {
                                        if let Err(err) = result {
                                            todo!();
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            },
            false,
        );

        let _ = watch(
            move || modify_value(),
            {
                let project = project.clone();
                let graph = graph.clone();
                let containers = containers.clone();
                move |value: &(String, data::Value), _, _| {
                    let containers_len = containers.with_untracked(|containers| containers.len());
                    let mut update = PropertiesUpdate::default();
                    update.metadata = MetadataAction {
                        insert: vec![value.clone()],
                        remove: vec![],
                    };

                    spawn_local({
                        let project = project.rid().get_untracked();
                        let containers = containers.with_untracked(|containers| {
                            containers
                                .iter()
                                .map(|container| {
                                    let node = graph.find_by_id(container).unwrap();
                                    graph.path(&node).unwrap()
                                })
                                .collect::<Vec<_>>()
                        });

                        async move {
                            match update_properties(project, containers, update).await {
                                Err(err) => {
                                    tracing::error!(?err);
                                    todo!();
                                }

                                Ok(container_results) => {
                                    assert_eq!(container_results.len(), containers_len);
                                    for result in container_results {
                                        if let Err(err) = result {
                                            todo!();
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            },
            false,
        );

        let keys = move || {
            state.with(|state| {
                state
                    .metadata()
                    .iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>()
            })
        };

        view! {
            <AddDatum keys=Signal::derive(keys) onadd=move |value| set_modify_value(value)/>
            <MetadataEditor
                value=Signal::derive(move || { state.with(|state| { state.metadata().clone() }) })
                onremove=move |value| set_remove_key(value)
                onmodify=move |value| set_modify_value(value)
            />
        }
    }
}

/// # Returns
/// Result of each Container's update.
async fn update_properties(
    project: ResourceId,
    containers: Vec<PathBuf>,
    update: lib::command::container::bulk::PropertiesUpdate,
) -> Result<
    Vec<Result<(), lib::command::container::bulk::error::Update>>,
    lib::command::container::bulk::error::ProjectNotFound,
> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        containers: Vec<PathBuf>,
        // update: lib::command::container::bulk::PropertiesUpdate,
        update: String, // TODO: Issue with serializing enum with Option. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/5993
    }

    tauri_sys::core::invoke_result(
        "container_properties_update_bulk",
        Args {
            project,
            containers,
            update: serde_json::to_string(&update).unwrap(),
        },
    )
    .await
}
