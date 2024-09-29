use super::{PopoutPortal, INPUT_DEBOUNCE};
use crate::{pages::project, types};
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::{
    ev::{Event, MouseEvent},
    *,
};
use leptos_icons::Icon;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use state::{ActiveResources, State};
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use tags::{AddTags, Editor as Tags};

#[derive(Clone, Copy)]
enum Widget {
    AddTags,
    AddMetadatum,
    AddAnalysisAssociation,
}

mod state {
    use super::super::common::bulk;
    use crate::pages::project::state;
    use leptos::*;
    use std::collections::HashMap;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[derive(Clone, Debug)]
    pub struct State {
        names: Vec<ReadSignal<String>>,
        kinds: Vec<ReadSignal<Option<String>>>,
        descriptions: Vec<ReadSignal<Option<String>>>,
        tags: Vec<ReadSignal<Vec<String>>>,
        metadata: Vec<ReadSignal<state::Metadata>>,
    }

    impl State {
        pub fn from_states(states: Vec<state::graph::Node>) -> Self {
            let mut names = Vec::with_capacity(states.len());
            let mut kinds = Vec::with_capacity(states.len());
            let mut descriptions = Vec::with_capacity(states.len());
            let mut tags = Vec::with_capacity(states.len());
            let mut metadata = Vec::with_capacity(states.len());
            states
                .iter()
                .map(|state| {
                    state.properties().with(|properties| {
                        let db::state::DataResource::Ok(properties) = properties else {
                            panic!("invalid state");
                        };

                        (
                            properties.name().read_only(),
                            properties.kind().read_only(),
                            properties.description().read_only(),
                            properties.tags().read_only(),
                            properties.metadata().read_only(),
                        )
                    })
                })
                .fold((), |(), (name, kind, description, tag, metadatum)| {
                    names.push(name);
                    kinds.push(kind);
                    descriptions.push(description);
                    tags.push(tag);
                    metadata.push(metadatum);
                });

            Self {
                names,
                kinds,
                descriptions,
                tags,
                metadata,
            }
        }
    }

    impl State {
        pub fn name(&self) -> Signal<bulk::Value<String>> {
            Signal::derive({
                let names = self.names.clone();
                move || {
                    let mut values = names.iter().map(|name| name.get()).collect::<Vec<_>>();
                    values.sort();
                    values.dedup();

                    match &values[..] {
                        [value] => bulk::Value::Equal(value.clone()),
                        _ => bulk::Value::Mixed,
                    }
                }
            })
        }

        pub fn kind(&self) -> Signal<bulk::Value<Option<String>>> {
            Signal::derive({
                let kinds = self.kinds.clone();
                move || {
                    let mut values = kinds.iter().map(|kind| kind.get()).collect::<Vec<_>>();
                    values.sort();
                    values.dedup();

                    match &values[..] {
                        [value] => bulk::Value::Equal(value.clone()),
                        _ => bulk::Value::Mixed,
                    }
                }
            })
        }

        pub fn description(&self) -> Signal<bulk::Value<Option<String>>> {
            Signal::derive({
                let descriptions = self.descriptions.clone();
                move || {
                    let mut values = descriptions
                        .iter()
                        .map(|description| description.get())
                        .collect::<Vec<_>>();
                    values.sort();
                    values.dedup();

                    match &values[..] {
                        [value] => bulk::Value::Equal(value.clone()),
                        _ => bulk::Value::Mixed,
                    }
                }
            })
        }

        /// Union of all tags.
        pub fn tags(&self) -> Signal<Vec<String>> {
            Signal::derive({
                let tags = self.tags.clone();
                move || {
                    let mut values = tags.iter().flat_map(|tag| tag.get()).collect::<Vec<_>>();
                    values.sort();
                    values.dedup();
                    values
                }
            })
        }

