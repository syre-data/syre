use crate::{components, pages::project, types};
use leptos::{
    either::Either,
    ev::{MouseEvent, SubmitEvent},
    prelude::*,
    task::spawn_local,
};
use leptos_icons::Icon;
use project_bar::ProjectBar;
use state::State;
use std::{ffi::OsString, path::PathBuf};
use syre_core::{self as core, types::ResourceId};
use syre_project_watcher::{self as db, state::Graph};

#[derive(derive_more::Deref, Clone, Copy)]
struct GraphRootName(ReadSignal<OsString>);

#[component]
pub fn Workspace() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <div class="border-b not-dark:border-b-secondary-900">
                <ProjectBar />
            </div>
            <div class="grow min-h-0">
                <DataView />
            </div>
        </div>
    }
}

#[component]
fn DataView() -> impl IntoView {
    let graph = expect_context::<project::state::Graph>();
    provide_context(GraphRootName(graph.root().name().read_only()));
    let state = State::from(graph);

    let toggle_sort = {
        let state = state.clone();
        move |property: &'static str| {
            let state = state.clone();
            move |e: MouseEvent| {
                if e.button() != types::MouseButton::Primary {
                    return;
                }

                let current_field = state.sort().read_untracked().field();
                match property {
                    "path" => {
                        if matches!(current_field, state::SortField::Path) {
                            state.toggle_sort_direction();
                        } else {
                            state.sort_by(state::SortField::Path);
                        }
                    }
                    "file" => {
                        if matches!(current_field, state::SortField::File) {
                            state.toggle_sort_direction();
                        } else {
                            state.sort_by(state::SortField::File);
                        }
                    }
                    "name" => {
                        if matches!(current_field, state::SortField::Name) {
                            state.toggle_sort_direction();
                        } else {
                            state.sort_by(state::SortField::Name);
                        }
                    }
                    "kind" => {
                        if matches!(current_field, state::SortField::Kind) {
                            state.toggle_sort_direction();
                        } else {
                            state.sort_by(state::SortField::Kind);
                        }
                    }
                    field => panic!("invalid field `{field}`"),
                }
            }
        }
    };

    view! {
        <Show
            when={
                let data = state.data();
                move || !data.read().is_empty()
            }
            fallback=NoData
        >
            <div class="overflow-auto w-full h-full">
                <table class="w-full">
                    <thead>
                        <tr>
                            <th
                                scope="col"
                                class="sticky px-2 text-left cursor-pointer"
                                on:mousedown=toggle_sort("path")
                            >
                                <span class="font-primary bold">"Path"</span>
                                <span
                                    class:invisible={
                                        let sort = state.sort();
                                        move || {
                                            !matches!(sort.get().field(), state::SortField::Path)
                                        }
                                    }
                                    class="pl-2 inline-block align-middle"
                                >
                                    {
                                        let sort = state.sort();
                                        move || {
                                            let icon = match sort.get().direction() {
                                                state::SortDirection::Asc => components::icon::CaretDown,
                                                state::SortDirection::Des => components::icon::CaretUp,
                                            };
                                            view! { <Icon icon /> }
                                        }
                                    }
                                </span>
                            </th>
                            <th
                                scope="col"
                                class="sticky px-2 text-left cursor-pointer"
                                on:mousedown=toggle_sort("file")
                            >
                                <span class="font-primary bold">"File"</span>
                                <span
                                    class:invisible={
                                        let sort = state.sort();
                                        move || {
                                            !matches!(sort.get().field(), state::SortField::File)
                                        }
                                    }
                                    class="pl-2 inline-block align-middle"
                                >
                                    {
                                        let sort = state.sort();
                                        move || {
                                            let icon = match sort.get().direction() {
                                                state::SortDirection::Asc => components::icon::CaretDown,
                                                state::SortDirection::Des => components::icon::CaretUp,
                                            };
                                            view! { <Icon icon /> }
                                        }
                                    }
                                </span>
                            </th>
                            <th
                                scope="col"
                                class="sticky px-2 text-left cursor-pointer"
                                on:mousedown=toggle_sort("name")
                            >
                                <span class="font-primary bold">"Name"</span>
                                <span
                                    class:invisible={
                                        let sort = state.sort();
                                        move || {
                                            !matches!(sort.get().field(), state::SortField::Name)
                                        }
                                    }
                                    class="pl-2 inline-block align-middle"
                                >
                                    {
                                        let sort = state.sort();
                                        move || {
                                            let icon = match sort.get().direction() {
                                                state::SortDirection::Asc => components::icon::CaretDown,
                                                state::SortDirection::Des => components::icon::CaretUp,
                                            };
                                            view! { <Icon icon /> }
                                        }
                                    }
                                </span>
                            </th>
                            <th
                                scope="col"
                                class="sticky px-2 text-left cursor-pointer"
                                on:mousedown=toggle_sort("kind")
                            >
                                <span class="font-primary bold">"Type"</span>
                                <span
                                    class:invisible={
                                        let sort = state.sort();
                                        move || {
                                            !matches!(sort.get().field(), state::SortField::Kind)
                                        }
                                    }
                                    class="pl-2 inline-block align-middle"
                                >
                                    {
                                        let sort = state.sort();
                                        move || {
                                            let icon = match sort.get().direction() {
                                                state::SortDirection::Asc => components::icon::CaretDown,
                                                state::SortDirection::Des => components::icon::CaretUp,
                                            };
                                            view! { <Icon icon /> }
                                        }
                                    }
                                </span>
                            </th>
                        </tr>
                    </thead>
                    <tbody class="h-full overflow-y-auto">
                        <For each=state.data() key=|datum| datum.asset().rid().get() let:datum>
                            <DataRow datum />
                        </For>
                    </tbody>
                </table>
            </div>
        </Show>
    }
}

