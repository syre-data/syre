use super::INPUT_DEBOUNCE;
use crate::{pages::project, types};
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::{ev::MouseEvent, *};
use leptos_icons::Icon;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use state::{ActiveResources, State};
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use tags::{AddTags, Editor as Tags};

mod state {
    use super::super::common::bulk;
    use crate::pages::project::state;
    use leptos::*;
    use std::collections::HashMap;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[derive(Clone, Debug)]
    pub struct State {
        name: bulk::Value<Option<String>>,
        kind: bulk::Value<Option<String>>,
        description: bulk::Value<Option<String>>,

        /// Union of all tags.
        tags: Vec<String>,

        /// Union of all metadata.
        metadata: bulk::Metadata,
    }

    impl State {
        pub fn from_states(states: Vec<state::Asset>) -> Self {
            let mut names = Vec::with_capacity(states.len());
            let mut kinds = Vec::with_capacity(states.len());
            let mut descriptions = Vec::with_capacity(states.len());
            let mut tags = Vec::with_capacity(states.len());
            let mut metadata = HashMap::with_capacity(states.len());
            states
                .iter()
                .map(|asset| {
                    (
                        asset.name().get_untracked(),
                        asset.kind().get_untracked(),
                        asset.description().get_untracked(),
                        asset.tags().get_untracked(),
                        asset.metadata().get_untracked(),
                    )
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
        pub fn name(&self) -> &bulk::Value<Option<String>> {
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
pub fn Editor(assets: Signal<Vec<ResourceId>>) -> impl IntoView {
    assert!(assets.with(|assets| assets.len()) > 1);
    let graph = expect_context::<project::state::Graph>();
    let add_tags_visible = create_rw_signal(false);
    let add_metadatum_visible = create_rw_signal(false);
    let add_analysis_visible = create_rw_signal(false);

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

    let _ = watch(
        move || add_analysis_visible(),
        move |add_analysis_visible, _, _| {
            if *add_analysis_visible {
                add_tags_visible.set(false);
                add_metadatum_visible.set(false);
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

    let show_add_analyses = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            add_analysis_visible.set(true);
        }
    };

    view! {
        <div>
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
                        <Tags />
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
                        <AddDatum visibility=add_metadatum_visible />
                        <Metadata oncancel_adddatum=move |_| add_metadatum_visible.set(false) />
                    </label>
                </div>
            </form>
        </div>
    }
}

mod name {
    use super::{
        super::common::bulk::{kind::Editor as KindEditor, Value},
        container_assets, update_properties, ActiveResources, State, INPUT_DEBOUNCE,
    };
    use crate::{components::form::debounced::InputText, pages::project::state, types::Messages};
    use leptos::*;
    use std::path::PathBuf;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();

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
                            tracing::error!(?err);
                            todo!();
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            for result in asset_results {
                                if let Err(err) = result {
                                    tracing::error!(?err);
                                    todo!();
                                }
                            }
                        }
                    }
                }
            })
        });

        view! {
            <NameEditor
                value=Signal::derive(move || { state.with(|state| { state.name().clone() }) })
                oninput
                debounce=INPUT_DEBOUNCE
            />
        }
    }

    #[component]
    fn NameEditor(
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

mod kind {
    use super::{
        super::common::bulk::{kind::Editor as KindEditor, Value},
        container_assets, update_properties, ActiveResources, State, INPUT_DEBOUNCE,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();

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
                            tracing::error!(?err);
                            todo!();
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            for result in asset_results {
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
            <KindEditor
                value=Signal::derive(move || { state.with(|state| { state.kind().clone() }) })
                oninput
                debounce=INPUT_DEBOUNCE
            />
        }
    }
}

mod description {
    use super::{
        super::common::bulk::{description::Editor as DescriptionEditor, Value},
        container_assets, update_properties, ActiveResources, State, INPUT_DEBOUNCE,
    };
    use crate::{pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::asset::bulk::PropertiesUpdate;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
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
                            tracing::error!(?err);
                            todo!();
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            for result in asset_results {
                                if let Err(err) = result {
                                    tracing::error!(?err);
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
                value=Signal::derive(move || state.with(|state| state.description().clone()))
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
        container_assets, update_properties, ActiveResources, State,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::TagsAction};

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
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
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                for result in asset_results {
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

        view! {
            <TagsEditor
                value=Signal::derive(move || { state.with(|state| { state.tags().clone() }) })
                onremove
            />
        }
    }

    #[component]
    pub fn AddTags(visibility: RwSignal<bool>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
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
                let asset_ids = assets.with_untracked(|assets| container_assets(assets, &graph));
                let expected_results_len = asset_ids.len();
                async move {
                    match update_properties(project, asset_ids, update).await {
                        Err(err) => {
                            tracing::error!(?err);
                            todo!();
                        }

                        Ok(asset_results) => {
                            assert_eq!(asset_results.len(), expected_results_len);
                            let mut all_ok = true;
                            for result in asset_results {
                                if let Err(err) = result {
                                    all_ok = false;
                                    todo!();
                                }
                            }

                            if all_ok {
                                visibility.set(false);
                                set_reset_form(());
                            }
                        }
                    }
                }
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
            bulk::metadata::Editor as MetadataEditor, metadata::AddDatum as AddDatumEditor,
        },
        container_assets, update_properties, ActiveResources, State,
    };
    use crate::{components::DetailPopout, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::data;
    use syre_desktop_lib::command::{asset::bulk::PropertiesUpdate, bulk::MetadataAction};

    #[component]
    pub fn Editor(#[prop(into)] oncancel_adddatum: Callback<()>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let onremove = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let assets = assets.clone();
            move |value: String| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
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
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                for result in asset_results {
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
            let assets = assets.clone();
            move |value: (String, data::Value)| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    insert: vec![value.clone()],
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
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                for result in asset_results {
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

        view! {
            <MetadataEditor
                value=Signal::derive(move || { state.with(|state| { state.metadata().clone() }) })
                onremove
                onmodify
            />
        }
    }

    #[component]
    pub fn AddDatum(visibility: RwSignal<bool>) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let assets = expect_context::<ActiveResources>();
        let state = expect_context::<Signal<State>>();
        let (reset_form, set_reset_form) = create_signal(());
        let onadd = Callback::new({
            let project = project.clone();
            let graph = graph.clone();
            let assets = assets.clone();
            move |value: (String, data::Value)| {
                let mut update = PropertiesUpdate::default();
                update.metadata = MetadataAction {
                    insert: vec![value.clone()],
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
                                tracing::error!(?err);
                                todo!();
                            }

                            Ok(asset_results) => {
                                assert_eq!(asset_results.len(), expected_results_len);
                                let mut all_ok = true;
                                for result in asset_results {
                                    if let Err(err) = result {
                                        all_ok = false;
                                        todo!();
                                    }
                                }

                                if all_ok {
                                    visibility.set(false);
                                    set_reset_form(());
                                }
                            }
                        }
                    }
                });
            }
        });

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
