use super::INPUT_DEBOUNCE;
use crate::{
    pages::project::{
        self,
        state::workspace_graph::{ResourceKind, SelectedResource},
    },
    types,
};
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::{ev::MouseEvent, *};
use leptos_icons::Icon;
use metadata::{AddDatum, Editor as Metadata};
use serde::Serialize;
use state::ActiveResources;
use std::{collections::HashMap, path::PathBuf};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use tags::{AddTags, Editor as Tags};

mod state {
    use super::super::common::bulk;
    use crate::pages::project::state::{self, workspace_graph::SelectedResource};
    use leptos::*;
    use std::collections::HashMap;
    use syre_local_database as db;

    #[derive(derive_more::Deref, Clone)]
    pub struct ActiveResources(Memo<Vec<SelectedResource>>);
    impl ActiveResources {
        pub fn new(resources: Memo<Vec<SelectedResource>>) -> Self {
            Self(resources)
        }
    }
}

#[component]
pub fn Editor(resources: Memo<Vec<SelectedResource>>) -> impl IntoView {
    use super::common::bulk;
    use syre_local_database as db;

    assert!(resources.with(|resources| resources.len()) > 1);
    let graph = expect_context::<project::state::Graph>();
    let add_tags_visible = create_rw_signal(false);
    let add_metadatum_visible = create_rw_signal(false);
    let add_analysis_visible = create_rw_signal(false);
    provide_context(ActiveResources::new(resources.clone()));

    let _ = watch(
        move || add_tags_visible(),
        move |add_tags_visible, _, _| {
            if *add_tags_visible {
                add_metadatum_visible.set(false);
                add_analysis_visible.set(false);
            }
        },
        false,
    );

    let _ = watch(
        move || add_metadatum_visible(),
        move |add_metadatum_visible, _, _| {
            if *add_metadatum_visible {
                add_tags_visible.set(false);
                add_analysis_visible.set(false);
            }
        },
        false,
    );

    let show_add_tags = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            add_tags_visible.set(true);
        }
    };

    let show_add_metadatum = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            add_metadatum_visible.set(true);
        }
    };

    let resource_lengths = move || {
        resources.with(|resources| {
            let (containers, assets) = partition_resources(resources);
            (containers.len(), assets.len())
        })
    };

    let resources = move || {
        resources.with(|resources| {
            let (containers, assets) = partition_resources(resources);
            assert!(containers.len() > 0);
            assert!(assets.len() > 0);

            let containers = containers
                .iter()
                .map(|rid| graph.find_by_id(rid).unwrap())
                .collect::<Vec<_>>();

            let assets = assets
                .iter()
                .map(|rid| graph.find_asset_by_id(rid).unwrap())
                .collect::<Vec<_>>();

            (containers, assets)
        })
    };

    let state_kind = create_memo({
        let resources = resources.clone();
        move |_| {
            let (containers, assets) = resources();
            let resources_len = containers.len() + assets.len();
            let mut kinds = Vec::with_capacity(resources_len);
            let container_properties = containers.iter().map(|state| {
                state.properties().with(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };
                    properties.kind().read_only()
                })
            });
            let asset_properties = assets.iter().map(|state| state.kind().read_only());
            // BUG: Using `kind.with` here causes an `Already mutably borowed` error.
            // This occurs becuase the event listener in `workspace` (`handle_event_graph_container_properties`)
            // `set`s the property, which triggering this signal.
            // In the `resources` call above `graph.find_by_id` then needs this same property
            // while it is still borrowed.
            // This causes the editor to be in the wrong state.
            // See also: https://discord.com/channels/1031524867910148188/1031524868883218474/1276132083080626227
            container_properties
                .chain(asset_properties)
                .fold((), |(), kind| {
                    kind.with_untracked(|kind| kinds.push(kind.clone()));
                });

            kinds.sort();
            kinds.dedup();
            match &kinds[..] {
                [kind] => bulk::Value::Equal(kind.clone()),
                _ => bulk::Value::Mixed,
            }
        }
    });

    let state_description = create_memo({
        let resources = resources.clone();
        move |_| {
            let (containers, assets) = resources();
            let resources_len = containers.len() + assets.len();
            let mut descriptions = Vec::with_capacity(resources_len);
            let container_properties = containers.iter().map(|state| {
                state.properties().with(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };
                    properties.description().read_only()
                })
            });
            let asset_properties = assets.iter().map(|state| state.description().read_only());
            // BUG: Using `description.with` here causes an `Already mutably borowed` error.
            // This occurs becuase the event listener in `workspace` (`handle_event_graph_container_properties`)
            // `set`s the property, which triggering this signal.
            // In the `resources` call above `graph.find_by_id` then needs this same property
            // while it is still borrowed.
            // This causes the editor to be in the wrong state.
            // See also: https://discord.com/channels/1031524867910148188/1031524868883218474/1276132083080626227
            container_properties
                .chain(asset_properties)
                .fold((), |(), description| {
                    description
                        .with_untracked(|description| descriptions.push(description.clone()));
                });

            descriptions.sort();
            descriptions.dedup();
            match &descriptions[..] {
                [kind] => bulk::Value::Equal(kind.clone()),
                _ => bulk::Value::Mixed,
            }
        }
    });

    let state_tags = create_memo({
        let resources = resources.clone();
        move |_| {
            let (containers, assets) = resources();
            let resources_len = containers.len() + assets.len();
            let mut tags = Vec::with_capacity(resources_len);
            let container_properties = containers.iter().map(|state| {
                state.properties().with(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };
                    properties.tags().read_only()
                })
            });
            let asset_properties = assets.iter().map(|state| state.tags().read_only());
            // BUG: Using `tags.with` here causes an `Already mutably borowed` error.
            // This occurs becuase the event listener in `workspace` (`handle_event_graph_container_properties`)
            // `set`s the property, which triggering this signal.
            // In the `resources` call above `graph.find_by_id` then needs this same property
            // while it is still borrowed.
            // This causes the editor to be in the wrong state.
            // See also: https://discord.com/channels/1031524867910148188/1031524868883218474/1276132083080626227
            container_properties
                .chain(asset_properties)
                .fold((), |(), tag| {
                    tag.with_untracked(|tag| tags.extend(tag.clone()));
                });

            tags.sort();
            tags.dedup();
            tags
        }
    });

    let state_metadata = create_memo({
        let resources = resources.clone();
        move |_| {
            let (containers, assets) = resources();
            let resources_len = containers.len() + assets.len();
            let mut metadata = HashMap::with_capacity(resources_len);
            let container_properties = containers.iter().map(|state| {
                state.properties().with(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };
                    properties.metadata().read_only()
                })
            });
            let asset_properties = assets.iter().map(|state| state.metadata().read_only());
            // BUG: Using `metadata.with` here causes an `Already mutably borowed` error.
            // This occurs becuase the event listener in `workspace` (`handle_event_graph_container_properties`)
            // `set`s the property, which triggering this signal.
            // In the `resources` call above `graph.find_by_id` then needs this same property
            // while it is still borrowed.
            // This causes the editor to be in the wrong state.
            // See also: https://discord.com/channels/1031524867910148188/1031524868883218474/1276132083080626227
            container_properties
                .chain(asset_properties)
                .fold((), |(), data| {
                    data.with_untracked(|data| {
                        for (key, value) in data.iter() {
                            let md = metadata
                                .entry(key.clone())
                                .or_insert(Vec::with_capacity(assets.len()));

                            if !value.with_untracked(|value| md.contains(value)) {
                                md.push(value.get_untracked());
                            }
                        }
                    });
                });

            let mut metadata = metadata
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
                .collect::<Vec<_>>();

            metadata.sort_by_key(|(key, _)| key.clone());
            metadata
        }
    });

    let metadata_keys = create_memo({
        let resources = resources.clone();
        move |_| {
            let (containers, assets) = resources();
            let resources_len = containers.len() + assets.len();
            let mut keys = Vec::new();
            let container_properties = containers.iter().map(|state| {
                state.properties().with(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };
                    properties.metadata().read_only()
                })
            });
            let asset_properties = assets.iter().map(|state| state.metadata().read_only());
            // BUG: Using `metadata.with` here causes an `Already mutably borowed` error.
            // This occurs becuase the event listener in `workspace` (`handle_event_graph_container_properties`)
            // `set`s the property, which triggering this signal.
            // In the `resources` call above `graph.find_by_id` then needs this same property
            // while it is still borrowed.
            // This causes the editor to be in the wrong state.
            // See also: https://discord.com/channels/1031524867910148188/1031524868883218474/1276132083080626227
            container_properties
                .chain(asset_properties)
                .fold((), |(), data| {
                    data.with_untracked(|data| {
                        let data_keys = data.iter().map(|(key, _)| key.clone());
                        keys.extend(data_keys);
                    });
                });

            keys.sort();
            keys.dedup();
            keys
        }
    });

    view! {
        <div>
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Bulk resources"</h3>
                <span class="text-sm text-secondary-500 dark:text-secondary-400">
                    "Editing "
                    {move || {
                        let (containers, assets) = resource_lengths();
                        let containers = if containers == 1 {
                            format!("1 container")
                        } else {
                            format!("{containers} containers")
                        };
                        let assets = if assets == 1 {
                            format!("1 asset")
                        } else {
                            format!("{assets} assets")
                        };
                        format!("{containers}, {assets}")
                    }}

                </span>
            </div>
            <form on:submit=move |e| e.prevent_default()>
                <div class="px-1 pb-1">
                    <label>
                        <span class="block">"Type"</span>
                        <Kind state=state_kind />
                    </label>
                </div>
                <div class="px-1 pb-1">
                    <label>
                        <span class="block">"Description"</span>
                        <Description state=state_description />
                    </label>
                </div>
                <div class="relative py-4 border-t border-t-secondary-200 dark:border-t-secondary-700">
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
                                        add_tags_visible,
                                    )

                                    class=(
                                        ["hover:bg-secondary-200", "dark:hover:bg-secondary-700"],
                                        move || !add_tags_visible(),
                                    )

                                    class="aspect-square w-full rounded-sm"
                                >
                                    <Icon icon=icondata::AiPlusOutlined />
                                </button>
                            </span>
                        </div>
                        <AddTags visibility=add_tags_visible />
                        <Tags state=state_tags />
                    </label>
                </div>
                <div class="relative py-4 border-t border-t-secondary-200 dark:border-t-secondary-700">
                    <label class="px-1 block">
                        <div class="flex">
                            <span class="grow">"Metadata"</span>
                            <span>
                                <button
                                    on:mousedown=show_add_metadatum
                                    class=(
                                        ["bg-primary-400", "dark:bg-primary-700"],
                                        add_metadatum_visible,
                                    )

                                    class=(
                                        ["hover:bg-secondary-200", "dark:hover:bg-secondary-700"],
                                        move || !add_metadatum_visible(),
                                    )

                                    class="aspect-square w-full rounded-sm"
                                >
                                    <Icon icon=icondata::AiPlusOutlined />
                                </button>
                            </span>
                        </div>
                        <AddDatum keys=metadata_keys visibility=add_metadatum_visible />
                        <Metadata
                            state=state_metadata
                            oncancel_adddatum=move |_| add_metadatum_visible.set(false)
                        />
                    </label>
                </div>
            </form>
        </div>
    }
}

