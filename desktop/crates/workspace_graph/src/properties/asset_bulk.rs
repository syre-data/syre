use super::PopoutPortal;
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::{
    either::either,
    ev::{Event, MouseEvent},
    html,
    portal::Portal,
    prelude::*,
};
use leptos_icons::Icon;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use state::{ActiveResources, State};
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_desktop_ui_lib as ui_lib;
use tags::{AddTags, Editor as Tags};

#[derive(Clone, Copy)]
enum Widget {
    AddTags,
    AddMetadatum,
}

mod state {
    use leptos::prelude::*;
    use std::collections::HashMap;
    use syre_core::types::ResourceId;
    use syre_desktop_editors::common::bulk;
    use syre_desktop_ui_lib as ui_lib;

    #[derive(Clone, Debug)]
    pub struct State {
        names: Vec<ReadSignal<Option<String>>>,
        kinds: Vec<ReadSignal<Option<String>>>,
        descriptions: Vec<ReadSignal<Option<String>>>,
        tags: Vec<ReadSignal<Vec<String>>>,
        metadata: Vec<ReadSignal<ui_lib::state::Metadata>>,
    }

    impl State {
        pub fn from_states(states: Vec<ui_lib::state::Asset>) -> Self {
            let names = states
                .iter()
                .map(|state| state.name().read_only())
                .collect();
            let kinds = states
                .iter()
                .map(|state| state.kind().read_only())
                .collect();
            let descriptions = states
                .iter()
                .map(|state| state.description().read_only())
                .collect();
            let tags = states
                .iter()
                .map(|state| state.tags().read_only())
                .collect();
            let metadata = states
                .iter()
                .map(|state| state.metadata().read_only())
                .collect();

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
        pub fn name(&self) -> Signal<bulk::Value<Option<String>>> {
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
                    tags.iter()
                        .map(|tags| tags.get())
                        .reduce(|intersection, tags| {
                            let mut intersection = intersection.clone();
                            intersection.retain(|current| tags.contains(current));
                            intersection
                        })
                        .unwrap()
                }
            })
        }

        /// Intersection of all metadata.
        pub fn metadata(&self) -> Signal<bulk::Metadata> {
            Signal::derive({
                let states = self.metadata.clone();
                move || {
                    let mut metadata = HashMap::new();
                    states.iter().for_each(|state| {
                        state.with(|data| {
                            data.iter().for_each(|(key, value)| {
                                let entry = metadata.entry(key.clone()).or_insert(vec![]);
                                entry.push(value.read_only());
                            });
                        });
                    });

                    metadata
                        .into_iter()
                        .filter_map(|(key, values)| {
                            if values.len() != states.len() {
                                return None;
                            }

                            Some(bulk::Metadatum::new(key, values))
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
pub fn Editor(assets: Signal<Vec<ResourceId>>) -> impl IntoView {
    assert!(assets.with_untracked(|assets| assets.len()) > 1);
    let graph = expect_context::<ui_lib::state::Graph>();
    let popout_portal = expect_context::<PopoutPortal>();
    let (widget, set_widget) = signal(None);
    let wrapper_node = NodeRef::<html::Div>::new();
    let tags_node = NodeRef::<html::Div>::new();
    let metadata_node = NodeRef::<html::Div>::new();

    provide_context(Signal::derive(move || {
        let states = assets.with(|assets| {
            assets
                .iter()
                .map(|rid| graph.find_asset_by_id(rid).unwrap())
                .collect::<Vec<_>>()
        });

        State::from_states(states)
    }));

    provide_context(ActiveResources::new(assets.clone()));

    let show_add_tags = move |e: MouseEvent| {
        if e.button() == ui_lib::types::MouseButton::Primary {
            let wrapper = wrapper_node.get_untracked().unwrap();
            let base = tags_node.get_untracked().unwrap();
            let portal = popout_portal.get_untracked().unwrap();

            let top = super::detail_popout_top(&portal, &base, &wrapper);
            (*portal)
                .style()
                .set_property("top", &format!("{top}px"))
                .unwrap();

            set_widget.update(|widget| {
                #[allow(unused_must_use)]
                {
                    widget.insert(Widget::AddTags);
                }
            });
        }
    };

    let show_add_metadatum = move |e: MouseEvent| {
        if e.button() == ui_lib::types::MouseButton::Primary {
            let wrapper = wrapper_node.get_untracked().unwrap();
            let base = metadata_node.get_untracked().unwrap();
            let portal = popout_portal.get_untracked().unwrap();

            let top = super::detail_popout_top(&portal, &base, &wrapper);
            (*portal)
                .style()
                .set_property("top", &format!("{top}px"))
                .unwrap();

            set_widget.update(|widget| {
                #[allow(unused_must_use)]
                {
                    widget.insert(Widget::AddMetadatum);
                }
            });
        }
    };

    let scroll = move |_: Event| {
        let wrapper = wrapper_node.get_untracked().unwrap();
        let portal = popout_portal.get_untracked().unwrap();
        let Some(base) = widget.with_untracked(|widget| {
            widget.map(|widget| match widget {
                Widget::AddTags => tags_node,
                Widget::AddMetadatum => metadata_node,
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

    let on_widget_close = move || {
        set_widget.update(|widget| {
            widget.take();
        });
    };

    view! {
        <div
            node_ref=wrapper_node
            on:scroll=scroll
            class="overflow-y-auto pr-2 h-full scrollbar-thin"
        >
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Bulk assets"</h3>
                <span class="text-sm text-secondary-500 dark:text-secondary-400">
                    "Editing " {move || assets.with(|assets| assets.len())} " assets"
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
                    node_ref=tags_node
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

                                    class="aspect-square w-full rounded-xs cursor-pointer"
                                >
                                    <Icon icon=ui_lib::icon::Add />
                                </button>
                            </span>
                        </div>
                        <Tags />
                    </label>
                </div>
                <div
                    node_ref=metadata_node
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

                                    class="aspect-square w-full rounded-xs cursor-pointer"
                                >
                                    <Icon icon=ui_lib::icon::Add />
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
                            {move || {
                                let widget = widget().unwrap();
                                either!(
                                    widget,
                                    Widget::AddTags => view! { <AddTags onclose=on_widget_close.clone() /> },
                                    Widget::AddMetadatum => view! { <AddDatum onclose=on_widget_close.clone() /> },
                                )
                            }}
                        </Portal>
                    }
                }}
            </Show>
        </div>
    }
}

mod name {
    use super::{ActiveResources, State, container_assets, update_properties};
    use leptos::{prelude::*, task::spawn_local};
    use syre_desktop_editors::{common::bulk::Value, types::InputDebounce};
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;
    use syre_desktop_ui_components::form::debounced::InputText;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let input_debounce = expect_context::<InputDebounce>();

        let oninput = Callback::new(move |input_value: Option<String>| {
            let mut update = PropertiesUpdate::default();
            let _ = update.name.insert(input_value.clone());
            spawn_local({
                let project = project.rid().get_untracked();
                let asset_ids = assets.with_untracked(|assets| container_assets(assets, &graph));
                let expected_results_len = asset_ids.len();
                async move {
                    match update_properties(project, asset_ids, update).await {
                        Err(err) => {
                            let msg = ui_lib::message::Builder::error("Could not save properties.");
                            let msg = msg.body(format!("{err:?}"));
                            messages.push_message(msg.build_str());
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            let errors = asset_results
                                .into_iter()
                                .filter_map(|err| err.err())
                                .collect::<Vec<_>>();

                            if !errors.is_empty() {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(super::UpdateErrors { errors });
                                messages.push_message(msg.build());
                            }
                        }
                    }
                }
            })
        });

        view! { <NameEditor value=state.read_untracked().name() oninput debounce=*input_debounce /> }
    }

    #[component]
    fn NameEditor(
        #[prop(into)] value: Signal<Value<Option<String>>>,
        #[prop(into)] oninput: Callback<Option<String>>,
        #[prop(into)] debounce: Signal<f64>,
    ) -> impl IntoView {
        let (processed_value, set_processed_value) = signal({
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

        let oninput_text = Callback::new(move |value: String| {
            let value = value.trim();
            let value = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };

            set_processed_value(value);
        });

        let placeholder = {
            let value = value.clone();
            move || {
                value.with(|value| match value {
                    Value::Mixed => Some("(mixed)".to_string()),
                    Value::Equal(_) => Some("(empty)".to_string()),
                })
            }
        };

        let _ = Effect::watch(
            processed_value,
            move |processed_value, _, _| {
                oninput.run(processed_value.clone());
            },
            false,
        );

        view! {
            <InputText
                value=Signal::derive(input_value)
                oninput=oninput_text
                debounce
                attr:placeholder=MaybeProp::derive(placeholder)
                attr:class="input-compact"
            />
        }
    }
}

mod kind {
    use super::{ActiveResources, State, container_assets, update_properties};
    use leptos::{prelude::*, task::spawn_local};
    use syre_desktop_editors::{common::bulk::kind::Editor as KindEditor, types::InputDebounce};
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let input_debounce = expect_context::<InputDebounce>();

        let oninput = Callback::new(move |input_value: Option<String>| {
            let mut update = PropertiesUpdate::default();
            let _ = update.kind.insert(input_value.clone());
            spawn_local({
                let project = project.rid().get_untracked();
                let asset_ids = assets.with_untracked(|assets| container_assets(assets, &graph));
                let expected_results_len = asset_ids.len();
                async move {
                    match update_properties(project, asset_ids, update).await {
                        Err(err) => {
                            let msg = ui_lib::message::Builder::error("Could not save properties.");
                            let msg = msg.body(format!("{err:?}"));
                            messages.push_message(msg.build_str());
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            let errors = asset_results
                                .into_iter()
                                .filter_map(|err| err.err())
                                .collect::<Vec<_>>();

                            if !errors.is_empty() {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(super::UpdateErrors { errors });
                                messages.push_message(msg.build());
                            }
                        }
                    }
                }
            });
        });

        view! { <KindEditor value=state.read_untracked().kind() oninput debounce=*input_debounce /> }
    }
}

mod description {
    use super::{ActiveResources, State, container_assets, update_properties};
    use leptos::{prelude::*, task::spawn_local};
    use syre_desktop_editors::common::bulk::description::Editor as DescriptionEditor;
    use syre_desktop_editors::types::InputDebounce;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let input_debounce = expect_context::<InputDebounce>();

        let oninput = Callback::new(move |input_value: Option<String>| {
            let mut update = PropertiesUpdate::default();
            let _ = update.description.insert(input_value.clone());
            spawn_local({
                let project = project.rid().get_untracked();
                let asset_ids = assets.with_untracked(|assets| container_assets(assets, &graph));
                let expected_results_len = asset_ids.len();
                async move {
                    match update_properties(project, asset_ids, update).await {
                        Err(err) => {
                            let msg = ui_lib::message::Builder::error("Could not save properties.");
                            let msg = msg.body(format!("{err:?}"));
                            messages.push_message(msg.build_str());
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            let errors = asset_results
                                .into_iter()
                                .filter_map(|err| err.err())
                                .collect::<Vec<_>>();

                            if !errors.is_empty() {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(super::UpdateErrors { errors });
                                messages.push_message(msg.build());
                            }
                        }
                    }
                }
            });
        });

        view! {
            <DescriptionEditor
                value=state.read_untracked().description()
                oninput
                debounce=*input_debounce
                class="input-compact w-full align-top"
            />
        }
    }
}

mod tags {
    use super::{ActiveResources, State, container_assets, update_properties};
    use leptos::{prelude::*, task::spawn_local};
    use syre_desktop_editors::common::bulk::tags::{
        AddTags as AddTagsEditor, Editor as TagsEditor,
    };
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::TagsAction};
    use syre_desktop_ui_components::DetailPopout;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onremove = Callback::new({
            let graph = graph.clone();
            let project = project.clone();
            let assets = assets.clone();
            move |value: String| {
                if value.is_empty() {
                    return;
                };

                let mut update = PropertiesUpdate::default();
                update.tags = TagsAction {
                    insert: vec![],
                    remove: vec![value.clone()],
                };
                spawn_local({
                    let project = project.rid().get_untracked();
                    let asset_ids =
                        assets.with_untracked(|assets| container_assets(assets, &graph));
                    let expected_results_len = asset_ids.len();
                    async move {
                        match update_properties(project, asset_ids, update).await {
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                let errors = asset_results
                                    .into_iter()
                                    .filter_map(|err| err.err())
                                    .collect::<Vec<_>>();

                                if !errors.is_empty() {
                                    let msg = ui_lib::message::Builder::error(
                                        "Could not save properties.",
                                    );
                                    let msg = msg.body(super::UpdateErrors { errors });
                                    messages.push_message(msg.build());
                                }
                            }
                        }
                    }
                });
            }
        });

        view! { <TagsEditor value=state.with_untracked(|state| { state.tags() }) onremove /> }
    }

    #[component]
    pub fn AddTags(#[prop(into, optional)] onclose: Option<Callback<()>>) -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let onadd = Callback::new(move |tags: Vec<String>| {
            if tags.is_empty() {
                return;
            };

            let mut update = PropertiesUpdate::default();
            update.tags = TagsAction {
                insert: tags.clone(),
                remove: vec![],
            };
            spawn_local({
                let project = project.rid().get_untracked();
                let asset_ids = assets.with_untracked(|assets| container_assets(assets, &graph));
                let expected_results_len = asset_ids.len();
                async move {
                    match update_properties(project, asset_ids, update).await {
                        Err(err) => {
                            let msg = ui_lib::message::Builder::error("Could not save properties.");
                            let msg = msg.body(format!("{err:?}"));
                            messages.push_message(msg.build_str());
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            let errors = asset_results
                                .into_iter()
                                .filter_map(|err| err.err())
                                .collect::<Vec<_>>();

                            if errors.is_empty() {
                                if let Some(onclose) = onclose {
                                    onclose.run(());
                                }
                            } else {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(super::UpdateErrors { errors });
                                messages.push_message(msg.build());
                            }
                        }
                    }
                }
            });
        });

        let onclose = Callback::new(move |_| {
            if let Some(onclose) = onclose {
                onclose.run(());
            }
        });

        view! {
            <DetailPopout title="Add tags" onclose>
                <AddTagsEditor onadd class="w-full px-1" />
            </DetailPopout>
        }
    }
}

mod metadata {
    use super::{ActiveResources, State, container_assets, update_properties};
    use leptos::{prelude::*, task::spawn_local};
    use syre_core::types::data;
    use syre_desktop_editors::{
        common::{bulk::metadata::Editor as MetadataEditor, metadata::AddDatum as AddDatumEditor},
        types::InputDebounce,
    };
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::MetadataAction};
    use syre_desktop_ui_components::DetailPopout;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let input_debounce = expect_context::<InputDebounce>();
        let (modifications, set_modifications) = signal(vec![]);
        let modifications: Signal<Vec<(String, data::Value)>> =
            leptos_use::signal_debounced(modifications, *input_debounce);

        let onremove = Callback::new({
            let project = project.rid().read_only();
            let graph = graph.clone();
            let assets = assets.clone();
            move |value: String| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    add: vec![],
                    update: vec![],
                    remove: vec![value.clone()],
                };

                spawn_local({
                    let project = project.get_untracked();
                    let asset_ids =
                        assets.with_untracked(|assets| container_assets(assets, &graph));
                    let expected_results_len = asset_ids.len();
                    async move {
                        match update_properties(project, asset_ids, update).await {
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                let errors = asset_results
                                    .into_iter()
                                    .filter_map(|err| err.err())
                                    .collect::<Vec<_>>();

                                if !errors.is_empty() {
                                    let msg = ui_lib::message::Builder::error(
                                        "Could not save properties.",
                                    );
                                    let msg = msg.body(super::UpdateErrors { errors });
                                    messages.push_message(msg.build());
                                }
                            }
                        }
                    }
                });
            }
        });

        let onmodify = Callback::new(move |value: (String, data::Value)| {
            set_modifications.update(|modifications| modifications.push(value));
        });

        let _ = Effect::watch(
            modifications,
            {
                let project = project.rid().read_only();
                let graph = graph.clone();
                let assets = assets.clone();
                move |modifications, _, _| {
                    let mut update = PropertiesUpdate::default();
                    update.metadata = MetadataAction {
                        add: vec![],
                        update: modifications.clone(),
                        remove: vec![],
                    };
                    set_modifications.update_untracked(|modifications| modifications.clear());

                    spawn_local({
                        let project = project.get_untracked();
                        let asset_ids =
                            assets.with_untracked(|assets| container_assets(assets, &graph));
                        let expected_results_len = asset_ids.len();
                        async move {
                            match update_properties(project, asset_ids, update).await {
                                Err(err) => {
                                    let msg = ui_lib::message::Builder::error(
                                        "Could not save properties.",
                                    );
                                    let msg = msg.body(format!("{err:?}"));
                                    messages.push_message(msg.build_str());
                                }

                                Ok(asset_results) => {
                                    assert_eq!(asset_results.len(), expected_results_len);
                                    let errors = asset_results
                                        .into_iter()
                                        .filter_map(|err| err.err())
                                        .collect::<Vec<_>>();

                                    if !errors.is_empty() {
                                        let msg = ui_lib::message::Builder::error(
                                            "Could not save properties.",
                                        );
                                        let msg = msg.body(super::UpdateErrors { errors });
                                        messages.push_message(msg.build());
                                    }
                                }
                            }
                        }
                    });
                }
            },
            false,
        );

        view! { <MetadataEditor value=state.read_untracked().metadata() onremove onmodify /> }
    }

    #[component]
    pub fn AddDatum(#[prop(optional, into)] onclose: Option<Callback<()>>) -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onadd = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let assets = assets.clone();
            move |value: (String, data::Value)| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    add: vec![value.clone()],
                    update: vec![],
                    remove: vec![],
                };

                spawn_local({
                    let project = project.rid().get_untracked();
                    let asset_ids =
                        assets.with_untracked(|assets| container_assets(assets, &graph));
                    let expected_results_len = asset_ids.len();
                    async move {
                        match update_properties(project, asset_ids, update).await {
                            Err(err) => {
                                let msg =
                                    ui_lib::message::Builder::error("Could not save properties.");
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                let errors = asset_results
                                    .into_iter()
                                    .filter_map(|err| err.err())
                                    .collect::<Vec<_>>();

                                if errors.is_empty() {
                                    if let Some(onclose) = onclose {
                                        onclose.run(());
                                    }
                                } else {
                                    let msg = ui_lib::message::Builder::error(
                                        "Could not save properties.",
                                    );
                                    let msg = msg.body(super::UpdateErrors { errors });
                                    messages.push_message(msg.build());
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
                        .map(|datum| datum.key().clone())
                        .collect::<Vec<_>>()
                })
            })
        };

        let onclose = Callback::new(move |_| {
            if let Some(onclose) = onclose {
                onclose.run(());
            }
        });

        view! {
            <DetailPopout title="Add metadata" onclose>
                <AddDatumEditor keys=Signal::derive(keys) onadd class="w-full px-1" />
            </DetailPopout>
        }
    }
}

