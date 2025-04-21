use crate::{project_bar::ProjectBar, state};
use leptos::{
    either::Either,
    ev::{MouseEvent, SubmitEvent},
    html,
    prelude::*,
    task::spawn_local,
};
use leptos_icons::Icon;
use std::{ffi::OsString, path::PathBuf};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_ui_lib as ui_lib;
use syre_project_watcher::{self as db, state::Graph};
use wasm_bindgen::{JsCast, prelude::Closure};

const MIN_COL_WIDTH: u32 = 100;
const MAX_COL_WIDTH_RATIO: f64 = 0.8;

#[derive(derive_more::Deref, Clone, Copy)]
struct GraphRootName(ReadSignal<OsString>);

#[component]
pub fn Workspace() -> impl IntoView {
    let graph = expect_context::<ui_lib::state::Graph>();
    let state = state::data::State::from(graph);
    let display_state = state::display::State::new();
    provide_context(state);
    provide_context(display_state);

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
    let graph = expect_context::<ui_lib::state::Graph>();
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    provide_context(GraphRootName(graph.root().name().read_only()));

    let table_node = NodeRef::<html::Table>::new();
    let col_node_path = display_state.columns().path().node_ref();
    let col_node_file = display_state.columns().file().node_ref();
    let col_node_name = display_state.columns().name().node_ref();
    let col_node_kind = display_state.columns().kind().node_ref();
    let col_node_description = display_state.columns().description().node_ref();
    let col_node_tags = display_state.columns().tags().node_ref();

    view! {
        <Show
            when={
                let data = state.data();
                move || !data.read().is_empty()
            }
            fallback=NoData
        >
            <div class="overflow-auto scrollbar-thin w-full h-full">
                <table node_ref=table_node class="relative min-w-full">
                    <colgroup>
                        <col
                            node_ref=col_node_path
                            class:collapse={
                                let visible = display_state.columns().path().visible().read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=col_node_file
                            class:collapse={
                                let visible = display_state.columns().file().visible().read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=col_node_name
                            class:collapse={
                                let visible = display_state.columns().name().visible().read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=col_node_kind
                            class:collapse={
                                let visible = display_state.columns().kind().visible().read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=col_node_description
                            class:collapse={
                                let visible = display_state
                                    .columns()
                                    .description()
                                    .visible()
                                    .read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=col_node_tags
                            class:collapse={
                                let visible = display_state.columns().tags().visible().read_only();
                                move || !visible()
                            }
                        />
                    </colgroup>
                    <thead>
                        <tr>
                            <TableHeaderSortable
                                display_name="Path"
                                sort_field=state::data::SortField::Path
                                col_node=col_node_path
                                table_node=table_node
                            />
                            <TableHeaderSortable
                                display_name="File"
                                sort_field=state::data::SortField::File
                                col_node=col_node_file
                                table_node=table_node
                            />
                            <TableHeaderSortable
                                display_name="Name"
                                sort_field=state::data::SortField::Name
                                col_node=col_node_name
                                table_node=table_node
                            />
                            <TableHeaderSortable
                                display_name="Type"
                                sort_field=state::data::SortField::Kind
                                col_node=col_node_kind
                                table_node=table_node
                            />
                            <TableHeader
                                display_name="Description"
                                col_node=col_node_description
                                table_node=table_node
                            />
                            <TableHeader
                                display_name="Tags"
                                col_node=col_node_tags
                                table_node=table_node
                            />
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

enum Col {
    Path,
}

#[component]
fn TableHeaderSortable(
    display_name: &'static str,
    sort_field: state::data::SortField,
    col_node: NodeRef<html::Col>,
    table_node: NodeRef<html::Table>,
) -> impl IntoView {
    let state = expect_context::<state::data::State>();
    let (is_resizing, set_is_resizing) = signal(false);
    let root_node = NodeRef::<html::Th>::new();
    let drag_handle_node = NodeRef::<html::Div>::new();

    let resize_start = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }
        e.stop_propagation();

        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let cursor_style = body.style().get_property_value("cursor").unwrap();
        body.style().set_property("cursor", "col-resize").unwrap();
        set_is_resizing(true);

        let resize_cb = Closure::<dyn Fn(MouseEvent)>::new({
            move |e: MouseEvent| {
                let table_node = table_node.get_untracked().unwrap();
                let root_node = root_node.get_untracked().unwrap();
                let col_node = col_node.get_untracked().unwrap();

                let table_bb = table_node.get_bounding_client_rect();
                let root_bb = root_node.get_bounding_client_rect();
                let col_bb = col_node.get_bounding_client_rect();

                let window_width = web_sys::window()
                    .unwrap()
                    .inner_width()
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let col_width = i32::clamp(
                    e.x() - root_bb.x() as i32,
                    MIN_COL_WIDTH as i32,
                    (window_width * MAX_COL_WIDTH_RATIO) as i32,
                );
                let delta_width = col_width - col_bb.width() as i32;

                let table_width = table_bb.width() as i32 + delta_width;
                (*table_node)
                    .style()
                    .set_property("width", &format!("{table_width}px"))
                    .unwrap();

                // col_node.style("width", &format!("{width}px"));
                (*col_node)
                    .style()
                    .set_property("width", &format!("{col_width}px"))
                    .unwrap();
            }
        });

        document
            .add_event_listener_with_callback("mousemove", resize_cb.as_ref().unchecked_ref())
            .unwrap();

        let resize_end = Closure::<dyn Fn(MouseEvent)>::new({
            let document = document.clone();
            move |_e: MouseEvent| {
                let body = document.body().unwrap();
                body.style()
                    .set_property("cursor", cursor_style.as_str())
                    .unwrap();
                set_is_resizing(false);

                document
                    .remove_event_listener_with_callback(
                        "mousemove",
                        resize_cb.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });

        document
            .add_event_listener_with_callback("mouseup", resize_end.as_ref().unchecked_ref())
            .unwrap();

        resize_end.forget();
    };

    let toggle_sort = {
        let state = state.clone();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            if sort_field == state.sort().read_untracked().field() {
                state.toggle_sort_direction();
            } else {
                state.sort_by(sort_field);
            }
        }
    };

    view! {
        <th
            node_ref=root_node
            scope="col"
            on:mousedown=toggle_sort
            class="sticky top-0 cursor-pointer pl-1 pr-2 pb-1 text-left bg-white dark:bg-secondary-800"
        >
            <div class="inline-flex w-full">
                <div class="grow">
                    <span class="font-primary bold">{display_name}</span>
                    <span
                        class:invisible={
                            let sort = state.sort();
                            move || { sort_field != sort.get().field() }
                        }
                        class="pl-2 inline-block align-middle"
                    >
                        {
                            let sort = state.sort();
                            move || {
                                let icon = match sort.get().direction() {
                                    state::data::SortDirection::Asc => ui_lib::icon::CaretDown,
                                    state::data::SortDirection::Des => ui_lib::icon::CaretUp,
                                };
                                view! { <Icon icon /> }
                            }
                        }
                    </span>
                </div>
                <div
                    node_ref=drag_handle_node
                    on:mousedown=resize_start
                    class="cursor-col-resize hover:bg-primary-700 hover:delay-100 hover:duration-200 hover:w-[4px] transition-width"
                    class=("w-[2px]", move || !is_resizing())
                    class=(["bg-primary-700", "w-[4px]"], is_resizing)
                ></div>
            </div>
        </th>
    }
}

#[component]
fn TableHeader(
    display_name: &'static str,
    col_node: NodeRef<html::Col>,
    table_node: NodeRef<html::Table>,
) -> impl IntoView {
    let state = expect_context::<state::data::State>();
    let (is_resizing, set_is_resizing) = signal(false);
    let root_node = NodeRef::<html::Th>::new();
    let drag_handle_node = NodeRef::<html::Div>::new();

    let resize_start = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }
        e.stop_propagation();

        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let cursor_style = body.style().get_property_value("cursor").unwrap();
        body.style().set_property("cursor", "col-resize").unwrap();
        set_is_resizing(true);

        let resize_cb = Closure::<dyn Fn(MouseEvent)>::new({
            move |e: MouseEvent| {
                let table_node = table_node.get_untracked().unwrap();
                let root_node = root_node.get_untracked().unwrap();
                let col_node = col_node.get_untracked().unwrap();

                let table_bb = table_node.get_bounding_client_rect();
                let root_bb = root_node.get_bounding_client_rect();
                let col_bb = col_node.get_bounding_client_rect();

                let window_width = web_sys::window()
                    .unwrap()
                    .inner_width()
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let col_width = i32::clamp(
                    e.x() - root_bb.x() as i32,
                    MIN_COL_WIDTH as i32,
                    (window_width * MAX_COL_WIDTH_RATIO) as i32,
                );
                let delta_width = col_width - col_bb.width() as i32;

                let table_width = table_bb.width() as i32 + delta_width;
                (*table_node)
                    .style()
                    .set_property("width", &format!("{table_width}px"))
                    .unwrap();

                // col_node.style("width", &format!("{width}px"));
                (*col_node)
                    .style()
                    .set_property("width", &format!("{col_width}px"))
                    .unwrap();
            }
        });

        document
            .add_event_listener_with_callback("mousemove", resize_cb.as_ref().unchecked_ref())
            .unwrap();

        let resize_end = Closure::<dyn Fn(MouseEvent)>::new({
            let document = document.clone();
            move |_e: MouseEvent| {
                let body = document.body().unwrap();
                body.style()
                    .set_property("cursor", cursor_style.as_str())
                    .unwrap();
                set_is_resizing(false);

                document
                    .remove_event_listener_with_callback(
                        "mousemove",
                        resize_cb.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });

        document
            .add_event_listener_with_callback("mouseup", resize_end.as_ref().unchecked_ref())
            .unwrap();

        resize_end.forget();
    };

    view! {
        <th
            node_ref=root_node
            scope="col"
            class="sticky top-0 pl-1 pr-2 pb-1 text-left bg-white dark:bg-secondary-800"
        >
            <div class="inline-flex w-full">
                <div class="grow">
                    <span class="font-primary bold">{display_name}</span>
                </div>
                <div
                    node_ref=drag_handle_node
                    on:mousedown=resize_start
                    class="cursor-col-resize hover:bg-primary-700 hover:delay-100 hover:duration-200 hover:w-[4px] transition-width"
                    class=("w-[2px]", move || !is_resizing())
                    class=(["bg-primary-700", "w-[4px]"], is_resizing)
                ></div>
            </div>
        </th>
    }
}

#[component]
fn DataRow(datum: state::data::Datum) -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let graph_root_name = expect_context::<GraphRootName>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let asset = datum.asset();

    let update_properties = {
        let project = project.rid().read_only();
        let container = datum.path();
        let path = asset.path().read_only();
        move |update: core::project::AssetProperties| {
            spawn_local(async move {
                if let Err(err) = update_asset_properties(
                    project.get_untracked(),
                    container.get_untracked(),
                    path.get_untracked(),
                    update,
                )
                .await
                {
                    tracing::error!(?err);
                    let mut msg = ui_lib::message::Builder::error("Could not save asset.");
                    msg.body(format!("{err:?}"));
                    messages.update(|messages| messages.push(msg.build()));
                }
            });
        }
    };

    let update_name = Callback::new({
        let update_properties = update_properties.clone();
        let asset = asset.clone();
        move |value: Option<String>| {
            if asset.name().get_untracked() == value {
                return;
            }

            let mut update = asset.as_properties();
            update.name = value;
            update_properties(update);
        }
    });

    let update_kind = Callback::new({
        let update_properties = update_properties.clone();
        let asset = asset.clone();
        move |value: Option<String>| {
            if asset.kind().get_untracked() == value {
                return;
            }

            let mut update = asset.as_properties();
            update.kind = value;
            update_properties(update);
        }
    });

    let update_description = Callback::new({
        let update_properties = update_properties.clone();
        let asset = asset.clone();
        move |value: Option<String>| {
            if asset.description().get_untracked() == value {
                return;
            }

            let mut update = asset.as_properties();
            update.description = value;
            update_properties(update);
        }
    });

    let update_tags = Callback::new({
        let update_properties = update_properties.clone();
        let asset = asset.clone();
        move |value: Vec<String>| {
            let tags = asset.tags().get_untracked();
            if value.iter().all(|tag| tags.contains(tag)) {
                return;
            }

            let mut update = asset.as_properties();
            update.tags = value;
            update_properties(update);
        }
    });

    const TH_CLASS: &str = "pl-1 pr-2 align-top text-left";
    const TD_CLASS: &str = "pl-1 pr-2 align-top";
    view! {
        <tr>
            <th scope="row" class=TH_CLASS>
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
            <th scope="row" class=TH_CLASS>
                {
                    let path = asset.path().read_only();
                    move || { path.get().to_string_lossy().to_string() }
                }
            </th>
            <td class=TD_CLASS>
                <properties::Name value=asset.name().read_only() on_change=update_name />
            </td>
            <td class=TD_CLASS>
                <properties::Kind value=asset.kind().read_only() on_change=update_kind />
            </td>
            <td class=TD_CLASS>
                <properties::Description
                    value=asset.description().read_only()
                    on_change=update_description
                />
            </td>
            <td class=TD_CLASS>
                <properties::Tags value=asset.tags().read_only() on_change=update_tags />
            </td>
        </tr>
    }
}

async fn update_asset_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    asset: impl Into<PathBuf>,
    properties: core::project::AssetProperties,
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

    #[component]
    pub fn Kind(
        value: ReadSignal<Option<String>>,
        on_change: Callback<Option<String>>,
    ) -> impl IntoView {
        let input_value = Signal::derive(move || value.get().unwrap_or_default());

        let change = Callback::new(move |value: String| {
            let value = if value.is_empty() { None } else { Some(value) };
            on_change.run(value);
        });

        view! { <editor::Input value=input_value on_change=change empty_value="(no type)".to_string() /> }
    }

    #[component]
    pub fn Description(
        value: ReadSignal<Option<String>>,
        on_change: Callback<Option<String>>,
    ) -> impl IntoView {
        let input_value = Signal::derive(move || value.get().unwrap_or_default());

        let change = Callback::new(move |value: String| {
            let value = if value.is_empty() { None } else { Some(value) };
            on_change.run(value);
        });

        view! {
            <editor::TextArea
                value=input_value
                on_change=change
                empty_value="(no description)".to_string()
            />
        }
    }

    #[component]
    pub fn Tags(value: ReadSignal<Vec<String>>, on_change: Callback<Vec<String>>) -> impl IntoView {
        let input_value = Signal::derive(move || value.get().join(", "));

        let change = Callback::new(move |value: String| {
            let value = value
                .split(",")
                .map(|tag| tag.trim())
                .filter_map(|tag| (!tag.is_empty()).then_some(tag.to_string()))
                .collect();
            on_change.run(value);
        });

        view! { <editor::Input value=input_value on_change=change empty_value="(no tags)".to_string() /> }
    }
}

pub(self) mod editor {
    use leptos::{
        either::Either,
        ev::{KeyboardEvent, MouseEvent, SubmitEvent},
        html,
        prelude::*,
    };
    use leptos_icons::Icon;
    use syre_desktop_ui_lib as ui_lib;

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
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                set_input_value(value.get_untracked());
                set_editing(true);
            }
        };

        let disable_editing = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
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
                if e.button() != ui_lib::types::MouseButton::Primary {
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
                                <Icon icon=ui_lib::icon::Accept />
                            </button>
                            <button
                                type="button"
                                class="cursor-pointer text-lg hover:text-syre-red-700 dark:hover:text-syre-red-600"
                                on:mousedown=disable_editing
                            >
                                <Icon icon=ui_lib::icon::Close />
                            </button>
                        </div>
                    </form>
                })
            } else {
                Either::Right(view! {
                    <div class="flex group">
                        <div
                            class=(
                                ["text-nowrap", "text-secondary-600", "dark:text-secondary-400"],
                                move || value.read().is_empty(),
                            )
                            class="grow"
                        >
                            {display_value.clone()}
                        </div>
                        <div class="pl-2 not-group-hover:invisible">
                            <button class="cursor-pointer" on:mousedown=enable_editing>
                                <Icon icon=ui_lib::icon::Edit />
                            </button>
                        </div>
                    </div>
                })
            }
        }
    }

    #[component]
    pub fn TextArea(
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
        let input_node = NodeRef::<html::Textarea>::new();

        let enable_editing = {
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                set_input_value(value.get_untracked());
                set_editing(true);
            }
        };

        let disable_editing = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
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
                if e.button() != ui_lib::types::MouseButton::Primary {
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
                        <textarea
                            node_ref=input_node
                            bind:value=(input_value, set_input_value)
                            on:keydown=handle_escape
                            class="input-compact grow"
                            placeholder=placeholder.clone()
                        >
                            {value.get_untracked()}
                        </textarea>
                        <div class="pl-2 flex gap-1">
                            <button
                                type="submit"
                                class="cursor-pointer text-lg hover:text-syre-green-700 dark:hover:text-syre-green-400"
                                on:mousedown=change
                            >
                                <Icon icon=ui_lib::icon::Accept />
                            </button>
                            <button
                                type="button"
                                class="cursor-pointer text-lg hover:text-syre-red-700 dark:hover:text-syre-red-600"
                                on:mousedown=disable_editing
                            >
                                <Icon icon=ui_lib::icon::Close />
                            </button>
                        </div>
                    </form>
                })
            } else {
                Either::Right(view! {
                    <div class="flex group">
                        <div
                            class=(
                                ["text-nowrap", "text-secondary-600", "dark:text-secondary-400"],
                                move || value.read().is_empty(),
                            )
                            class="grow"
                        >
                            {display_value.clone()}
                        </div>
                        <div class="pl-2 not-group-hover:invisible">
                            <button class="cursor-pointer" on:mousedown=enable_editing>
                                <Icon icon=ui_lib::icon::Edit />
                            </button>
                        </div>
                    </div>
                })
            }
        }
    }
}