#[component]
fn NoData() -> impl IntoView {
    view! { <div class="pt-2 text-center">"(no data)"</div> }
}

#[component]
fn DataRow(datum: state::Datum) -> impl IntoView {
    let project = expect_context::<project::state::Project>();
    let graph_root_name = expect_context::<GraphRootName>();
    let messages = expect_context::<types::Messages>();
    let asset = datum.asset();

    let update_name = Callback::new({
        let project = project.rid().read_only();
        let container = datum.path();
        let asset = asset.clone();
        move |value: Option<String>| {
            if asset.name().get_untracked() == value {
                return;
            }

            let mut update = asset.as_properties();
            update.name = value;
            spawn_local({
                let path = asset.path().read_only();
                async move {
                    if let Err(err) = update_asset_properties(
                        project.get_untracked(),
                        container.get_untracked(),
                        path.get_untracked(),
                        update,
                    )
                    .await
                    {
                        tracing::error!(?err);
                        let mut msg = types::message::Builder::error("Could not save asset.");
                        msg.body(format!("{err:?}"));
                        messages.update(|messages| messages.push(msg.build()));
                    }
                }
            });
        }
    });

    view! {
        <tr>
            <th scope="row" class="px-2 text-left">
                {
                    let path = datum.path();
                    move || {
                        std::iter::once(std::path::Component::Normal(&graph_root_name.get()))
                            .chain(path.get().components().into_iter().skip(1))
                            .collect::<PathBuf>()
                            .to_string_lossy()
                            .to_string()
                    }
                }
            </th>
            <th scope="row" class="px-2 text-left">
                {
                    let path = asset.path().read_only();
                    move || { path.get().to_string_lossy().to_string() }
                }
            </th>
            <td class="px-2">
                <properties::Name value=asset.name().read_only() on_change=update_name />
            </td>
            <td class="px-2">
                {
                    let kind = asset.kind().read_only();
                    move || { kind.get().unwrap_or("(no type)".to_string()) }
                }
            </td>
        </tr>
    }
}