/// # Returns
/// Result of each Asset's update.
async fn update_properties(
    project: ResourceId,
    assets: Vec<lib::command::asset::bulk::ContainerAssets>,
    update: lib::command::asset::bulk::PropertiesUpdate,
) -> Result<
    Vec<Result<(), lib::command::asset::bulk::error::Update>>,
    lib::command::error::ProjectNotFound,
> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        assets: Vec<lib::command::asset::bulk::ContainerAssets>,
        // update: lib::command::asset::bulk::PropertiesUpdate,
        update: String, // TODO: Issue with serializing enum with Option. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/5993
    }

    tauri_sys::core::invoke_result(
        "asset_properties_update_bulk",
        Args {
            project,
            assets,
            update: serde_json::to_string(&update).unwrap(),
        },
    )
    .await
}

/// Transforms a list of asset [`ResourceId`]s into
/// [`ContainerAssets`](lib::command::asset::bulk::ContainerAssets).
fn container_assets(
    assets: &Vec<ResourceId>,
    graph: &ui_lib::state::Graph,
) -> Vec<lib::command::asset::bulk::ContainerAssets> {
    let mut asset_ids = Vec::<(PathBuf, Vec<ResourceId>)>::new();
    for asset in assets {
        let node = graph.find_by_asset_id(asset).unwrap();
        let container = graph.path(&node).unwrap();
        if let Some(container_assets) = asset_ids
            .iter_mut()
            .find(|(container_id, _)| *container_id == container).map(|(_, container_assets)| container_assets)
        {
            container_assets.push(asset.clone());
        } else {
            asset_ids.push((container, vec![asset.clone()]));
        }
    }

    asset_ids.into_iter().map(|ids| ids.into()).collect()
}

#[derive(Clone)]
struct UpdateErrors {
    errors: Vec<lib::command::asset::bulk::error::Update>,
}
impl ui_lib::message::AsAnyView for UpdateErrors {
    fn as_any_view(&self) -> AnyView {
        super::errors_to_list_view(self.errors.clone())
    }
}
