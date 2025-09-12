use crate::{project_bar::ProjectBar, state, types, utils};
use leptos::{either::Either, ev::MouseEvent, html, prelude::*, task::spawn_local};
use leptos_icons::{Icon, Symbol};
use std::{ffi::OsString, path::PathBuf};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_lib as lib;
use syre_desktop_ui_lib as ui_lib;
use wasm_bindgen::{JsCast, prelude::Closure};

const MIN_COL_WIDTH: u32 = 100;
const MAX_COL_WIDTH_RATIO: f64 = 0.8;
const VIRTUALIZATION_WINDOW: usize = 40; // number of elements that nominally fit on screen.
const VIRTUALIZATION_OVERSCAN: usize = 20;
const TABLE_HEADER_LINE_HEIGHT: usize = 28; // NB: Acqired manually via DOM inspection. Should match height of `<DataRow>`.
const TABLE_ROW_LINE_HEIGHT: usize = 24; // NB: Acqired manually via DOM inspection. Should match height of `<DataRow>`.

const CLASS_EMPTY_VALUE: &str = "text-nowrap text-secondary-500 dark:text-secondary-400";
const CLASS_EMPTY_VALUE_ARR: &[&str] = &[
    "text-nowrap",
    "text-secondary-500",
    "dark:text-secondary-400",
];
const EMPTY_NAME: &str = "(no name)";
const EMPTY_KIND: &str = "(no type)";
const EMPTY_DESCRIPTION: &str = "(no description)";
const EMPTY_TAGS: &str = "(no tags)";
const EMPTY_METADATUM: &str = "(n/a)";

#[derive(Clone)]
struct VirtualizationRange {
    /// Start index
    start: Signal<usize>,
    /// End index
    end: Signal<usize>,
    /// Number of items
    length: Signal<usize>,

    active: Vec<RwSignal<bool>>,
}

impl VirtualizationRange {
    pub fn new(top: Signal<f64>, data: ReadSignal<Vec<state::data::Datum>>) -> Self {
        let start = Signal::derive(move || {
            let data_top = (top.get() as usize)
                .checked_sub(TABLE_HEADER_LINE_HEIGHT)
                .unwrap_or(0)
                / TABLE_ROW_LINE_HEIGHT;
            data_top.checked_sub(VIRTUALIZATION_OVERSCAN).unwrap_or(0)
        });

        let end = Signal::derive(move || {
            let data_top = (top.get() as usize)
                .checked_sub(TABLE_HEADER_LINE_HEIGHT)
                .unwrap_or(0)
                / TABLE_ROW_LINE_HEIGHT;
            let end = data_top + VIRTUALIZATION_WINDOW + VIRTUALIZATION_OVERSCAN;
            usize::min(end, data.read().len())
        });

        let length = Signal::derive(move || end.get() - start.get());

        let active = data
            .read_untracked()
            .iter()
            .map(|_| RwSignal::new(false))
            .collect::<Vec<_>>();
        Effect::new({
            let active = active.clone();
            move || {
                let data_top = (top.get() as usize)
                    .checked_sub(TABLE_HEADER_LINE_HEIGHT)
                    .unwrap_or(0)
                    / TABLE_ROW_LINE_HEIGHT;
                let start = data_top.checked_sub(VIRTUALIZATION_OVERSCAN).unwrap_or(0);
                let end = data_top + VIRTUALIZATION_WINDOW + VIRTUALIZATION_OVERSCAN;
                let end = usize::min(end, data.read().len());
                active.iter().enumerate().for_each(|(idx, active)| {
                    let val = idx >= start && idx <= end;
                    if active.with_untracked(|active| *active != val) {
                        active.set(val);
                    }
                });
            }
        });

        Self {
            start,
            end,
            length,
            active,
        }
    }

    /// Start of data including overscan.
    pub fn start(&self) -> Signal<usize> {
        self.start
    }

    /// End of data including overscan.
    pub fn end(&self) -> Signal<usize> {
        self.end
    }

    pub fn length(&self) -> Signal<usize> {
        self.length
    }

    pub fn active(&self, idx: usize) -> ReadSignal<bool> {
        self.active[idx].read_only()
    }
}

#[derive(derive_more::Deref, Clone, Copy)]
struct GraphRootName(ReadSignal<OsString>);

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn Workspace() -> impl IntoView {
    let graph = expect_context::<ui_lib::state::Graph>();
    let state = state::data::State::from(graph);
    let display_state = state::display::State::new(&state);
    provide_context(state);
    provide_context(display_state);

    view! {
        <div class="flex flex-col h-full">
            <div class="border-b not-dark:border-b-secondary-900">
                <ProjectBar />
            </div>
            <FilterBar />
            <div class="grow min-h-0">
                <DataView />
            </div>
        </div>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="text-center">"Preparing database view"</div> }
}