        /// Union of all metadata.
        pub fn metadata(&self) -> Signal<bulk::Metadata> {
            Signal::derive({
                let metadata = self.metadata.clone();
                move || {
                    let mut values = HashMap::new();
                    for container_md in metadata.iter() {
                        container_md.with(|container_md| {
                            for (key, value) in container_md.iter() {
                                let entry = values.entry(key.clone()).or_insert(vec![]);
                                entry.push(value.get());
                            }
                        })
                    }

                    values
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
                        .collect()
                }
            })
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
    let popout_portal = expect_context::<PopoutPortal>();
    let (widget, set_widget) = create_signal(None);
    let wrapper_node = NodeRef::<html::Div>::new();
    let tags_node = NodeRef::<html::Div>::new();
    let metadata_node = NodeRef::<html::Div>::new();

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

    let show_add_tags = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            let wrapper = wrapper_node.get_untracked().unwrap();
            let base = tags_node.get_untracked().unwrap();
            let portal = popout_portal.get_untracked().unwrap();

            let top = super::detail_popout_top(&portal, &base, &wrapper);
            (*portal)
                .style()
                .set_property("top", &format!("{top}px"))
                .unwrap();

            set_widget.update(|widget| {
                #[allow(unused_must_use)] {
                widget.insert(Widget::AddTags);
                }
            });
        }
    };

    let show_add_metadatum = move |e: MouseEvent| {
        let wrapper = wrapper_node.get_untracked().unwrap();
        let base = metadata_node.get_untracked().unwrap();
        let portal = popout_portal.get_untracked().unwrap();

        let top = super::detail_popout_top(&portal, &base, &wrapper);
        (*portal)
            .style()
            .set_property("top", &format!("{top}px"))
            .unwrap();

        if e.button() == types::MouseButton::Primary {
            set_widget.update(|widget| {
                #[allow(unused_must_use)] {
                widget.insert(Widget::AddMetadatum);
                }
            });
        }
    };

    let scroll = move |_: Event| {
        let wrapper = wrapper_node.get_untracked().unwrap();
        let portal = popout_portal.get_untracked().unwrap();
        let Some(base) = widget.with(|widget| {
            widget.map(|widget| match widget {
                Widget::AddTags => tags_node,
                Widget::AddMetadatum => metadata_node,
                Widget::AddAnalysisAssociation => todo!(),
            })
        }) else {
            return;
        };
        let base = base.get_untracked().unwrap();

        let top = super::detail_popout_top(&portal, &base, &wrapper);
        (*portal)
            .style()
            .set_property("top", &format!("{top}px"))
            .unwrap();
    };

    let on_widget_close = move |_| {
        set_widget.update(|widget| {
            widget.take();
        });
    };

    view! {
        <div
            ref=wrapper_node
            on:scroll=scroll
            class="overflow-y-auto pr-2 h-full scrollbar-thin"
        >
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Bulk containers"</h3>
                <span class="text-sm text-secondary-500 dark:text-secondary-400">
                    "Editing " {move || containers.with(|containers| containers.len())}
                    " containers"
                </span>
            </div>
            <form on:submit=move |e| e.prevent_default()>
                <div class="px-1 pb-1">
                    <label>
                        <span class="block">"Name"</span>
                        <Name />
                    </label>
                </div>
                <div class="px-1 pb-1">
                    <label>
                        <span class="block">"Type"</span>
                        <Kind />
                    </label>
                </div>
                <div class="px-1 pb-1">
                    <label>
                        <span class="block">"Description"</span>
                        <Description />
                    </label>
                </div>
                <div
                    ref=tags_node
                    class="relative py-4 border-t border-t-secondary-200 dark:border-t-secondary-700"
                >
                    <label class="block px-1">
                        <div class="flex">
                            <span class="grow">"Tags"</span>
                            <span>
                                // TODO: Button hover state seems to be triggered by hovering over
                                // parent section.
                                <button
                                    on:mousedown=show_add_tags
                                    class=(
                                        ["bg-primary-400", "dark:bg-primary-700"],
                                        move || {
                                            widget
                                                .with(|widget| {
                                                    widget
                                                        .map_or(false, |widget| matches!(widget, Widget::AddTags))
                                                })
                                        },
                                    )

                                    class=(
                                        ["hover:bg-secondary-200", "dark:hover:bg-secondary-700"],
                                        move || {
                                            widget
                                                .with(|widget| {
                                                    widget
                                                        .map_or(false, |widget| !matches!(widget, Widget::AddTags))
                                                })
                                        },
                                    )

                                    class="aspect-square w-full rounded-sm"
                                >
                                    <Icon icon=icondata::AiPlusOutlined />
                                </button>
                            </span>
                        </div>
                        <Tags />
                    </label>
                </div>
                <div
                    ref=metadata_node
                    class="relative py-4 border-t border-t-secondary-200 dark:border-t-secondary-700"
                >
                    <label class="px-1 block">
                        <div class="flex">
                            <span class="grow">"Metadata"</span>
                            <span>
                                <button
                                    on:mousedown=show_add_metadatum
                                    class=(
                                        ["bg-primary-400", "dark:bg-primary-700"],
                                        move || {
                                            widget
                                                .with(|widget| {
                                                    widget
                                                        .map_or(
                                                            false,
                                                            |widget| matches!(widget, Widget::AddMetadatum),
                                                        )
                                                })
                                        },
                                    )

                                    class=(
                                        ["hover:bg-secondary-200", "dark:hover:bg-secondary-700"],
                                        move || {
                                            widget
                                                .with(|widget| {
                                                    widget
                                                        .map_or(
                                                            false,
                                                            |widget| !matches!(widget, Widget::AddMetadatum),
                                                        )
                                                })
                                        },
                                    )

                                    class="aspect-square w-full rounded-sm"
                                >
                                    <Icon icon=icondata::AiPlusOutlined />
                                </button>
                            </span>
                        </div>
                        <Metadata />
                    </label>
                </div>
            </form>
            <Show
                when=move || widget.with(|widget| widget.is_some()) && popout_portal.get().is_some()
                fallback=|| view! {}
            >
                {move || {
                    let mount = popout_portal.get_untracked().unwrap();
                    let mount = (*mount).clone();
                    view! {
                        <Portal mount>
                            {move || match widget().unwrap() {
                                Widget::AddTags => {
                                    view! { <AddTags onclose=on_widget_close.clone() /> }
                                }
                                Widget::AddMetadatum => {
                                    view! { <AddDatum onclose=on_widget_close.clone() /> }
                                }
                                Widget::AddAnalysisAssociation => todo!(),
                            }}
                        </Portal>
                    }
                }}
            </Show>

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
        let (input_value, set_input_value) = create_signal({
            state.with(|state| match state.name().get() {
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
                                            view! { <ErrRenameIoMessage errors=rename_errors /> },
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
                                        msg.body(view! { <ErrNameCollisionMessage paths /> });
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
            state.with(|state| {
                state.name().with(|state| match state {
                    Value::Mixed => "(mixed)".to_string(),
                    Value::Equal(_) => "(empty)".to_string(),
                })
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
                class="input-compact w-full"
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
                                        <strong>{path.to_string_lossy().to_string()}:</strong>
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
        super::common::bulk::kind::Editor as KindEditor, update_properties, ActiveResources, State,
        INPUT_DEBOUNCE,
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

        let oninput = Callback::new(move |input_value: Option<String>| {
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
        });

        view! { <KindEditor value=state.with(|state| state.kind()) oninput debounce=INPUT_DEBOUNCE /> }
    }
}

mod description {
    use super::{
        super::common::bulk::description::Editor as DescriptionEditor, update_properties,
        ActiveResources, State, INPUT_DEBOUNCE,
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
        let oninput = Callback::new(move |input_value: Option<String>| {
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
        });

        view! {
            <DescriptionEditor
                value=state.with(|state| state.description())
                oninput
                debounce=INPUT_DEBOUNCE
                class="input-compact w-full align-top"
            />
        }
    }
}

mod tags {
    use super::{
        super::common::bulk::tags::{AddTags as AddTagsEditor, Editor as TagsEditor},
        update_properties, ActiveResources, State,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::{bulk::TagsAction, container::bulk::PropertiesUpdate};

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onremove = Callback::new({
            let graph = graph.clone();
            let project = project.clone();
            let containers = containers.clone();
            move |value: String| {
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
        });

        view! { <TagsEditor value=state.with(|state| { state.tags() }) onremove /> }
    }

    #[component]
    pub fn AddTags(#[prop(optional, into)] onclose: Option<Callback<()>>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let (reset_form, set_reset_form) = create_signal(());
        let onadd = Callback::new(move |tags: Vec<String>| {
            if tags.is_empty() {
                return;
            };

            let containers_len = containers.with_untracked(|containers| containers.len());
            let mut update = PropertiesUpdate::default();
            update.tags = TagsAction {
                insert: tags.clone(),
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
                            let mut all_ok = true;
                            for result in container_results {
                                if let Err(err) = result {
                                    all_ok = false;
                                    todo!();
                                }
                            }

                            if all_ok {
                                if let Some(onclose) = onclose {
                                    onclose(());
                                }
                                set_reset_form(());
                            }
                        }
                    }
                }
            });
        });

        let close = move |_| {
            if let Some(onclose) = onclose {
                onclose(());
            }
        };

        view! {
            <DetailPopout title="Add tags" onclose=Callback::new(close)>
                <AddTagsEditor onadd=Callback::new(onadd) reset=reset_form class="w-full px-1" />
            </DetailPopout>
        }
    }
}

mod metadata {
    use super::{
        super::common::{
            bulk::metadata::Editor as MetadataEditor, metadata::AddDatum as AddDatumEditor,
        },
        update_properties, ActiveResources, State,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::data;
    use syre_desktop_lib::command::{bulk::MetadataAction, container::bulk::PropertiesUpdate};

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onremove = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let containers = containers.clone();
            move |value: String| {
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
        });

        let onmodify = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let containers = containers.clone();
            move |value: (String, data::Value)| {
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
        });

        view! { <MetadataEditor value=state.with(|state| { state.metadata() }) onremove onmodify /> }
    }

    #[component]
    pub fn AddDatum(#[prop(optional, into)] onclose: Option<Callback<()>>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let containers = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onadd = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let containers = containers.clone();
            move |value: (String, data::Value)| {
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
                                let mut all_ok = true;
                                for result in container_results {
                                    if let Err(err) = result {
                                        all_ok = false;
                                        todo!();
                                    }
                                }

                                if all_ok {
                                    if let Some(onclose) = onclose {
                                        onclose(());
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        let keys = move || {
            state.with(|state| {
                state.metadata().with(|metadata| {
                    metadata
                        .iter()
                        .map(|(key, _)| key.clone())
                        .collect::<Vec<_>>()
                })
            })
        };

        let close = move |_| {
            if let Some(onclose) = onclose {
                onclose(());
            }
        };

        view! {
            <DetailPopout title="Add metadata" onclose=Callback::new(close)>
                <AddDatumEditor
                    keys=Signal::derive(keys)
                    onadd=Callback::new(onadd)
                    class="w-full px-1"
                />
            </DetailPopout>
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
    lib::command::error::ProjectNotFound,
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
