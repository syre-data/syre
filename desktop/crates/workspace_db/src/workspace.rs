use crate::{project_bar::ProjectBar, state};
use leptos::{
    either::Either,
    ev::{MouseEvent, SubmitEvent},
    html,
    prelude::*,
    svg::filter,
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
    let display_state = state::display::State::new(state.data());
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
pub fn FilterBar() -> impl IntoView {
    use crate::filter::DataFilter;

    let display_state = expect_context::<state::display::State>();
    let filter_bar = display_state.filter_bar().read_only();
    move || {
        filter_bar.read().then_some(view! {
            <div class="px-2 py-1 border-b not-dark:border-b-secondary-900 focus-within:inset-shadow-sm \
            inset-shadow-primary-200/50 dark:inset-shadow-primary-800/50">
                <DataFilter />
            </div>
        })
    }
}

#[component]
fn DataView() -> impl IntoView {
    let graph = expect_context::<ui_lib::state::Graph>();
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    provide_context(GraphRootName(graph.root().name().read_only()));

    let table_node = NodeRef::<html::Table>::new();
    let col_node_name = display_state.columns().name().node_ref();
    let col_node_kind = display_state.columns().kind().node_ref();
    let col_node_description = display_state.columns().description().node_ref();
    let col_node_tags = display_state.columns().tags().node_ref();

    state
        .metadata_keys()
        .read_untracked()
        .iter()
        .for_each(|key| {
            assert!(display_state.columns().new_metadata(key.clone()));
        });

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
                            node_ref=display_state.columns().path().node_ref()
                            class:collapse={
                                let visible = display_state.columns().path().visible().read_only();
                                move || !visible()
                            }
                        />
                        <col
                            node_ref=display_state.columns().file().node_ref()
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
                        <For
                            each=state.metadata_keys()
                            key=|key| key.clone()
                            let:key
                            clone:display_state
                        >
                            <col
                                node_ref=display_state.columns().metadata(&key).unwrap().node_ref()
                                class:collapse={
                                    let visible = display_state
                                        .columns()
                                        .metadata(&key)
                                        .unwrap()
                                        .visible()
                                        .read_only();
                                    move || !visible()
                                }
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
                            <For each=state.metadata_keys() key=|key| key.clone() let:key>
                                <TableHeaderSortable
                                    display_name=key.clone()
                                    sort_field=state::display::SortField::Metadata(key.clone())
                                    col_node=col_node_kind
                                    table_node=table_node
                                />
                            </For>
                        </tr>
                    </thead>
                    <tbody class="h-full overflow-y-auto">
                        <For
                            each=display_state.data()
                            key=|datum| datum.asset().rid().get()
                            let:datum
                        >
                            <DataRow datum />
                        </For>
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

#[component]
fn NoData() -> impl IntoView {
    view! { <div class="pt-2 text-center">"(no data)"</div> }
}

#[component]
fn EmptyFilter() -> impl IntoView {
    view! { <div class="pt-2 text-center">"(empty filter)"</div> }
}

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
                .reduce(|width, ancestor| width + ancestor)
                .unwrap_or(0)
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

    view! {
        <th
            node_ref=root_node
            scope="col"
            on:mousedown=toggle_sort
            class=("z-20", column.pinned().read_only())
            class="group sticky top-0 cursor-pointer pl-1 pr-2 pb-1 text-left \
            bg-white dark:bg-secondary-800"
        >
            <div class="inline-flex w-full">
                <div class="flex grow items-center pr-1">
                    <span class="grow font-primary bold">{display_name.into()}</span>
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

    view! {
        <th
            node_ref=root_node
            scope="col"
            on:mousedown=toggle_sort
            class="group sticky top-0 cursor-pointer pl-1 pr-2 pb-1 text-left \
            bg-white dark:bg-secondary-800"
        >
            <div class="inline-flex w-full">
                <div class="flex grow items-center pr-1">
                    <span class="grow font-primary bold">{display_name.into()}</span>
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
                    class="cursor-col-resize hover:bg-primary-700 hover:delay-100 hover:duration-200 \
                    hover:w-[4px] transition-width"
                    class=("w-[2px]", move || !is_resizing())
                    class=(["bg-primary-700", "w-[4px]"], is_resizing)
                ></div>
            </div>
        </th>
    }
}

#[component]
fn DataRow(datum: state::data::Datum) -> impl IntoView {
    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    let project = expect_context::<ui_lib::state::Project>();
    let graph_root_name = expect_context::<GraphRootName>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let file_node_ref = NodeRef::<html::Th>::new();

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
                    tracing::error!(?err);
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

    let update_metadatum = |key: String| {
        Callback::new(move |value: core::types::Value| {
            tracing::debug!(?value);
        })
    };

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

    let metadata = datum.metadata();
    const TH_CLASS: &str = "pl-1 pr-2 align-top text-left";
    const TD_CLASS: &str = "pl-1 pr-2 align-top";
    view! {
        <tr>
            <th
                scope="row"
                class=TH_CLASS
                class=(
                    ["sticky", "left-0", "z-10", "bg-secondary-800"],
                    display_state.columns().path().pinned().read_only(),
                )
            >
                {
                    let path = datum.path();
                    move || { path.read().to_string_lossy().to_string() }
                }
            </th>
            <th
                node_ref=file_node_ref
                scope="row"
                class=TH_CLASS
                class=(
                    ["sticky", "z-10", "bg-secondary-800"],
                    display_state.columns().file().pinned().read_only(),
                )
            >
                {
                    let path = datum.asset().path().read_only();
                    move || { path.get().to_string_lossy().to_string() }
                }
            </th>
            <td class=TD_CLASS>
                <properties::Name value=datum.asset().name().read_only() on_change=update_name />
            </td>
            <td class=TD_CLASS>
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
                <td class=TD_CLASS>
                    {move || {
                        let value = metadata
                            .read()
                            .iter()
                            .find_map(|(datum_key, value)| {
                                (datum_key == &key).then_some(value.clone())
                            });
                        if let Some(value) = value {
                            Either::Left(
                                view! {
                                    <editor::Metadatum
                                        value
                                        on_change=update_metadatum(key.clone())
                                    />
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <span class="text-nowrap text-secondary-500 dark:text-secondary-400">
                                        "(n/a)"
                                    </span>
                                },
                            )
                        }
                    }}
                </td>
            </For>
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

    #[component]
    pub fn Metadatum(
        value: ReadSignal<Vec<String>>,
        on_change: Callback<Vec<String>>,
    ) -> impl IntoView {
        let input_value = Signal::derive(move || value.get().join(", "));

        let change = Callback::new(move |value: String| {
            let value = value
                .split(",")
                .map(|tag| tag.trim())
                .filter_map(|tag| (!tag.is_empty()).then_some(tag.to_string()))
                .collect();
            on_change.run(value);
        });

        view! { <editor::Input value=input_value on_change=change empty_value="(na)".to_string() /> }
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
    use syre_core as core;
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
                                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
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
                                ["text-nowrap", "text-secondary-500", "dark:text-secondary-400"],
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
    pub fn Metadatum(
        value: ReadSignal<core::types::Value>,
        on_change: Callback<core::types::Value>,
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
                        "todo"
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
                        <div class="grow">
                            {move || {
                                match value.get() {
                                    core::types::Value::String(value) => value,
                                    core::types::Value::Quantity { magnitude, unit } => {
                                        format!("{magnitude} {unit}")
                                    }
                                    core::types::Value::Bool(value) => {
                                        if value { "true".to_string() } else { "false".to_string() }
                                    }
                                    core::types::Value::Number(value) => value.to_string(),
                                    core::types::Value::Array(value) => format!("{value:?}"),
                                    core::types::Value::Null => {
                                        unreachable!("value can not be null")
                                    }
                                }
                            }}
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