#[component]
pub fn FilterBar() -> impl IntoView {
    use crate::filter::DataFilter;

    let display_state = expect_context::<state::display::State>();
    let workspace_graph_state = expect_context::<ui_lib::state::WorkspaceGraph>();

    let select_all = {
        let selection_resources = workspace_graph_state.selection_resources().clone();
        let data = display_state.data();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            let rids = data
                .read_untracked()
                .iter()
                .map(|datum| datum.asset().rid().get_untracked())
                .collect();
            selection_resources.set_many(&rids, true).unwrap();
        }
    };

    let clear_all = {
        let selection_resources = workspace_graph_state.selection_resources().clone();
        let data = display_state.data();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            let rids = data
                .read_untracked()
                .iter()
                .map(|datum| datum.asset().rid().get_untracked())
                .collect();
            selection_resources.set_many(&rids, false).unwrap();
        }
    };

    let all_selected = {
        let selected = workspace_graph_state.selection_resources().selected();
        let data = display_state.data();
        move || {
            data.read().iter().all(|datum| {
                selected
                    .read()
                    .iter()
                    .find(|resource| *resource.rid() == datum.asset().rid().read_only())
                    .is_some()
            })
        }
    };

    view! {
        <div
            class="flex gap-1 px-2 py-1 border-b not-dark:border-b-secondary-900 focus-within:inset-shadow-sm \
            inset-shadow-primary-200/50 dark:inset-shadow-primary-800/50"
            class:hidden={
                let visible = display_state.filter_bar().read_only();
                move || !visible()
            }
        >
            <div class="grow">
                <DataFilter />
            </div>
            <div>
                {
                    let all_selected = all_selected.clone();
                    let select_all = select_all.clone();
                    let clear_all = clear_all.clone();
                    move || {
                        if all_selected() {
                            Either::Left(
                                view! {
                                    <button
                                        on:mousedown=clear_all.clone()
                                        class="btn-secondary p-1 rounded-xs cursor-pointer"
                                        title="Clear selection"
                                    >
                                        <Icon icon=icondata::RiCheckboxMultipleSystemFill />
                                    </button>
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <button
                                        on:mousedown=select_all.clone()
                                        class="btn-secondary p-1 rounded-xs cursor-pointer"
                                        title="Select all"
                                    >
                                        <Icon icon=icondata::RiCheckboxMultipleSystemLine />
                                    </button>
                                },
                            )
                        }
                    }
                }
            </div>
        </div>
    }
}