mod kind {
    use super::{
        super::common::bulk::{kind::Editor as KindEditor, Value},
        container_assets, partition_resources, update_properties, ActiveResources, INPUT_DEBOUNCE,
    };
    use crate::{
        pages::project::{properties::mixed_bulk::resources_to_update_args, state},
        types::Messages,
    };
    use leptos::*;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor(state: Memo<Value<Option<String>>>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();

        let oninput = Callback::new(move |input_value: Option<String>| {
            let mut update = PropertiesUpdate::default();
            let _ = update.kind.insert(input_value.clone());
            spawn_local({
                let project = project.rid().get_untracked();
                let resources = resources.clone();
                let graph = graph.clone();
                let messages = messages.clone();
                async move {
                    update_properties(project, resources, update, &graph, messages).await;
                }
            });
        });

        view! { <KindEditor value=state oninput debounce=INPUT_DEBOUNCE /> }
    }
}

mod description {
    use super::{
        super::common::bulk::{description::Editor as DescriptionEditor, Value},
        container_assets, partition_resources, update_properties, ActiveResources, INPUT_DEBOUNCE,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor(state: Memo<Value<Option<String>>>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();
        let oninput = Callback::new(move |input_value: Option<String>| {
            let mut update = PropertiesUpdate::default();
            let _ = update.description.insert(input_value.clone());
            spawn_local({
                let project = project.rid().get_untracked();
                let resources = resources.clone();
                let graph = graph.clone();
                let messages = messages.clone();
                async move { update_properties(project, resources, update, &graph, messages).await }
            });
        });

        view! {
            <DescriptionEditor
                value=state
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
        container_assets, partition_resources, update_properties, ActiveResources,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::TagsAction};

    #[component]
    pub fn Editor(state: Memo<Vec<String>>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();
        let onremove = Callback::new({
            let graph = graph.clone();
            let project = project.clone();
            let resources = resources.clone();
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
                    let resources = resources.clone();
                    let graph = graph.clone();
                    let messages = messages.clone();
                    async move { update_properties(project, resources, update, &graph, messages).await }
                });
            }
        });