async fn update_asset_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    asset: impl Into<PathBuf>,
    properties: syre_core::project::AssetProperties,
) -> Result<(), ()> {
    #[derive(serde::Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
        asset: PathBuf,
        // properties: syre_core::project::AssetProperties,
        properties: String, // TODO: Issue with serializing enum with Option. perform manually.
                            // See: https://github.com/tauri-apps/tauri/issues/5993
    }

    tauri_sys::core::invoke_result(
        "asset_properties_update",
        Args {
            project,
            container: container.into(),
            asset: asset.into(),
            properties: serde_json::to_string(&properties).unwrap(),
        },
    )
    .await
}

pub(self) mod properties {
    use super::editor;
    use leptos::prelude::*;

    #[component]
    pub fn Name(
        value: ReadSignal<Option<String>>,
        on_change: Callback<Option<String>>,
    ) -> impl IntoView {
        let input_value = Signal::derive(move || value.get().unwrap_or_default());

        let change = Callback::new(move |value: String| {
            let value = if value.is_empty() { None } else { Some(value) };
            on_change.run(value);
        });

        view! { <editor::Input value=input_value on_change=change empty_value="(no name)".to_string() /> }
    }
}

pub(self) mod editor {
    use crate::{components::icon, types};
    use leptos::{
        either::Either,
        ev::{KeyboardEvent, MouseEvent, SubmitEvent},
        html,
        prelude::*,
    };
    use leptos_icons::Icon;

    const ESCAPE_KEY_CODE: &str = "Escape";

    #[component]
    pub fn Input(
        value: Signal<String>,
        on_change: Callback<String>,

        /// Displayed if `value` is empty and not in editing mode.
        empty_value: String,

        /// `<input>` placeholder.
        #[prop(optional)]
        placeholder: Option<String>,
    ) -> impl IntoView {
        let (editing, set_editing) = signal(false);
        let (input_value, set_input_value) = signal(value.get_untracked());
        let input_node = NodeRef::<html::Input>::new();

        let enable_editing = {
            move |e: MouseEvent| {
                if e.button() != types::MouseButton::Primary {
                    return;
                }

                set_input_value(value.get_untracked());
                set_editing(true);
            }
        };

        let disable_editing = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            set_editing(false);
        };

        let handle_escape = move |e: KeyboardEvent| {
            if e.key() == ESCAPE_KEY_CODE {
                set_editing(false);
            }
        };

        let trigger_change = move || {
            set_editing(false);
            on_change.run(input_value.get_untracked());
        };

        let submit = {
            let trigger_change = trigger_change.clone();
            move |e: SubmitEvent| {
                e.prevent_default();
                trigger_change();
            }
        };

        let change = {
            let trigger_change = trigger_change.clone();
            move |e: MouseEvent| {
                if e.button() != types::MouseButton::Primary {
                    return;
                }

                trigger_change();
            }
        };

        let display_value = {
            let empty_value = empty_value.clone();
            move || {
                if value.read().is_empty() {
                    empty_value.clone()
                } else {
                    value.get()
                }
            }
        };

        Effect::new(move || {
            if editing() {
                if let Some(input_node) = input_node.get() {
                    if let Err(err) = input_node.focus() {
                        tracing::error!(?err);
                    };
                }
            }
        });

        move || {
            if editing.get() {
                Either::Left(view! {
                    <form on:submit=submit class="flex">
                        <input
                            node_ref=input_node
                            bind:value=(input_value, set_input_value)
                            on:keydown=handle_escape
                            class="input-compact grow"
                            placeholder=placeholder.clone()
                        />
                        <div class="pl-2 flex gap-1">
                            <button
                                type="submit"
                                class="cursor-pointer text-lg hover:text-syre-green-700 dark:hover:text-syre-green-400"
                                on:mousedown=change
                            >
                                <Icon icon=icon::Accept />
                            </button>
                            <button
                                type="button"
                                class="cursor-pointer text-lg hover:text-syre-red-700 dark:hover:text-syre-red-600"
                                on:mousedown=disable_editing
                            >
                                <Icon icon=icon::Close />
                            </button>
                        </div>
                    </form>
                })
            } else {
                Either::Right(view! {
                    <div class="flex group">
                        <div class="grow">{display_value.clone()}</div>
                        <div class="pl-2 not-group-hover:invisible">
                            <button class="cursor-pointer" on:mousedown=enable_editing>
                                <Icon icon=icon::Edit />
                            </button>
                        </div>
                    </div>
                })
            }
        }
    }
}