#[component]
fn DataView() -> impl IntoView {
    const MAX_COL_LEN: usize = 25;
    // NB: This must be manually changed if root font size changes.
    // Can check with `getComputedStyle(document.documentElement).fontSize`.
    // May not be accurate for character width, so may have to change from given value.
    const REM_TO_PX: usize = 10;
    const EDIT_BUTTON_WIDTH: usize = 1;

    let graph = expect_context::<ui_lib::state::Graph>();
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    let root_node = NodeRef::<html::Div>::new();
    let scroll = leptos_use::use_scroll_with_options(
        root_node,
        leptos_use::UseScrollOptions::default()
            .behavior(leptos_use::ScrollBehavior::Smooth)
            .throttle(50.0),
    );
    let vrange = VirtualizationRange::new(scroll.y, display_state.data());
    provide_context(vrange.clone());
    provide_context(GraphRootName(graph.root().name().read_only()));

    let table_node = NodeRef::<html::Table>::new();
    let col_node_name = display_state.columns().name().node_ref();
    let col_node_kind = display_state.columns().kind().node_ref();
    let col_node_description = display_state.columns().description().node_ref();
    let col_node_tags = display_state.columns().tags().node_ref();

    // NOTE: Must manually set table column widths
    // because `table-layout: auto` causes errors
    // when finding the column widths if pinnable columns
    // are pinned before adjusting their size.
    // Using `table-layout: fixed` resolves this issue,
    // but means initial column widths must be
    // calculated manually.
    let mut max_width_path = 9; // (no path)
    let mut max_width_file = 9; // (no file)
    let mut max_width_name = 9; // (no name)
    let mut max_width_kind = 9; // (no type)
    let mut max_width_desc = 16; // (no description)
    let mut max_width_tags = 9; // (no tags)
    let mut max_width_md = std::collections::HashMap::new();
    for datum in state.data().read_untracked().iter() {
        let path_width = datum
            .path()
            .read_untracked()
            .to_string_lossy()
            .chars()
            .count();
        let file_width = datum
            .asset()
            .path()
            .read_untracked()
            .to_string_lossy()
            .chars()
            .count();
        let name_width = datum
            .asset()
            .name()
            .read_untracked()
            .as_ref()
            .map(|value| value.chars().count())
            .unwrap_or(0);
        let kind_width = datum
            .asset()
            .kind()
            .read_untracked()
            .as_ref()
            .map(|value| value.chars().count())
            .unwrap_or(0);
        let desc_width = datum
            .asset()
            .description()
            .read_untracked()
            .as_ref()
            .map(|value| value.chars().count())
            .unwrap_or(0);
        let tags_width = datum
            .asset()
            .tags()
            .read_untracked()
            .iter()
            .map(|tag| tag.chars().count())
            .sum();
        if path_width > max_width_path {
            max_width_path = path_width;
        }
        if file_width > max_width_file {
            max_width_file = file_width;
        }
        if path_width > max_width_path {
            max_width_path = path_width;
        }
        if name_width > max_width_name {
            max_width_name = name_width;
        }
        if kind_width > max_width_kind {
            max_width_kind = kind_width;
        }
        if desc_width > max_width_desc {
            max_width_desc = desc_width;
        }
        if tags_width > max_width_tags {
            max_width_tags = tags_width;
        }

        for (key, value) in datum.metadata().read_untracked().iter() {
            let width = value.with_untracked(|value| metadatum_value_len(value));
            let entry = max_width_md
                .entry(key.clone())
                .or_insert(usize::max(key.chars().count(), 5) + EDIT_BUTTON_WIDTH); // key length or `(n/a)`, +1 for edit button
            if width > *entry {
                *entry = width;
            }
        }
    }
    let width_path = usize::min(max_width_path, MAX_COL_LEN) * REM_TO_PX;
    let width_file = usize::min(max_width_file, MAX_COL_LEN) * REM_TO_PX;
    let width_name = (usize::min(max_width_name + EDIT_BUTTON_WIDTH, MAX_COL_LEN)) * REM_TO_PX;
    let width_kind = (usize::min(max_width_kind + EDIT_BUTTON_WIDTH, MAX_COL_LEN)) * REM_TO_PX;
    let width_desc = (usize::min(max_width_desc + EDIT_BUTTON_WIDTH, MAX_COL_LEN)) * REM_TO_PX;
    let width_tags = (usize::min(max_width_tags + EDIT_BUTTON_WIDTH, MAX_COL_LEN)) * REM_TO_PX;
    let width_md = max_width_md
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                (usize::min(value + EDIT_BUTTON_WIDTH, MAX_COL_LEN)) * REM_TO_PX,
            )
        })
        .collect::<Vec<_>>();
    let width_table = width_path
        + width_file
        + width_name
        + width_kind
        + width_desc
        + width_tags
        + width_md.iter().map(|(_, value)| value).sum::<usize>();

    display_state.columns().path().width().set(width_path);
    display_state.columns().file().width().set(width_file);

    let table_height = {
        let data = display_state.data();
        move || {
            format!(
                "{}px",
                TABLE_HEADER_LINE_HEIGHT + data.read().len() * TABLE_ROW_LINE_HEIGHT
            )
        }
    };

    let table_buffer_top = {
        let start = vrange.start();
        move || format!("{}px", start.get() * TABLE_ROW_LINE_HEIGHT)
    };

    let table_buffer_bottom = {
        let end = vrange.end();
        let data = display_state.data();
        move || {
            let buffer = (data.read().len() - end.get()) * TABLE_ROW_LINE_HEIGHT;
            format!("{buffer}px")
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
            <div
                node_ref=root_node
                class="overflow-auto scrollbar-thin w-full h-full"
            >
                <Symbol icon=ui_lib::icon::Edit id="workspace_db-data_view-edit"/>
                <table
                    node_ref=table_node
                    class="table-fixed relative min-w-full"
                    style:width=format!("{width_table}px")
                    style:height=table_height
                >
                    <colgroup>
                        <col
                            node_ref=display_state.columns().path().node_ref()
                            class:collapse={
                                let visible = display_state.columns().path().visible().read_only();
                                move || !visible()
                            }
                            style:width=format!("{width_path}px")
                        />
                        <col
                            node_ref=display_state.columns().file().node_ref()
                            class:collapse={
                                let visible = display_state.columns().file().visible().read_only();
                                move || !visible()
                            }
                            style:width=format!("{width_file}px")
                        />
                        <col
                            node_ref=col_node_name
                            class:collapse={
                                let visible = display_state.columns().name().visible().read_only();
                                move || !visible()
                            }
                            style:width=format!("{width_name}px")
                        />
                        <col
                            node_ref=col_node_kind
                            class:collapse={
                                let visible = display_state.columns().kind().visible().read_only();
                                move || !visible()
                            }
                            style:width=format!("{width_kind}px")
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
                            style:width=format!("{width_desc}px")
                        />
                        <col
                            node_ref=col_node_tags
                            class:collapse={
                                let visible = display_state.columns().tags().visible().read_only();
                                move || !visible()
                            }
                            style:width=format!("{width_tags}px")
                        />
                        <For
                            each=display_state.columns().metadata()
                            key=|(key, _)| key.clone()
                            let:((key, column))
                            clone:width_md
                        >
                            <col
                                node_ref=column.node_ref()
                                class:collapse={
                                    let visible = column.visible().read_only();
                                    move || !visible()
                                }
                                style:width=format!(
                                    "{}px",
                                    width_md.iter().find_map(|(md_key, value)| (*md_key == key).then_some(value)).unwrap() // TODO: May be `None`.
                                )
                            />
                        </For>
                    </colgroup>
                    <thead>
                        <tr>
                            <TableHeaderSortablePinnable
                                display_name="Path"
                                sort_field=state::display::SortField::Path
                                column=display_state.columns().path().clone()
                                table_node=table_node
                            />
                            <TableHeaderSortablePinnable
                                display_name="File"
                                sort_field=state::display::SortField::File
                                column=display_state.columns().file().clone()
                                table_node=table_node
                                ancestors=vec![display_state.columns().path().clone()]
                            />
                            <TableHeaderSortable
                                display_name="Name"
                                sort_field=state::display::SortField::Name
                                col_node=col_node_name
                                table_node=table_node
                            />
                            <TableHeaderSortable
                                display_name="Type"
                                sort_field=state::display::SortField::Kind
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
                            <For each=state.metadata_keys() key=|key| key.clone() let:key clone:display_state>
                                <TableHeaderSortable
                                    display_name=key.clone()
                                    sort_field=state::display::SortField::Metadata(key.clone())
                                    col_node=display_state.columns().get_metadata(&key).map(|col| col.node_ref()).unwrap()
                                    table_node=table_node
                                />
                            </For>
                        </tr>
                    </thead>
                    <tbody class="overflow-y-auto">
                        <ForEnumerate
                            each=display_state.data()
                            key=|datum| datum.asset().rid().get()
                            let(idx, datum)
                            clone:vrange
                        >
                        {
                            let active = vrange.active(idx.get_untracked());
                            view! { <DataRow datum active {..} style:height="1em" /> }
                        }
                        </ForEnumerate>
                    </tbody>
                </table>
                {
                    let data = display_state.data();
                    move || { data.read().is_empty().then_some(view! { <EmptyFilter /> }) }
                }
            </div>
        </Show>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn NoData() -> impl IntoView {
    view! { <div class="pt-2 text-center">"(no data)"</div> }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn EmptyFilter() -> impl IntoView {
    view! { <div class="pt-2 text-center">"(empty filter)"</div> }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn TableHeaderSortablePinnable(
    display_name: impl Into<String>,
    sort_field: state::display::SortField,
    column: state::display::ColumnPinnable,
    table_node: NodeRef<html::Table>,
    #[prop(optional)] ancestors: Vec<state::display::ColumnPinnable>,
) -> impl IntoView {
    let display_state = expect_context::<state::display::State>();
    let (is_resizing, set_is_resizing) = signal(false);
    let root_node = NodeRef::<html::Th>::new();
    let drag_handle_node = NodeRef::<html::Div>::new();

    let resize_start = {
        let col_node = column.node_ref();
        let col_width_state = column.width().write_only();
        move |e: MouseEvent| {
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

                    (*col_node)
                        .style()
                        .set_property("width", &format!("{col_width}px"))
                        .unwrap();

                    col_width_state.set(col_width as usize);
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
        }
    };

    let toggle_sort = {
        let sort = display_state.sort();
        let sort_field = sort_field.clone();
        move |e: MouseEvent| {
            use state::display::{Sort, SortDirection};

            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            if sort_field != sort.read_untracked().field().clone() {
                sort.set(Sort::ascending(sort_field.clone()));
            } else {
                let dir = sort.read_untracked().direction();
                match dir {
                    SortDirection::Asc => {
                        sort.set(Sort::descending(sort_field.clone()));
                    }
                    SortDirection::Des => {
                        sort.set(Sort::ascending(sort_field.clone()));
                    }
                }
            }
        }
    };

    let toggle_pinned = {
        let pinned = column.pinned();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            *pinned.write() = !pinned();
        }
    };

    let ancestors_width = {
        let ancestors = ancestors.clone();
        move || {
            ancestors
                .iter()
                .filter(|ancestor| ancestor.visible().get() && ancestor.pinned().get())
                .map(|ancestor| ancestor.width().get())
                .sum::<usize>()
        }
    };

    Effect::watch(
        {
            let pinned = column.pinned().read_only();
            move || (pinned(), ancestors_width())
        },
        {
            move |(pinned, left), _, _| {
                let node = root_node.get_untracked().unwrap();
                if *pinned {
                    (*node)
                        .style()
                        .set_property("left", &format!("{left}px"))
                        .unwrap();
                } else {
                    (*node).style().remove_property("left").unwrap();
                }
            }
        },
        false,
    );

    let display_name = display_name.into();
    view! {
        <th
            node_ref=root_node
            scope="col"
            on:mousedown=toggle_sort
            class=("z-20", column.pinned().read_only())
            class="group sticky top-0 truncate cursor-pointer pl-1 pr-2 pb-1 text-left \
            bg-white dark:bg-secondary-800 z-10"
            title=display_name.clone()
        >
            <div class="inline-flex w-full">
                <div class="flex grow items-center pr-1 truncate">
                    <span class="grow font-primary bold truncate">{display_name.clone()}</span>
                    <span
                        class=(
                            ["not-group-hover:invisible", "group-hover:visible"],
                            {
                                let sort = display_state.sort();
                                let sort_field = sort_field.clone();
                                move || { sort_field != *sort.get().field() }
                            },
                        )
                        class="pl-2 inline-block align-middle"
                    >
                        {
                            let sort = display_state.sort();
                            let sort_field = sort_field.clone();
                            move || {
                                let icon = if sort_field == *sort.get().field() {
                                    match sort.get().direction() {
                                        state::display::SortDirection::Asc => {
                                            ui_lib::icon::CaretDown
                                        }
                                        state::display::SortDirection::Des => ui_lib::icon::CaretUp,
                                    }
                                } else {
                                    ui_lib::icon::CaretDown
                                };
                                view! { <Icon icon /> }
                            }
                        }
                    </span>
                    <span class=(
                        ["not-group-hover:invisible", "group-hover:visible"],
                        {
                            let pinned = column.pinned().read_only();
                            move || !pinned.get()
                        },
                    )>
                        <button class="cursor-pointer" on:mousedown=toggle_pinned>
                            <Icon icon=icondata::BsPinAngleFill />
                        </button>
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

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn TableHeaderSortable(
    display_name: impl Into<String>,
    sort_field: state::display::SortField,
    col_node: NodeRef<html::Col>,
    table_node: NodeRef<html::Table>,
) -> impl IntoView {
    let display_state = expect_context::<state::display::State>();
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
        let sort = display_state.sort();
        let sort_field = sort_field.clone();
        move |e: MouseEvent| {
            use state::display::{Sort, SortDirection};

            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            if sort_field != sort.read_untracked().field().clone() {
                sort.set(Sort::ascending(sort_field.clone()));
            } else {
                let dir = sort.read_untracked().direction();
                match dir {
                    SortDirection::Asc => {
                        sort.set(Sort::descending(sort_field.clone()));
                    }
                    SortDirection::Des => {
                        sort.set(Sort::ascending(sort_field.clone()));
                    }
                }
            }
        }
    };

    let display_name = display_name.into();
    view! {
        <th
            node_ref=root_node
            scope="col"
            on:mousedown=toggle_sort
            class="group sticky top-0 truncate cursor-pointer pl-1 pr-2 pb-1 text-left \
            bg-white dark:bg-secondary-800 z-10"
            title=display_name.clone()
        >
            <div class="inline-flex w-full">
                <div class="flex grow items-center pr-1 truncate">
                    <span class="grow font-primary bold truncate">{display_name.clone()}</span>
                    <span
                        class=(
                            ["not-group-hover:invisible", "group-hover:visible"],
                            {
                                let sort = display_state.sort();
                                let sort_field = sort_field.clone();
                                move || { sort_field != *sort.get().field() }
                            },
                        )
                        class="pl-2 inline-block align-middle"
                    >
                        {
                            let sort = display_state.sort();
                            let sort_field = sort_field.clone();
                            move || {
                                let icon = if sort_field == *sort.get().field() {
                                    match sort.get().direction() {
                                        state::display::SortDirection::Asc => {
                                            ui_lib::icon::CaretDown
                                        }
                                        state::display::SortDirection::Des => ui_lib::icon::CaretUp,
                                    }
                                } else {
                                    ui_lib::icon::CaretDown
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

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn TableHeader(
    display_name: &'static str,
    col_node: NodeRef<html::Col>,
    table_node: NodeRef<html::Table>,
) -> impl IntoView {
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
            class="sticky top-0 truncate pl-1 pr-2 pb-1 text-left bg-white dark:bg-secondary-800 z-10"
            title=display_name
        >
            <div class="inline-flex w-full">
                <div class="grow">
                    <span class="font-primary bold">{display_name}</span>
                </div>
                <div
                    node_ref=drag_handle_node
                    on:mousedown=resize_start
                    class="cursor-col-resize hover:bg-primary-700 hover:delay-100 hover:duration-200 \
                    hover:w-[4px] transition-width"
                    class=("w-[2px]", move || !is_resizing())
                    class=(["bg-primary-700", "w-[4px]"], is_resizing)
                ></div>
            </div>
        </th>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn DataRow(datum: state::data::Datum, active: ReadSignal<bool>) -> impl IntoView {
    let workspace_graph_state = expect_context::<ui_lib::state::WorkspaceGraph>();
    let display_state = expect_context::<state::display::State>();
    let file_node_ref = NodeRef::<html::Th>::new();

    let selection_resource = datum
        .asset()
        .rid()
        .with_untracked(|rid| workspace_graph_state.selection_resources().get(rid))
        .unwrap();

    let file_ancestors_width = {
        let ancestors = vec![display_state.columns().path().clone()];
        move || {
            ancestors
                .iter()
                .filter(|ancestor| ancestor.visible().get() && ancestor.pinned().get())
                .map(|ancestor| ancestor.width().get())
                .reduce(|width, ancestor| width + ancestor)
                .unwrap_or(0)
        }
    };

    Effect::watch(
        {
            let pinned = display_state.columns().file().pinned().read_only();
            move || (pinned(), file_ancestors_width())
        },
        {
            move |(pinned, left), _, _| {
                let node = file_node_ref.get_untracked().unwrap();
                if *pinned {
                    (*node)
                        .style()
                        .set_property("left", &format!("{left}px"))
                        .unwrap();
                } else {
                    (*node).style().remove_property("left").unwrap();
                }
            }
        },
        false,
    );

    let mousedown = {
        let selection_resources = workspace_graph_state.selection_resources().clone();
        let rid = datum.asset().rid().read_only();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }
            e.stop_propagation();

            let action = rid.with_untracked(|rid| {
                selection_resources.selected().with_untracked(|selected| {
                    utils::interpret_resource_selection_action(rid, selected, e.shift_key())
                })
            });
            match action {
                types::SelectionAction::Unselect => {
                    rid.with_untracked(|rid| selection_resources.set(rid, false).unwrap())
                }
                types::SelectionAction::Select => {
                    rid.with_untracked(|rid| selection_resources.set(rid, true).unwrap())
                }
                types::SelectionAction::SelectOnly => {
                    rid.with_untracked(|rid| selection_resources.select_only(rid).unwrap())
                }
                types::SelectionAction::Clear => selection_resources.clear(),
            }
        }
    };

    let path = {
        let path = datum.path();
        move || path.read().to_string_lossy().to_string()
    };

    let file_str = {
        let path = datum.asset().path().read_only();
        move || path.read().to_string_lossy().to_string()
    };

    const TH_CLASS: &str = "pl-1 pr-2 align-top truncate text-left";
    view! {
        <tr
            on:mousedown=mousedown
            class="group hover:bg-secondary-100 dark:hover:bg-secondary-700"
            class=(["bg-secondary-100", "dark:bg-secondary-700"], selection_resource.clone())
        >
            <th
                scope="row"
                class=TH_CLASS
                class=(
                    [
                        "left-0",
                        "sticky",
                        "z-10",
                        "bg-white",
                        "group-hover:bg-secondary-100",
                        "dark:bg-secondary-800",
                        "dark:group-hover:bg-secondary-700",
                    ],
                    display_state.columns().path().pinned().read_only(),
                )
                title=path.clone()
            >
                {path}
            </th>
            <th
                node_ref=file_node_ref
                scope="row"
                class=TH_CLASS
                class=(
                    [
                        "sticky",
                        "z-10",
                        "bg-white",
                        "group-hover:bg-secondary-100",
                        "dark:bg-secondary-800",
                        "dark:group-hover:bg-secondary-700",
                    ],
                    display_state.columns().file().pinned().read_only(),
                )
                title=file_str.clone()
            >
                {file_str}
            </th>
            {
                let datum = datum.clone();
                move || {
                    if active.get() {
                        Either::Left(view! { <DataRowActive datum=datum.clone() /> })
                    } else {
                        Either::Right(view! { <DataRowStatic datum=datum.clone() /> })
                    }
                }
            }
        </tr>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn DataRowStatic(datum: state::data::Datum) -> impl IntoView {
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    let project = expect_context::<ui_lib::state::Project>();
    let graph_root_name = expect_context::<GraphRootName>();

    let md_title = {
        let metadata = datum.metadata();
        move |key: String| {
            move || {
                metadata
                    .read()
                    .iter()
                    .find_map(|(datum_key, value)| (datum_key == &key).then_some(value.clone()))
                    .map(|datum| datum.value())
                    .map(|value| metadatum_value_to_string(value.get()))
            }
        }
    };

    let name = datum.asset().name().read_only();
    let kind = datum.asset().kind().read_only();
    let description = datum.asset().description().read_only();
    let tags = datum.asset().tags().read_only();
    let metadata = datum.metadata();

    const TD_CLASS: &str = "pl-1 pr-2 align-top truncate";
    const TD_METADATA_CLASS: &str = "pl-1 pr-2 align-top truncate has-[form]:overflow-visible";
    view! {
        <td
            class=TD_CLASS
            class=(
                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                move || name.read().is_none(),
            )
            title=move || name.get()
        >
            {move || name.get().unwrap_or(EMPTY_NAME.to_string())}
        </td>
        <td
            class=TD_CLASS
            class=(
                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                move || kind.read().is_none(),
            )
            title=move || kind.get()
        >
            {move || kind.get().unwrap_or(EMPTY_KIND.to_string())}
        </td>
        <td
            class=TD_CLASS
            class=(
                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                move || description.read().is_none(),
            )
        >
            {move || description.get().unwrap_or(EMPTY_DESCRIPTION.to_string())}
        </td>
        <td
            class=TD_CLASS
            class=(
                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                move || tags.read().is_empty(),
            )
        >
            {move || {
                if tags.read().is_empty() { EMPTY_TAGS.to_string() } else { tags.get().join(", ") }
            }}
        </td>
        <For each=state.metadata_keys() key=|key| key.clone() let:key>
            <td class=TD_METADATA_CLASS title=md_title(key.clone())>
                {
                    let key = key.clone();
                    move || {
                        let value = metadata
                            .read()
                            .iter()
                            .find_map(|(datum_key, value)| {
                                (datum_key == &key).then_some(value.clone())
                            });
                        if let Some(value) = value.as_ref() {
                            let value = value.value();
                            Either::Left(move || metadatum_value_to_string(value.get()))
                        } else {
                            Either::Right(
                                view! { <span class=CLASS_EMPTY_VALUE>{EMPTY_METADATUM}</span> },
                            )
                        }
                    }
                }
            </td>
        </For>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn DataRowActive(datum: state::data::Datum) -> impl IntoView {
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    let project = expect_context::<ui_lib::state::Project>();
    let graph_root_name = expect_context::<GraphRootName>();
    let messages = expect_context::<ui_lib::message::Messages>();

    let update_properties = {
        let project = project.rid().read_only();
        let container = datum.path();
        let path = datum.asset().path().read_only();
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
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not update asset properties: {err:?}");
                    let msg = ui_lib::message::Builder::error("Could not save asset.");
                    let msg = msg.body(format!("{err:?}"));
                    messages.push_message(msg.build_str());
                }
            });
        }
    };

    let update_name = Callback::new({
        let update_properties = update_properties.clone();
        let asset = datum.asset().clone();
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
        let asset = datum.asset().clone();
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
        let asset = datum.asset().clone();
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
        let asset = datum.asset().clone();
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

    let update_metadatum = {
        let update_properties = update_properties.clone();
        let asset = datum.asset().clone();
        move |key: String| {
            Callback::new({
                let key = key.clone();
                let update_properties = update_properties.clone();
                let asset = asset.clone();
                move |value: core::types::Value| {
                    let mut update = asset.as_properties();
                    update
                        .metadata
                        .entry(key.clone())
                        .and_modify(|md_value| *md_value = value.clone())
                        .or_insert(value);

                    update_properties(update);
                }
            })
        }
    };

    let remove_metadatum = Callback::new({
        let update_properties = update_properties.clone();
        let asset = datum.asset().clone();
        move |key: String| {
            let mut update = asset.as_properties();
            update
                .metadata
                .remove(&key)
                .expect("value should be present");
            update_properties(update);
        }
    });

    let md_title = {
        let metadata = datum.metadata();
        move |key: String| {
            move || {
                metadata
                    .read()
                    .iter()
                    .find_map(|(datum_key, value)| (datum_key == &key).then_some(value.clone()))
                    .map(|datum| datum.value())
                    .map(|value| metadatum_value_to_string(value.get()))
            }
        }
    };

    let metadata = datum.metadata();
    const TD_CLASS: &str = "pl-1 pr-2 align-top truncate";
    const TD_METADATA_CLASS: &str = "pl-1 pr-2 align-top truncate has-[form]:overflow-visible";
    view! {
        <td
            class=TD_CLASS
            title={
                let name = datum.asset().name().read_only();
                move || name.get()
            }
        >
            <properties::Name value=datum.asset().name().read_only() on_change=update_name />
        </td>
        <td
            class=TD_CLASS
            title={
                let kind = datum.asset().kind().read_only();
                move || kind.get()
            }
        >
            <properties::Kind value=datum.asset().kind().read_only() on_change=update_kind />
        </td>
        <td class=TD_CLASS>
            <properties::Description
                value=datum.asset().description().read_only()
                on_change=update_description
            />
        </td>
        <td class=TD_CLASS>
            <properties::Tags value=datum.asset().tags().read_only() on_change=update_tags />
        </td>
        <For each=state.metadata_keys() key=|key| key.clone() let:key>
            <td class=TD_METADATA_CLASS title=md_title(key.clone())>
                <properties::Metadatum
                    key=key.clone()
                    metadata
                    on_change=update_metadatum(key.clone())
                    on_remove=remove_metadatum
                />
            </td>
        </For>
    }
}

async fn update_asset_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    asset: impl Into<PathBuf>,
    properties: core::project::AssetProperties,
) -> Result<(), lib::command::asset::error::Update> {
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
    use crate::state;
    use leptos::prelude::*;
    use syre_core as core;

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

        view! { <editor::Input value=input_value on_change=change empty_value=super::EMPTY_NAME /> }
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

        view! { <editor::Input value=input_value on_change=change empty_value=super::EMPTY_KIND /> }
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
                empty_value=super::EMPTY_DESCRIPTION
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

        view! { <editor::Input value=input_value on_change=change empty_value=super::EMPTY_TAGS /> }
    }

    #[component]
    pub fn Metadatum(
        key: String,
        metadata: ReadSignal<Vec<(String, state::data::MetadatumValue)>>,
        on_change: Callback<core::types::Value>,
        on_remove: Callback<String>,
    ) -> impl IntoView {
        let remove = Trigger::new();
        Effect::watch(
            move || remove.track(),
            {
                let key = key.clone();
                move |_, _, _| {
                    on_remove.run(key.clone());
                }
            },
            false,
        );

        move || {
            let value = metadata
                .read()
                .iter()
                .find_map(|(datum_key, value)| (datum_key == &key).then_some(value.clone()));

            view! { <editor::Metadatum value on_change on_remove=remove /> }
        }
    }
}

pub(self) mod editor {
    use crate::state;
    use leptos::{
        either::Either,
        ev::{KeyboardEvent, MouseEvent, SubmitEvent},
        html,
        prelude::*,
    };
    use leptos_icons::Icon;
    use syre_core as core;
    use syre_desktop_ui_lib as ui_lib;

    const ESCAPE_KEY_CODE: &str = "Escape";

    #[component]
    fn EditIcon() -> impl IntoView {
        template! {
            <svg width="1em" height="1em">
                <use href="#workspace_db-data_view-edit" />
            </svg>
        }
    }

    #[component]
    pub fn Input(
        value: Signal<String>,
        on_change: Callback<String>,
        /// Displayed if `value` is empty and not in editing mode.
        #[prop(into)]
        empty_value: String,
        /// `<input>` placeholder.
        #[prop(optional)]
        placeholder: Option<String>,
    ) -> impl IntoView {
        let (editing, set_editing) = signal(false);
        let (input_value, set_input_value) = signal(value.get_untracked());
        let input_node = NodeRef::<html::Input>::new();

        move || {
            if editing.get() {
                Either::Left(
                    view! { <InputEditor value on_change set_editing placeholder=placeholder.clone() /> },
                )
            } else {
                Either::Right(
                    view! { <InputValue value empty_value=empty_value.clone() set_editing /> },
                )
            }
        }
    }

    #[component]
    fn InputEditor(
        value: Signal<String>,
        on_change: Callback<String>,
        set_editing: WriteSignal<bool>,
        /// `<input>` placeholder.
        placeholder: Option<String>,
    ) -> impl IntoView {
        let (input_value, set_input_value) = signal(value.get_untracked());
        let input_node = NodeRef::<html::Input>::new();

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

        Effect::new(move || {
            if let Some(input_node) = input_node.get() {
                if let Err(err) = input_node.focus() {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not focus input node: {err:?}");
                };
            }
        });

        view! {
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
        }
    }

    #[component]
    fn InputValue(
        value: Signal<String>,
        set_editing: WriteSignal<bool>,

        /// Displayed if `value` is empty and not in editing mode.
        #[prop(into)]
        empty_value: String,
    ) -> impl IntoView {
        let enable_editing = {
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                set_editing(true);
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

        view! {
            <div class="flex group/editor">
                <div
                    class=(
                        ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                        move || value.read().is_empty(),
                    )
                    class="grow truncate"
                >
                    {display_value.clone()}
                </div>
                <div class="pl-2 invisible group-hover/editor:visible">
                    <button class="cursor-pointer align-middle" on:mousedown=enable_editing>
                        <EditIcon />
                    </button>
                </div>
            </div>
        }
    }

    #[component]
    pub fn TextArea(
        value: Signal<String>,
        on_change: Callback<String>,

        /// Displayed if `value` is empty and not in editing mode.
        #[prop(into)]
        empty_value: String,

        /// `<input>` placeholder.
        #[prop(optional)]
        placeholder: Option<String>,
    ) -> impl IntoView {
        let (editing, set_editing) = signal(false);

        move || {
            if editing.get() {
                Either::Left(
                    view! { <TextAreaEditor value on_change set_editing placeholder=placeholder.clone() /> },
                )
            } else {
                Either::Right(
                    view! { <TextAreaValue value empty_value=empty_value.clone() set_editing /> },
                )
            }
        }
    }

    #[component]
    fn TextAreaEditor(
        value: Signal<String>,
        on_change: Callback<String>,
        set_editing: WriteSignal<bool>,
        /// `<input>` placeholder.
        placeholder: Option<String>,
    ) -> impl IntoView {
        let (input_value, set_input_value) = signal(value.get_untracked());
        let input_node = NodeRef::<html::Textarea>::new();

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

        Effect::new(move || {
            if let Some(input_node) = input_node.get() {
                if let Err(err) = input_node.focus() {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("could not focus input node: {err:?}");
                };
            }
        });

        view! {
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
        }
    }

    #[component]
    fn TextAreaValue(
        value: Signal<String>,
        /// Displayed if `value` is empty and not in editing mode.
        #[prop(into)]
        empty_value: String,
        set_editing: WriteSignal<bool>,
    ) -> impl IntoView {
        let enable_editing = {
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                set_editing(true);
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

        view! {
            <div class="flex group/editor">
                <div
                    class=(
                        ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
                        move || value.read().is_empty(),
                    )
                    class="grow truncate"
                >
                    {display_value.clone()}
                </div>
                <div class="pl-2 invisible group-hover/editor:visible">
                    <button class="cursor-pointer align-middle" on:mousedown=enable_editing>
                        <EditIcon />
                    </button>
                </div>
            </div>
        }
    }

    #[component]
    pub fn Metadatum(
        value: Option<state::data::MetadatumValue>,
        on_change: Callback<core::types::Value>,
        on_remove: Trigger,
    ) -> impl IntoView {
        let (editing, set_editing) = signal(false);

        move || {
            if editing.get() {
                Either::Left(
                    view! { <MetadatumEditor value=value.clone() on_change on_remove set_editing /> },
                )
            } else {
                Either::Right(view! { <MetadatumValue value=value.clone() set_editing /> })
            }
        }
    }

    #[component]
    fn MetadatumEditor(
        value: Option<state::data::MetadatumValue>,
        on_change: Callback<core::types::Value>,
        on_remove: Trigger,
        set_editing: WriteSignal<bool>,
    ) -> impl IntoView {
        use syre_desktop_editors as editors;

        let (input_value, set_input_value) = signal(
            value
                .as_ref()
                .map(|value| value.get_untracked())
                .unwrap_or(core::types::Value::Number(0.into())),
        );

        if let Some(value) = value.as_ref() {
            Effect::watch(
                value.value(),
                move |value, _, _| {
                    set_input_value(value.clone());
                },
                false,
            );
        }

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

        let oninput = Callback::new(move |value: core::types::Value| {
            set_input_value(value);
        });

        let is_removeable = value
            .as_ref()
            .map(|value| value.is_owned())
            .unwrap_or(false);

        let remove = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            on_remove.notify();
        };

        view! {
            <div class="relative">
                <div class="absolute top-0 left-0 text-primary-600">
                    <Icon icon=icondata::BsCircleFill />
                </div>
                <div class="absolute top-2 left-2 w-30 bg-white dark:bg-secondary-800 border rounded-sm">
                    <div class="p-1">
                        <form on:submit=submit on:keydown=handle_escape>
                            <editors::common::metadata::ValueEditor value=input_value oninput />
                            <div class="flex gap-2 items-center justify-center pt-1">
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
                    </div>
                    {is_removeable
                        .then_some(
                            view! {
                                <hr class="border-secondary-900 dark:border-secondary-200" />
                                <div class="text-center">
                                    <button
                                        on:click=remove
                                        class="p-2 cursor-pointer text-center \
                                        hover:text-syre-red-700 dark:hover:text-syre-red-600"
                                    >
                                        <Icon icon=ui_lib::icon::Trash />
                                    </button>
                                </div>
                            },
                        )}
                </div>
            </div>
        }
    }

    #[component]
    fn MetadatumValue(
        value: Option<state::data::MetadatumValue>,
        set_editing: WriteSignal<bool>,
    ) -> impl IntoView {
        let enable_editing = {
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                set_editing(true);
            }
        };

        view! {
            <div class="flex group/editor">
                <div class="grow truncate">
                    {if let Some(value) = value.as_ref() {
                        let value = value.value();
                        Either::Left(move || super::metadatum_value_to_string(value.get()))
                    } else {
                        Either::Right(
                            view! {
                                <span class="text-nowrap text-secondary-500 dark:text-secondary-400">
                                    {super::EMPTY_METADATUM}
                                </span>
                            },
                        )
                    }}
                </div>
                <div class="pl-2 invisible group-hover/editor:visible">
                    <button class="cursor-pointer align-middle" on:mousedown=enable_editing>
                        <EditIcon />
                    </button>
                </div>
            </div>
        }
    }
}

fn metadatum_value_to_string(value: core::types::Value) -> String {
    match value {
        core::types::Value::String(value) => value,
        core::types::Value::Quantity { magnitude, unit } => {
            format!("{magnitude} {unit}")
        }
        core::types::Value::Bool(value) => {
            if value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        core::types::Value::Number(value) => value.to_string(),
        core::types::Value::Array(value) => format!("{value:?}"),
        core::types::Value::Null => {
            unreachable!("value can not be null")
        }
    }
}

fn metadatum_value_len(value: &core::types::Value) -> usize {
    use core::types::Value;
    match value {
        Value::String(value) => value.chars().count(),
        Value::Number(value) => value.to_string().chars().count(),
        Value::Quantity { magnitude, unit } => {
            magnitude.to_string().chars().count() + unit.chars().count()
        }
        Value::Array(value) => value.iter().map(|value| metadatum_value_len(value)).sum(),
        Value::Bool(_) => 5, // `true` or `false`
        Value::Null => unreachable!("value can not be null"),
    }
}