        view! { <TagsEditor value=state onremove /> }
    }

    #[component]
    pub fn AddTags(visibility: RwSignal<bool>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();
        let (reset_form, set_reset_form) = create_signal(());
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
                let resources = resources.clone();
                let graph = graph.clone();
                let messages = messages.clone();
                async move { update_properties(project, resources, update, &graph, messages).await }
            });
        });

        view! {
            <DetailPopout title="Add tags" visibility onclose=move |_| set_reset_form(())>
                <AddTagsEditor onadd reset=reset_form class="w-full px-1" />
            </DetailPopout>
        }
    }
}

mod metadata {
    use super::{
        super::common::{
            bulk::{self, metadata::Editor as MetadataEditor},
            metadata::AddDatum as AddDatumEditor,
        },
        container_assets, partition_resources, update_properties, ActiveResources,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::data;
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::MetadataAction};

    #[component]
    pub fn Editor(
        state: Memo<bulk::Metadata>,
        #[prop(into)] oncancel_adddatum: Callback<()>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();
        let onremove = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let resources = resources.clone();
            let messages = messages.clone();
            move |value: String| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    insert: vec![],
                    remove: vec![value.clone()],
                };

                spawn_local({
                    let project = project.rid().get_untracked();
                    let resources = resources.clone();
                    let graph = graph.clone();
                    let messages = messages.clone();
                    async move { update_properties(project, resources, update, &graph, messages).await }
                });
            }
        });

        let onmodify = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let resources = resources.clone();
            let messages = messages.clone();
            move |value: (String, data::Value)| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    insert: vec![value.clone()],
                    remove: vec![],
                };

                spawn_local({
                    let project = project.rid().get_untracked();
                    let resources = resources.clone();
                    let graph = graph.clone();
                    let messages = messages.clone();
                    async move { update_properties(project, resources, update, &graph, messages).await }
                });
            }
        });

        view! { <MetadataEditor value=state onremove onmodify /> }
    }

    #[component]
    pub fn AddDatum(keys: Memo<Vec<String>>, visibility: RwSignal<bool>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let resources = expect_context::<ActiveResources>();
        let (reset_form, set_reset_form) = create_signal(());
        let onadd = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let resources = resources.clone();
            move |value: (String, data::Value)| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    insert: vec![value.clone()],
                    remove: vec![],
                };

                spawn_local({
                    let project = project.rid().get_untracked();
                    let resources = resources.clone();
                    let graph = graph.clone();
                    let messages = messages.clone();
                    async move { update_properties(project, resources, update, &graph, messages).await }
                });
            }
        });

        view! {
            <DetailPopout title="Add metadata" visibility onclose=move |_| set_reset_form(())>
                <AddDatumEditor
                    keys=Signal::derive(keys)
                    onadd
                    reset=reset_form
                    class="w-full px-1"
                />
            </DetailPopout>
        }
    }
}