mod project_bar {
    use super::super::super::DataView;
    use crate::{components, pages::project::state, types};
    use leptos::{ev::MouseEvent, prelude::*};
    use leptos_icons::Icon;

    #[component]
    pub fn ProjectBar() -> impl IntoView {
        view! {
            <div class="flex px-2 py-1">
                <div class="w-1/3 inline-flex gap-2"></div>
                <div class="w-1/3 text-center">
                    <ProjectInfo />
                </div>
                <div class="w-1/3 text-right">
                    <Controls />
                </div>
            </div>
        }
    }

    #[component]
    fn ProjectInfo() -> impl IntoView {
        let project = expect_context::<state::Project>();

        view! { <div class="grow text-center font-primary">{project.properties().name()}</div> }
    }

    #[component]
    fn Controls() -> impl IntoView {
        const COMMAND_BUTTON_CLASS: &str = "btn-secondary p-1 rounded-xs cursor-pointer";

        let data_view = expect_context::<RwSignal<DataView>>();

        let toggle_data_view = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            data_view.set(DataView::Graph)
        };

        let refresh = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            let window = web_sys::window().unwrap();
            window.location().reload().unwrap();
        };

        view! {
            <ol class="flex gap-1 justify-end">
                <li>
                    <button
                        on:mousedown=toggle_data_view
                        type="button"
                        class=COMMAND_BUTTON_CLASS
                        title="Toggle data view"
                    >
                        <Icon icon=icondata::ImTree />
                    </button>

                </li>
                <li>
                    <button
                        on:mousedown=refresh
                        type="button"
                        class="btn-secondary p-1 rounded-xs cursor-pointer"
                        title="Refresh"
                    >
                        <Icon icon=components::icon::Refresh />
                    </button>
                </li>
            </ol>
        }
    }
}

mod state {
    use crate::pages::project::state;
    use leptos::prelude::*;
    use std::{assert_matches::assert_matches, path::PathBuf};
    use syre_project_watcher as db;

    #[derive(Clone, Copy, Default, Debug)]
    pub enum SortDirection {
        /// Ascending.
        #[default]
        Asc,
        /// Descending.
        Des,
    }

    #[derive(Clone, Copy, Default, PartialEq, Debug)]
    pub enum SortField {
        #[default]
        Path,
        File,
        Name,
        Kind,
    }

    #[derive(Clone)]
    pub struct Sort {
        field: SortField,
        direction: SortDirection,
    }

    impl Sort {
        pub fn field(&self) -> SortField {
            self.field
        }

        pub fn direction(&self) -> SortDirection {
            self.direction
        }
    }

    impl Default for Sort {
        fn default() -> Self {
            Self {
                field: Default::default(),
                direction: Default::default(),
            }
        }
    }

    #[derive(Clone)]
    pub struct Datum {
        container: RwSignal<state::graph::Node>,
        path: RwSignal<PathBuf>,
        asset: state::Asset,
    }

    impl Datum {
        /// Container path.
        pub fn path(&self) -> ReadSignal<PathBuf> {
            self.path.read_only()
        }

        pub fn asset(&self) -> &state::Asset {
            &self.asset
        }
    }

    #[derive(Clone)]
    pub struct State {
        /// Graph state.
        graph: state::Graph,

        /// Containers' assets state.
        node_states: RwSignal<Vec<ReadSignal<state::container::AssetsState>>>,

        /// Containers' assets' states.
        node_assets: RwSignal<Vec<ReadSignal<Vec<state::Asset>>>>,

        /// Individual assets.
        data: RwSignal<Vec<Datum>>,

        sort: RwSignal<Sort>,
    }