/// # Returns
/// Results each resources update as (containers, assets).
async fn update_properties(
    project: ResourceId,
    resources: ActiveResources,
    update: lib::command::asset::bulk::PropertiesUpdate,
    graph: &project::state::Graph,
    messages: types::Messages,
) {
    let (containers, asset_ids) =
        resources.with_untracked(|resources| resources_to_update_args(resources, &graph));
    let expected_results_containers = containers.len();
    let expected_results_assets = asset_ids.len();

    match update_properties_invoke(project, containers, asset_ids, update).await {
        Err(err) => {
            tracing::error!(?err);
            todo!();
        }

        Ok((container_results, asset_results)) => {
            assert_eq!(container_results.len(), expected_results_containers);
            assert_eq!(asset_results.len(), expected_results_assets);
            for result in container_results {
                if let Err(err) = result {
                    todo!();
                }
            }
            for result in asset_results {
                if let Err(err) = result {
                    todo!();
                }
            }
        }
    }
}

/// # Returns
/// Results each resources update as (containers, assets).
async fn update_properties_invoke(
    project: ResourceId,
    containers: Vec<PathBuf>,
    assets: Vec<lib::command::asset::bulk::ContainerAssets>,
    update: lib::command::asset::bulk::PropertiesUpdate,
) -> Result<
    (
        Vec<Result<(), lib::command::container::bulk::error::Update>>,
        Vec<Result<(), lib::command::asset::bulk::error::Update>>,
    ),
    lib::command::error::ProjectNotFound,
> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        containers: Vec<PathBuf>,
        assets: Vec<lib::command::asset::bulk::ContainerAssets>,
        // update: lib::command::asset::bulk::PropertiesUpdate,
        update: String, // TODO: Issue with serializing enum with Option. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/5993
    }

    tauri_sys::core::invoke_result(
        "properties_update_bulk",
        Args {
            project,
            containers,
            assets,
            update: serde_json::to_string(&update).unwrap(),
        },
    )
    .await
}

/// Partition resources into (containers, assets).
fn partition_resources<'a>(
    resources: &'a Vec<SelectedResource>,
) -> (Vec<&'a SelectedResource>, Vec<&'a SelectedResource>) {
    resources
        .iter()
        .partition(|resource| match resource.kind() {
            ResourceKind::Container => true,
            ResourceKind::Asset => false,
        })
}

/// Transforms a list of asset [`ResourceId`]s into
/// [`ContainerAssets`](lib::command::asset::bulk::ContainerAssets).
fn container_assets(
    assets: Vec<&ResourceId>,
    graph: &project::state::Graph,
) -> Vec<lib::command::asset::bulk::ContainerAssets> {
    let mut asset_ids = Vec::<(PathBuf, Vec<ResourceId>)>::new();
    for asset in assets {
        let node = graph.find_by_asset_id(asset).unwrap();
        let container = graph.path(&node).unwrap();
        if let Some((container_id, ref mut container_assets)) = asset_ids
            .iter_mut()
            .find(|(container_id, _)| *container_id == container)
        {
            container_assets.push(asset.clone());
        } else {
            asset_ids.push((container, vec![asset.clone()]));
        }
    }

    asset_ids.into_iter().map(|ids| ids.into()).collect()
}

fn resources_to_update_args(
    resources: &Vec<SelectedResource>,
    graph: &project::state::Graph,
) -> (
    Vec<PathBuf>,
    Vec<lib::command::asset::bulk::ContainerAssets>,
) {
    let (containers, assets) = partition_resources(resources);
    let containers = containers
        .iter()
        .map(|container| {
            let node = graph.find_by_id(container.rid()).unwrap();
            graph.path(&node).unwrap()
        })
        .collect();
    let asset_ids = assets.iter().map(|resource| resource.rid()).collect();
    let asset_ids = container_assets(asset_ids, &graph);
    (containers, asset_ids)
}