    impl State {
        pub fn from(graph: state::Graph) -> Self {
            let node_states = graph
                .nodes()
                .read_untracked()
                .iter()
                .map(|node| node.assets().read_only())
                .collect::<Vec<_>>();

            let node_assets = node_states
                .iter()
                .filter_map(|state| {
                    if let db::state::DataResource::Ok(assets) = state.get_untracked() {
                        Some(assets.read_only())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let data = graph
                .nodes()
                .read_untracked()
                .iter()
                .map(|node| (node, node.assets().read_only()))
                .filter_map(|(node, assets)| {
                    if let db::state::DataResource::Ok(assets) = assets.get_untracked() {
                        Some((node, assets.read_only()))
                    } else {
                        None
                    }
                })
                .flat_map(|(node, assets)| {
                    let path = graph.path(node).expect(&format!(
                        "container path of `{:?}` should exist",
                        node.name().get_untracked(),
                    ));

                    assets
                        .get_untracked()
                        .into_iter()
                        .map(move |asset| (node.clone(), path.clone(), asset))
                })
                .map(|(node, path, asset)| Datum {
                    container: RwSignal::new(node),
                    path: RwSignal::new(path),
                    asset,
                })
                .collect::<Vec<_>>();

            let node_states = RwSignal::new(node_states);
            let node_assets = RwSignal::new(node_assets);
            let data = RwSignal::new(data);
            let sort = RwSignal::new(Sort::default());

            let _ = Effect::watch(
                graph.nodes(),
                move |graph, _, _| {
                    let update = graph
                        .iter()
                        .map(|node| node.assets().read_only())
                        .collect::<Vec<_>>();

                    node_states.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                node_states,
                move |node_states, _, _| {
                    let update = node_states
                        .iter()
                        .filter_map(|node| {
                            if let db::state::DataResource::Ok(assets) = node.get_untracked() {
                                Some(assets.read_only())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    node_assets.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                node_assets,
                move |node_assets, _, _| {
                    let update = node_assets
                        .iter()
                        .flat_map(|assets| assets.get_untracked())
                        .collect::<Vec<_>>();

                    todo!();
                    // data.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                sort,
                move |sort, prev, _| {
                    let field = sort.field();
                    let needs_sorting = if let Some(prev) = prev {
                        field != prev.field()
                    } else {
                        true
                    };

                    if needs_sorting {
                        assert_matches!(sort.direction(), SortDirection::Asc);
                        match field {
                            SortField::Path => data.write().sort_by_key(|datum| {
                                datum
                                    .path()
                                    .get_untracked()
                                    .to_string_lossy()
                                    .to_lowercase()
                            }),
                            SortField::File => data.write().sort_by_key(|datum| {
                                datum
                                    .asset()
                                    .path()
                                    .get_untracked()
                                    .to_string_lossy()
                                    .to_lowercase()
                            }),
                            SortField::Name => {
                                data.write().sort_by_key(|datum| {
                                    datum
                                        .asset()
                                        .name()
                                        .get_untracked()
                                        .map(|value| value.to_lowercase())
                                });
                            }
                            SortField::Kind => {
                                data.write().sort_by_key(|datum| {
                                    datum
                                        .asset()
                                        .kind()
                                        .get_untracked()
                                        .map(|value| value.to_lowercase())
                                });
                            }
                        }
                    } else {
                        data.write().reverse();
                    }
                },
                false,
            );

            Self {
                graph,
                node_states,
                node_assets,
                data,
                sort,
            }
        }

        pub fn data(&self) -> ReadSignal<Vec<Datum>> {
            self.data.read_only()
        }

        pub fn sort(&self) -> ReadSignal<Sort> {
            self.sort.read_only()
        }

        pub fn sort_by(&self, field: SortField) {
            if field != self.sort.read_untracked().field {
                self.sort.set(Sort {
                    field,
                    direction: SortDirection::default(),
                });
            }
        }

        pub fn toggle_sort_direction(&self) {
            let direction = match self.sort.read_untracked().direction() {
                SortDirection::Asc => SortDirection::Des,
                SortDirection::Des => SortDirection::Asc,
            };

            let mut sort = self.sort().get_untracked();
            sort.direction = direction;
            self.sort.set(sort);
        }
    }
}
