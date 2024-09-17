use super::{
    common::{asset_title_closure, interpret_resource_selection_action, SelectionAction},
    state,
};
use crate::{
    commands, common,
    components::{message::Builder as Message, ModalDialog, TruncateLeft},
    pages::project::actions,
    types,
};
use futures::StreamExt;
use has_id::HasId;
use leptos::{
    ev::{DragEvent, MouseEvent, WheelEvent},
    *,
};
use leptos_icons::*;
use serde::Serialize;
use std::{cmp, io, num::NonZeroUsize, ops::Deref, path::PathBuf, rc::Rc};
use syre_core::{project::AnalysisAssociation, types::ResourceId};
use syre_desktop_lib as lib;
use syre_local as local;
use syre_local_database as db;
use tauri_sys::{core::Channel, menu};

const CONTAINER_WIDTH: usize = 250;
const MAX_CONTAINER_HEIGHT: usize = 300;
const PADDING_X_SIBLING: usize = 20;
const PADDING_Y_CHILDREN: usize = 30;
const RADIUS_ADD_CHILD: usize = 10;
const ZOOM_FACTOR_IN: f32 = 0.9; // zoom in should reduce viewport.
const ZOOM_FACTOR_OUT: f32 = 1.1;
const VB_WIDTH_MIN: usize = 500;
const VB_WIDTH_MAX: usize = 10_000;
const VB_HEIGHT_MIN: usize = 500;
const VB_HEIGHT_MAX: usize = 10_000;
pub const DATA_KEY_CONTAINER: &str = "container";
pub const DATA_KEY_ASSET: &str = "asset";

/// Context menu for containers that are `Ok`.
#[derive(derive_more::Deref, Clone)]
struct ContextMenuContainerOk(Rc<menu::Menu>);
impl ContextMenuContainerOk {
    pub fn new(menu: Rc<menu::Menu>) -> Self {
        Self(menu)
    }
}

/// Context menu for assets.
#[derive(derive_more::Deref, Clone)]
struct ContextMenuAsset(Rc<menu::Menu>);
impl ContextMenuAsset {
    pub fn new(menu: Rc<menu::Menu>) -> Self {
        Self(menu)
    }
}

/// Active container for the container context menu.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct ContextMenuActiveContainer(state::graph::Node);

/// Active asset for the asset context menu.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct ContextMenuActiveAsset(ResourceId);

// /// Resize observer run when container nodes change size.
// #[derive(derive_more::Deref, derive_more::From, Clone)]
// struct ContainerResizeObserver(web_sys::ResizeObserver);

#[derive(derive_more::Deref, derive_more::From, Clone, Copy)]
struct ContainerHeight(ReadSignal<usize>);

#[derive(derive_more::Deref, derive_more::From, Clone)]
struct Container(state::graph::Node);

/// Node ref to the modal portal.
#[derive(Clone)]
pub struct PortalRef(NodeRef<html::Div>);
impl Deref for PortalRef {
    type Target = NodeRef<html::Div>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[component]
pub fn Canvas() -> impl IntoView {
    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let messages = expect_context::<types::Messages>();

    let context_menu_active_container =
        create_rw_signal::<Option<ContextMenuActiveContainer>>(None);
    let context_menu_active_asset = create_rw_signal::<Option<ContextMenuActiveAsset>>(None);
    provide_context(context_menu_active_container.clone());
    provide_context(context_menu_active_asset);

    let context_menu_container_ok = create_local_resource(|| (), {
        let project = project.clone();
        let graph = graph.clone();
        let messages = messages.clone();
        move |_| {
            let project = project.clone();
            let graph = graph.clone();
            let messages = messages.clone();
            async move {
                let mut container_open = tauri_sys::menu::item::MenuItemOptions::new("Open");
                container_open.set_id("canvas:container-open");

                let mut container_duplicate =
                    tauri_sys::menu::item::MenuItemOptions::new("Duplicate");
                container_duplicate.set_id("canvas:container-duplicate");

                let mut container_trash = tauri_sys::menu::item::MenuItemOptions::new("Trash");
                container_trash.set_id("canvas:container-trash");

                let (menu, mut listeners) = menu::Menu::with_id_and_items(
                    "canvas:container-ok-context_menu",
                    vec![
                        container_open.into(),
                        container_duplicate.into(),
                        container_trash.into(),
                    ],
                )
                .await;

                spawn_local({
                    // pop from end to beginning
                    let container_trash = listeners.pop().unwrap().unwrap();
                    let container_duplicate = listeners.pop().unwrap().unwrap();
                    let container_open = listeners.pop().unwrap().unwrap();
                    handle_context_menu_container_events(
                        project,
                        graph,
                        messages,
                        context_menu_active_container.read_only(),
                        container_open,
                        container_duplicate,
                        container_trash,
                    )
                });

                Rc::new(menu)
            }
        }
    });

    let context_menu_asset = create_local_resource(|| (), {
        let project = project.clone();
        let graph = graph.clone();
        let messages = messages.clone();
        move |_| {
            let project = project.clone();
            let graph = graph.clone();
            let messages = messages.clone();
            async move {
                let mut asset_open = tauri_sys::menu::item::MenuItemOptions::new("Open");
                asset_open.set_id("canvas:asset-open");

                let (menu, mut listeners) = menu::Menu::with_id_and_items(
                    "canvas:asset-context_menu",
                    vec![asset_open.into()],
                )
                .await;

                spawn_local({
                    let asset_open = listeners.pop().unwrap().unwrap();
                    handle_context_menu_asset_events(
                        project,
                        graph,
                        messages,
                        context_menu_active_asset.read_only(),
                        asset_open,
                    )
                });

                Rc::new(menu)
            }
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { <CanvasLoading /> }
        }>

            {move || {
                let Some(context_menu_container_ok) = context_menu_container_ok.get() else {
                    return None;
                };
                let Some(context_menu_asset) = context_menu_asset.get() else {
                    return None;
                };
                Some(view! { <CanvasView context_menu_container_ok context_menu_asset /> })
            }}

        </Suspense>
    }
}

#[component]
fn CanvasLoading() -> impl IntoView {
    view! { <div class="text-center pt-4">"Setting up canvas"</div> }
}

#[component]
fn CanvasView(
    context_menu_container_ok: Rc<menu::Menu>,
    context_menu_asset: Rc<menu::Menu>,
) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
    let workspace_state = expect_context::<state::Workspace>();

    let portal_ref = create_node_ref();
    let (container_height, set_container_height) = create_signal(0);
    // let container_resize_observer =
    //     Closure::<dyn Fn(wasm_bindgen::JsValue)>::new(move |entries: wasm_bindgen::JsValue| {
    //         assert!(entries.is_array());
    //         let entries = entries.dyn_ref::<js_sys::Array>().unwrap();
    //         let height = entries
    //             .iter()
    //             .map(|entry| {
    //                 let entry = entry.dyn_ref::<web_sys::ResizeObserverEntry>().unwrap();
    //                 let border_box = entry.border_box_size().get(0); //.dyn_ref::<web_sys>();
    //                 let border_box = border_box.dyn_ref::<web_sys::ResizeObserverSize>().unwrap();
    //                 border_box.block_size() as usize
    //             })
    //             .max()
    //             .unwrap();

    //         let height = clamp(height, 0, MAX_CONTAINER_HEIGHT);
    //         set_container_height(height);
    //     });
    provide_context(ContextMenuContainerOk::new(context_menu_container_ok));
    provide_context(ContextMenuAsset::new(context_menu_asset));
    provide_context(PortalRef(portal_ref.clone()));
    provide_context(ContainerHeight(container_height));
    // provide_context(ContainerResizeObserver(
    //     web_sys::ResizeObserver::new(container_resize_observer.as_ref().unchecked_ref()).unwrap(),
    // ));
    // container_resize_observer.forget();

    create_effect(move |_| {
        let height = workspace_state.preview().with(|preview| {
            let mut height: usize = 0;
            if preview.assets {
                height += 3;
            }
            if preview.analyses {
                height += 3;
            }
            if preview.kind {
                height += 1;
            }
            if preview.description {
                height += 3;
            }
            if preview.tags {
                height += 1;
            }
            if preview.metadata {
                height += 5;
            }

            height * 24
        });

        let height = clamp(height, 0, MAX_CONTAINER_HEIGHT);
        set_container_height(height);
    });

    let (vb_x, set_vb_x) = create_signal(0);
    let (vb_y, set_vb_y) = create_signal(0);
    let (vb_width, set_vb_width) = create_signal(1000);
    let (vb_height, set_vb_height) = create_signal(1000);
    let (pan_drag, set_pan_drag) = create_signal(None);
    let (was_dragged, set_was_dragged) = create_signal(false);

    let mousedown = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary as i16 {
            set_pan_drag(Some((e.client_x(), e.client_y())));
        }
    };

    let mouseup = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary as i16 && pan_drag.with(|c| c.is_some()) {
            if !was_dragged() {
                workspace_graph_state.select_clear();
            }

            set_pan_drag(None);
            set_was_dragged(false);
        }
    };

    let mousemove = {
        let graph = graph.clone();
        move |e: MouseEvent| {
            if pan_drag.with(|c| c.is_some()) {
                assert_eq!(e.button(), types::MouseButton::Primary as i16);
                let (dx, dy) = pan_drag.with(|c| {
                    let (x, y) = c.unwrap();
                    (e.client_x() - x, e.client_y() - y)
                });

                if dx > 0 || dy > 0 {
                    set_was_dragged(true);
                }

                let x = vb_x() - dx;
                let y = vb_y() - dy;
                let x_max = (graph.root().subtree_width().get().get()
                    * (CONTAINER_WIDTH + PADDING_X_SIBLING)) as i32
                    - vb_width() / 2;
                let y_max = cmp::max(
                    (graph.root().subtree_height().get().get()
                        * (MAX_CONTAINER_HEIGHT + PADDING_Y_CHILDREN)) as i32
                        - vb_height() / 2,
                    0,
                );
                set_vb_x(clamp(
                    x,
                    -TryInto::<i32>::try_into(vb_width() / 2).unwrap(),
                    x_max.try_into().unwrap(),
                ));
                set_vb_y(clamp(
                    y,
                    -TryInto::<i32>::try_into(vb_height() / 2).unwrap(),
                    y_max.try_into().unwrap(),
                ));
                set_pan_drag(Some((e.client_x(), e.client_y())));
            }
        }
    };

    let mouseleave = move |e: MouseEvent| {
        if pan_drag.with(|c| c.is_some()) {
            assert_eq!(e.button(), types::MouseButton::Primary as i16);
            set_pan_drag(None);
        }
    };

    let wheel = move |e: WheelEvent| {
        let dy = e.delta_y();
        let zoom = if dy < 0.0 {
            ZOOM_FACTOR_IN
        } else if dy > 0.0 {
            ZOOM_FACTOR_OUT
        } else {
            return;
        };

        let width = (vb_width() as f32 * zoom).round() as usize;
        let height = (vb_height() as f32 * zoom).round() as usize;
        set_vb_width(clamp(
            width.try_into().unwrap(),
            VB_WIDTH_MIN.try_into().unwrap(),
            VB_WIDTH_MAX.try_into().unwrap(),
        ));
        set_vb_height(clamp(
            height.try_into().unwrap(),
            VB_HEIGHT_MIN.try_into().unwrap(),
            VB_HEIGHT_MAX.try_into().unwrap(),
        ));
    };

    view! {
        <div id="canvas">
            <svg
                on:mousedown=mousedown
                on:mouseup=mouseup
                on:mousemove=mousemove
                on:mouseleave=mouseleave
                on:wheel=wheel
                viewBox=move || {
                    format!("{} {} {} {}", vb_x.get(), vb_y.get(), vb_width.get(), vb_height.get())
                }

                class=("cursor-grabbing", move || pan_drag.with(|c| c.is_some()))
            >
                <Graph root=graph.root().clone() />
            </svg>

            <div ref=portal_ref></div>
        </div>
    }
}

#[component]
fn Graph(root: state::graph::Node) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let container_height = expect_context::<ContainerHeight>();
    // let container_resize_observer = expect_context::<ContainerResizeObserver>();
    let portal_ref = expect_context::<PortalRef>();
    let create_child_ref = NodeRef::<html::Dialog>::new();
    let container_node = NodeRef::<html::Div>::new();
    let create_child_dialog_show = move |e: MouseEvent| {
        if e.button() != types::MouseButton::Primary as i16 {
            return;
        }

        let dialog = create_child_ref.get().unwrap();
        dialog.show_modal().unwrap();
    };

    let children = graph.children(&root).unwrap().read_only();
    let siblings = {
        let graph = graph.clone();
        let root = root.clone();
        move || {
            graph
                .parent(&root)
                .map(|parent| parent.with(|parent| graph.children(parent).unwrap().read_only()))
        }
    };

    let width = {
        let root = root.clone();
        move || {
            root.subtree_width().with(|width| {
                width.get() * (CONTAINER_WIDTH + PADDING_X_SIBLING) - PADDING_X_SIBLING
            })
        }
    };

    let height = {
        let root = root.clone();
        move || {
            let tree_height = root.subtree_height().get();
            let height = tree_height.get() * (container_height() + PADDING_Y_CHILDREN)
                - PADDING_Y_CHILDREN
                + RADIUS_ADD_CHILD;

            cmp::max(height, 0)
        }
    };

    let x = {
        let root = root.clone();
        move || {
            let older_sibling_width = siblings()
                .map(|siblings| {
                    siblings.with(|siblings| {
                        root.sibling_index().with(|index| {
                            siblings
                                .iter()
                                .take(*index)
                                .map(|sibling| sibling.subtree_width().get())
                                .reduce(|total, width| total.checked_add(width.get()).unwrap())
                                .map(|width| width.get())
                                .unwrap_or(0)
                        })
                    })
                })
                .unwrap_or(0);

            older_sibling_width * (CONTAINER_WIDTH + PADDING_X_SIBLING)
        }
    };

    let y = {
        let root = root.clone();
        move || {
            if state::graph::Node::ptr_eq(&root, graph.root()) {
                0
            } else {
                container_height.with(|height| *height + PADDING_Y_CHILDREN)
            }
        }
    };

    let x_node = {
        let width = width.clone();
        move || (width() - CONTAINER_WIDTH) / 2
    };

    fn x_child_offset(index: usize, children: &Vec<state::graph::Node>) -> usize {
        children
            .iter()
            .take(index)
            .map(|child| child.subtree_width().get())
            .reduce(|total, width| total.checked_add(width.get()).unwrap())
            .map(|width| width.get())
            .unwrap_or(0)
    }

    let line_points = {
        let x_node = x_node.clone();
        let children = children.clone();
        move |sibling_index: ReadSignal<usize>,
              subtree_width: ReadSignal<NonZeroUsize>,
              container_height: ReadSignal<usize>| {
            let x_node = x_node.clone();
            move || {
                let parent_x = x_node() + CONTAINER_WIDTH / 2;
                let parent_y = container_height();
                let midway_y = cmp::max(
                    container_height() as i32 + (PADDING_Y_CHILDREN / 2) as i32,
                    0,
                );
                let child_y = container_height().checked_add(PADDING_Y_CHILDREN).unwrap();
                let child_x_offset =
                    children.with(|children| x_child_offset(sibling_index.get(), children));
                let child_x = (child_x_offset + subtree_width.get().get() / 2)
                    * (CONTAINER_WIDTH + PADDING_X_SIBLING)
                    + CONTAINER_WIDTH / 2;
                format!(
                    "{},{} {},{} {},{} {},{}",
                    parent_x, parent_y, parent_x, midway_y, child_x, midway_y, child_x, child_y,
                )
            }
        }
    };

    let child_key = |child: &state::graph::Node| {
        child.properties().with(|properties| {
            properties
                .as_ref()
                .map(|properties| properties.rid().with(|rid| rid.to_string()))
                .unwrap_or_else(|_| {
                    todo!("use path as id");
                })
        })
    };

    // let _ = watch(
    //     move || container_node.get(),
    //     move |container_node, _, _| {
    //         if let Some(container_node) = container_node {
    //             let options = web_sys::ResizeObserverOptions::new();
    //             options.set_box(web_sys::ResizeObserverBoxOptions::ContentBox);
    //             container_resize_observer
    //                 .observe_with_options(container_node.dyn_ref().unwrap(), &options)
    //         }
    //     },
    //     true,
    // );

    const PLUS_SCALE: f32 = 0.7;
    view! {
        <svg width=width height=height x=x y=y>
            <g>
                <For each=children key=child_key let:child>
                    <polyline
                        fill="none"
                        class="stroke-secondary-400 dark:stroke-secondary-500"
                        points=line_points(
                            child.sibling_index(),
                            child.subtree_width(),
                            *container_height,
                        )
                    ></polyline>
                </For>

            </g>
            <g class="group">
                <foreignObject width=CONTAINER_WIDTH height=*container_height x=x_node.clone() y=0>
                    <ContainerView node_ref=container_node container=root.clone() />
                </foreignObject>
                <g class="group-[:not(:hover)]:hidden hover:cursor-pointer">
                    <circle
                        on:mousedown=create_child_dialog_show
                        cx={
                            let x_node = x_node.clone();
                            move || { x_node() + CONTAINER_WIDTH / 2 }
                        }

                        cy=move || container_height()
                        r=RADIUS_ADD_CHILD
                        class="stroke-black dark:stroke-white fill-white dark:fill-secondary-700 stroke-2"
                    ></circle>
                    <line
                        x1={
                            let x_node = x_node.clone();
                            move || {
                                (x_node() + CONTAINER_WIDTH / 2) as f32
                                    - RADIUS_ADD_CHILD as f32 * PLUS_SCALE
                            }
                        }

                        x2={
                            let x_node = x_node.clone();
                            move || {
                                (x_node() + CONTAINER_WIDTH / 2) as f32
                                    + RADIUS_ADD_CHILD as f32 * PLUS_SCALE
                            }
                        }

                        y1=move || container_height()
                        y2=move || container_height()
                        class="stroke-black dark:stroke-white stroke-2 linecap-round"
                    ></line>
                    <line
                        x1={
                            let x_node = x_node.clone();
                            move || { x_node() + CONTAINER_WIDTH / 2 }
                        }

                        x2={
                            let x_node = x_node.clone();
                            move || { x_node() + CONTAINER_WIDTH / 2 }
                        }

                        y1=move || container_height() as f32 - RADIUS_ADD_CHILD as f32 * PLUS_SCALE
                        y2=move || container_height() as f32 + RADIUS_ADD_CHILD as f32 * PLUS_SCALE
                        class="stroke-black dark:stroke-white stroke-2 linecap-round"
                    ></line>
                </g>
            </g>
            <g>
                <For each=children key=child_key let:child>
                    <Graph root=child />
                </For>

            </g>
        </svg>

        {move || {
            if let Some(mount) = portal_ref.get() {
                let mount = (*mount).clone();
                view! {
                    <Portal mount clone:root>
                        <ModalDialog node_ref=create_child_ref clone:root>
                            <CreateChildContainer
                                parent=root.clone()
                                parent_ref=create_child_ref.clone()
                            />
                        </ModalDialog>
                    </Portal>
                }
                    .into_view()
            } else {
                ().into_view()
            }
        }}
    }
}

#[component]
fn CreateChildContainer(
    parent: state::graph::Node,
    parent_ref: NodeRef<html::Dialog>,
) -> impl IntoView {
    use syre_local::project::container;

    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let (name, set_name) = create_signal("".to_string());

    let create_child = create_action({
        move |name: &String| {
            let graph = graph.clone();
            let project = project.rid().clone();
            let parent = parent.clone();
            let name = name.clone();
            async move {
                let parent_path = graph.path(&parent).unwrap();
                let path = parent_path.join(name);
                match commands::graph::create_child(project.get_untracked(), path).await {
                    Ok(_id) => {
                        // TODO: Buffer id to ensure it is published in an update.
                        let dialog = parent_ref.get_untracked().unwrap();
                        dialog.close();
                        set_name("".to_string());
                        Ok(())
                    }
                    Err(err) => match err {
                        container::error::Build::Load | container::error::Build::NotADirectory => {
                            unreachable!()
                        }
                        container::error::Build::Save(err) => {
                            tracing::error!(?err);
                            Err("Could not save the container.")
                        }
                        container::error::Build::AlreadyResource => {
                            Err("Folder is already a resource.")
                        }
                    },
                }
            }
        }
    });

    let close = move |_| {
        let dialog = parent_ref.get().unwrap();
        dialog.close();
        set_name("".to_string());
    };

    view! {
        <div class="px-4 py-2 rounded bg-white dark:bg-secondary-900">
            <h1 class="text-center text-lg pb-2 dark:text-white">"Create a new child"</h1>
            <form on:submit=move |e| {
                e.prevent_default();
                create_child.dispatch(name())
            }>
                <div class="pb-2">
                    <input
                        placeholder="Name"
                        on:input=move |e| set_name(event_target_value(&e))
                        prop:value=name
                        class="input-simple"
                        minlength="1"
                        autofocus
                        required
                    />
                    {move || {
                        create_child
                            .value()
                            .with(|value| {
                                if let Some(Err(error)) = value {
                                    tracing::debug!(? error);
                                    let msg = "Something went wrong.";
                                    view! { <div>{msg}</div> }.into_view()
                                } else {
                                    ().into_view()
                                }
                            })
                    }}

                </div>
                <div class="flex gap-2">
                    <button disabled=create_child.pending() class="btn btn-primary">
                        "Create"
                    </button>
                    <button
                        type="button"
                        on:mousedown=close
                        disabled=create_child.pending()
                        class="btn btn-secondary"
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

#[component]
fn ContainerView(
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    container: state::graph::Node,
) -> impl IntoView {
    move || {
        container.properties().with(|properties| {
            if properties.is_ok() {
                view! { <ContainerOk node_ref container=container.clone() /> }
            } else {
                view! { <ContainerErr node_ref container=container.clone() /> }
            }
        })
    }
}

/// A container whose properties are valid.
/// The state of analyses and assets is unknown.
#[component]
fn ContainerOk(
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    container: state::graph::Node,
) -> impl IntoView {
    use super::workspace::{DragOverWorkspaceResource, WorkspaceResource};

    assert!(container
        .properties()
        .with_untracked(|properties| properties.is_ok()));

    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let messages = expect_context::<types::Messages>();
    let context_menu = expect_context::<ContextMenuContainerOk>();
    let context_menu_active_container =
        expect_context::<RwSignal<Option<ContextMenuActiveContainer>>>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
    let drag_over_workspace_resource = expect_context::<Signal<DragOverWorkspaceResource>>();
    let (drag_over, set_drag_over) = create_signal(0);
    provide_context(Container(container.clone()));

    let title = {
        let container = container.clone();
        move || {
            container
                .properties()
                .with(|properties| properties.as_ref().unwrap().name())
        }
    };

    let rid = {
        let container = container.clone();
        move || {
            container.properties().with(|properties| {
                let db::state::DataResource::Ok(properties) = properties else {
                    panic!("invalid container state");
                };

                properties.rid().with(|rid| rid.to_string())
            })
        }
    };

    let mousedown = {
        let container = container.clone();
        let workspace_graph_state = workspace_graph_state.clone();
        move |e: MouseEvent| {
            let button = e.button();
            if button == types::MouseButton::Primary as i16 {
                e.stop_propagation();
                container.properties().with(|properties| {
                    if let db::state::DataResource::Ok(properties) = properties {
                        properties.rid().with(|rid| {
                            let action = workspace_graph_state.selection().with(|selection| {
                                interpret_resource_selection_action(rid, &e, selection)
                            });
                            match action {
                                SelectionAction::Remove => {
                                    workspace_graph_state.select_remove(&rid)
                                }
                                SelectionAction::Add => workspace_graph_state.select_add(
                                    rid.clone(),
                                    state::workspace_graph::ResourceKind::Container,
                                ),
                                SelectionAction::SelectOnly => workspace_graph_state.select_only(
                                    rid.clone(),
                                    state::workspace_graph::ResourceKind::Container,
                                ),

                                SelectionAction::Clear => workspace_graph_state.select_clear(),
                            }
                        });
                    }
                });
            }
        }
    };

    let selected = {
        let container = container.clone();
        let workspace_graph_state = workspace_graph_state.clone();
        move || {
            container.properties().with(|properties| {
                if let db::state::DataResource::Ok(properties) = properties {
                    workspace_graph_state.selection().with(|selection| {
                        properties
                            .rid()
                            .with(|rid| selection.iter().any(|resource| resource.rid() == rid))
                    })
                } else {
                    false
                }
            })
        }
    };

    let highlight = {
        let container = container.clone();
        move || {
            let drag_over_workspace = drag_over_workspace_resource.with(|resource| {
                let Some(WorkspaceResource::Container(over_id)) = resource.as_ref() else {
                    return false;
                };

                container.properties().with(|properties| {
                    if let db::state::DataResource::Ok(properties) = properties {
                        return properties.rid().with(|rid| over_id == rid);
                    }

                    false
                })
            });

            selected() || drag_over() > 0 || drag_over_workspace
        }
    };

    let contextmenu = {
        let container = container.clone();
        move |e: MouseEvent| {
            e.prevent_default();

            context_menu_active_container.update(|active_container| {
                let _ = active_container.insert(container.clone().into());
            });

            let menu = context_menu.clone();
            spawn_local(async move {
                menu.popup().await.unwrap();
            });
        }
    };

    let drop = {
        let project = project.rid().read_only();
        let graph = graph.clone();
        let container = container.clone();
        let messages = messages.clone();
        move |e: DragEvent| {
            e.prevent_default();
            set_drag_over(0);

            let data = e.data_transfer().unwrap();
            let data = data.get_data(common::APPLICATION_JSON).unwrap();
            let Ok(action) = serde_json::from_str::<actions::container::Action>(&data) else {
                tracing::warn!("invalid action: `{}`", data);
                return;
            };
            match action {
                actions::container::Action::AddAnalysisAssociation(analysis) => {
                    handle_container_action_add_analysis_accociation(
                        analysis,
                        container.clone(),
                        &graph,
                        project.get_untracked(),
                        messages.clone(),
                    )
                }
            }
        }
    };

    view! {
        <div
            on:mousedown=mousedown
            on:contextmenu=contextmenu
            on:dragenter=move |_| set_drag_over.update(|count| *count += 1)
            on:dragover=move |e| e.prevent_default()
            on:dragleave=move |_| set_drag_over.update(|count| *count -= 1)
            on:drop=drop
            class="h-full cursor-pointer rounded pt-2 pb-4 border-secondary-900 dark:border-secondary-100 bg-white dark:bg-secondary-700"
            class=(
                "border-2",
                {
                    let highlight = highlight.clone();
                    move || !highlight()
                },
            )

            class=(
                ["border-4", "border-primary-400", "dark:border-primary-700"],
                {
                    let highlight = highlight.clone();
                    move || highlight()
                },
            )

            data-resource=DATA_KEY_CONTAINER
            data-rid=rid
        >
            // NB: inner div with node ref is used for resizing observer to obtain content height.
            <div ref=node_ref>
                <div class="pb-2 text-center text-lg">
                    <span class="font-primary">{title}</span>
                </div>

                <div>
                    <ContainerPreview
                        properties=container.properties().read_only()
                        assets=container.assets().read_only()
                        analyses=container.analyses().read_only()
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ContainerPreview(
    properties: ReadSignal<state::container::PropertiesState>,
    analyses: ReadSignal<state::container::AnalysesState>,
    assets: ReadSignal<state::container::AssetsState>,
) -> impl IntoView {
    assert!(properties.with_untracked(|properties| properties.is_ok()));
    assert!(analyses.with_untracked(|analyses| analyses.is_ok()));
    let workspace_state = expect_context::<state::Workspace>();
    let state = workspace_state.preview().clone();

    let kind =
        properties.with_untracked(|properties| properties.as_ref().unwrap().kind().read_only());

    let description = properties
        .with_untracked(|properties| properties.as_ref().unwrap().description().read_only());

    let tags =
        properties.with_untracked(|properties| properties.as_ref().unwrap().tags().read_only());

    let metadata =
        properties.with_untracked(|properties| properties.as_ref().unwrap().metadata().read_only());

    view! {
        <div class="overflow-y-auto scrollbar">
            <Assets assets />

            <Analyses analyses=analyses
                .with_untracked(|analyses| analyses.as_ref().unwrap().read_only()) />

            <div>
                <div class:hidden=move || { state.with(|preview| !preview.kind) } class="px-2">
                    {move || kind().unwrap_or("(no type)".to_string())}
                </div>

                <div
                    class:hidden=move || { state.with(|preview| !preview.description) }
                    class="px-2"
                >
                    {move || description().unwrap_or("(no description)".to_string())}
                </div>

                <div class:hidden=move || { state.with(|preview| !preview.tags) } class="px-2">
                    {move || {
                        tags.with(|tags| {
                            if tags.is_empty() { "(no tags)".to_string() } else { tags.join(", ") }
                        })
                    }}

                </div>

                <Metadata metadata />
            </div>
        </div>
    }
}

#[component]
fn Assets(assets: ReadSignal<state::container::AssetsState>) -> impl IntoView {
    move || {
        assets.with(|assets| match assets {
            Ok(assets) => view! { <AssetsPreview assets=assets.read_only() /> }.into_view(),
            Err(err) => "(error)".into_view(),
        })
    }
}

#[component]
fn AssetsPreview(assets: ReadSignal<Vec<state::Asset>>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    view! {
        <div
            class:hidden=move || workspace_state.preview().with(|preview| !preview.assets)
            class="pb"
        >
            <Show
                when=move || assets.with(|assets| !assets.is_empty())
                fallback=|| view! { "(no data)" }
            >
                <For each=assets key=|asset| asset.rid().get() let:asset>
                    <Asset asset />
                </For>
            </Show>
        </div>
    }
}

#[component]
fn Asset(asset: state::Asset) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let container = expect_context::<Container>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
    let context_menu = expect_context::<ContextMenuAsset>();
    let context_menu_active_asset = expect_context::<RwSignal<Option<ContextMenuActiveAsset>>>();
    let messages = expect_context::<types::Messages>();

    let rid = {
        let rid = asset.rid();
        move || rid.with(|rid| rid.to_string())
    };

    let title = asset_title_closure(&asset);

    let mousedown = {
        let workspace_graph_state = workspace_graph_state.clone();
        let rid = asset.rid();
        move |e: MouseEvent| {
            if e.button() == types::MouseButton::Primary as i16 {
                e.stop_propagation();
                rid.with(|rid| {
                    let action = workspace_graph_state
                        .selection()
                        .with(|selection| interpret_resource_selection_action(rid, &e, selection));
                    match action {
                        SelectionAction::Remove => workspace_graph_state.select_remove(&rid),
                        SelectionAction::Add => workspace_graph_state
                            .select_add(rid.clone(), state::workspace_graph::ResourceKind::Asset),
                        SelectionAction::SelectOnly => workspace_graph_state
                            .select_only(rid.clone(), state::workspace_graph::ResourceKind::Asset),
                        SelectionAction::Clear => workspace_graph_state.select_clear(),
                    }
                });
            }
        }
    };

    let selected = {
        let asset = asset.clone();
        let workspace_graph_state = workspace_graph_state.clone();
        move || {
            workspace_graph_state.selection().with(|selection| {
                asset
                    .rid()
                    .with(|rid| selection.iter().any(|resource| resource.rid() == rid))
            })
        }
    };

    let contextmenu = {
        let asset = asset.clone();
        move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();

            context_menu_active_asset.update(|active_asset| {
                let _ = active_asset.insert(asset.rid().get_untracked().into());
            });

            let menu = context_menu.clone();
            spawn_local(async move {
                menu.popup().await.unwrap();
            });
        }
    };

    let remove = create_action({
        let asset = asset.clone();
        let container = container.clone();
        let graph = graph.clone();
        let project = project.rid().read_only();
        let messages = messages.clone();

        move |_| {
            let asset = asset.clone();
            let container = container.clone();
            let graph = graph.clone();
            let project = project.clone();
            let messages = messages.clone();

            async move {
                let container_path = graph.path(&container).unwrap();
                if let Err(err) = remove_asset(
                    project.get_untracked(),
                    container_path,
                    asset.path().get_untracked(),
                )
                .await
                {
                    tracing::error!(?err);
                    let mut msg = Message::error("Could not remove asset file");
                    msg.body(format!("{err:?}"));
                    messages.update(|messages| messages.push(msg.build()));
                };
            }
        }
    });

    let remove_asset = move |e: MouseEvent| {
        if e.button() != types::MouseButton::Primary as i16 {
            return;
        }

        e.stop_propagation();
        remove.dispatch(())
    };

    view! {
        <div
            on:mousedown=mousedown
            on:contextmenu=contextmenu
            title=asset_title_closure(&asset)
            class=("bg-secondary-400", selected)
            class="flex cursor-pointer px-2 py-0.5 border rounded-sm border-transparent hover:border-secondary-400"
            data-resource=DATA_KEY_ASSET
            data-rid=rid
        >
            <TruncateLeft class="grow">{title}</TruncateLeft>
            <button
                on:mousedown=remove_asset
                class="aspect-square h-full rounded-sm hover:bg-secondary-200 dark:hover:bg-secondary-700"
            >
                <Icon icon=icondata::AiMinusOutlined />
            </button>
        </div>
    }
}

#[component]
fn Analyses(analyses: ReadSignal<Vec<state::AnalysisAssociation>>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();

    view! {
        <div
            class:hidden=move || workspace_state.preview().with(|preview| !preview.analyses)
            class="pb"
        >
            <Show
                when=move || analyses.with(|analyses| !analyses.is_empty())
                fallback=|| view! { "(no analyses)" }
            >
                <For each=analyses key=|association| association.analysis().clone() let:association>
                    <AnalysisAssociation association />
                </For>
            </Show>
        </div>
    }
}

#[component]
fn AnalysisAssociation(association: state::AnalysisAssociation) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let container = expect_context::<Container>();
    let messages = expect_context::<types::Messages>();

    let title = {
        let association = association.clone();
        let project = project.clone();
        move || {
            project.analyses().with(|analyses| {
                let db::state::DataResource::Ok(analyses) = analyses else {
                    return None;
                };

                analyses.with(|analyses| {
                    analyses.iter().find_map(|analysis| {
                        analysis.properties().with(|properties| {
                            if properties.id() != association.analysis() {
                                return None;
                            }

                            let title = match properties {
                                local::types::AnalysisKind::Script(script) => {
                                    if let Some(name) = script.name.as_ref() {
                                        name.clone()
                                    } else {
                                        script.path.to_string_lossy().to_string()
                                    }
                                }

                                local::types::AnalysisKind::ExcelTemplate(template) => {
                                    if let Some(name) = template.name.as_ref() {
                                        name.clone()
                                    } else {
                                        template.template.path.to_string_lossy().to_string()
                                    }
                                }
                            };

                            Some(title)
                        })
                    })
                })
            })
        }
    };

    let hover_title = {
        let association = association.clone();
        let title = title.clone();
        move || {
            if title().is_none() {
                Some(association.analysis().to_string())
            } else {
                None
            }
        }
    };

    let update_associations = create_action({
        let project = project.clone();
        let container = container.clone();
        let messages = messages.clone();
        move |associations: &Vec<AnalysisAssociation>| {
            let project = project.rid().get_untracked();
            let container_path = graph.path(&container).unwrap();
            let messages = messages.clone();
            let associations = associations.clone();
            async move {
                if let Err(err) = commands::container::update_analysis_associations(
                    project,
                    container_path,
                    associations,
                )
                .await
                {
                    tracing::error!(?err);
                    let mut msg = Message::error("Could not update analysis association.");
                    msg.body(format!("{err:?}"));
                    messages.update(|messages| messages.push(msg.build()));
                }
            }
        }
    });

    let autorun_toggle = {
        let association = association.clone();
        let container = container.clone();

        move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary as i16 {
                return;
            }
            e.stop_propagation();

            let mut associations = container.analyses().with_untracked(|analyses| {
                analyses.as_ref().unwrap().with_untracked(|associations| {
                    associations
                        .iter()
                        .map(|association| association.as_association())
                        .collect::<Vec<_>>()
                })
            });
            let assoc = associations
                .iter_mut()
                .find(|analysis| analysis.analysis() == association.analysis())
                .unwrap();
            assoc.autorun = !assoc.autorun;

            update_associations.dispatch(associations);
        }
    };

    let remove_association = {
        let association = association.clone();
        let container = container.clone();

        move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary as i16 {
                return;
            }
            e.stop_propagation();

            let mut associations = container.analyses().with_untracked(|analyses| {
                analyses.as_ref().unwrap().with_untracked(|associations| {
                    associations
                        .iter()
                        .map(|association| association.as_association())
                        .collect::<Vec<_>>()
                })
            });
            associations.retain(|assoc| assoc.analysis() != association.analysis());

            update_associations.dispatch(associations);
        }
    };

    view! {
        <div class="flex px-2">
            <div title=hover_title class="grow">
                {move || title().unwrap_or("(no title)".to_string())}
            </div>
            <div class="inline-flex gap-1">
                <span>"(" {association.priority()} ")"</span>
                <span on:mousedown=autorun_toggle class="inline-flex">
                    {move || {
                        if association.autorun().get() {
                            view! { <Icon icon=icondata::BsStarFill /> }
                        } else {
                            view! { <Icon icon=icondata::BsStar /> }
                        }
                    }}

                </span>
            </div>
            <div>
                <button
                    on:mousedown=remove_association
                    class="aspect-square h-full rounded-sm hover:bg-secondary-200 dark:hover:bg-secondary-700"
                >
                    <Icon icon=icondata::AiMinusOutlined />
                </button>
            </div>
        </div>
    }
}

#[component]
fn Metadata(metadata: ReadSignal<state::Metadata>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    view! {
        <div class:hidden=move || { workspace_state.preview().with(|preview| !preview.metadata) }>
            <Show
                when=move || metadata.with(|metadata| !metadata.is_empty())
                fallback=|| view! { <div class="px-2">"(no metadata)"</div> }
            >
                <For each=metadata key=|(key, _)| key.clone() let:datum>
                    <div class="px-2">
                        <span>{datum.0} ": "</span>
                        <span>{move || datum.1.with(|value| serde_json::to_string(value))}</span>
                    </div>
                </For>
            </Show>
        </div>
    }
}

#[component]
fn ContainerErr(
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    container: state::graph::Node,
) -> impl IntoView {
    assert!(container
        .properties()
        .with_untracked(|properties| properties.is_err()));

    view! {
        <div ref=node_ref data-resource=DATA_KEY_CONTAINER>
            <div>
                <span>{container.name().with(|name| name.to_string_lossy().to_string())}</span>
            </div>

            <div>
                <div>"Error"</div>
            </div>
        </div>
    }
}

fn handle_container_action_add_analysis_accociation(
    analysis: ResourceId,
    container: state::graph::Node,
    graph: &state::Graph,
    project: ResourceId,
    messages: types::Messages,
) {
    let associations = container.analyses().read_only();
    let Some(mut associations) = associations.with_untracked(|associations| {
        let db::state::DataResource::Ok(associations) = associations else {
            panic!("invalid state");
        };

        if associations.with(|associations| {
            associations
                .iter()
                .any(|association| *association.analysis() == analysis)
        }) {
            None
        } else {
            Some(
                associations
                    .get_untracked()
                    .into_iter()
                    .map(|assoc| assoc.as_association())
                    .collect::<Vec<_>>(),
            )
        }
    }) else {
        return;
    };
    associations.push(AnalysisAssociation::new(analysis));

    let project = project.clone();
    let container = graph.path(&container).unwrap();
    spawn_local(async move {
        if let Err(err) =
            commands::container::update_analysis_associations(project, container, associations)
                .await
        {
            tracing::error!(?err);
            let mut msg = Message::error("Could not save container.");
            msg.body(format!("{err:?}"));
            messages.update(|messages| messages.push(msg.build()));
        }
    });
}

fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    assert!(min < max);
    if value <= min {
        min
    } else if value >= max {
        max
    } else {
        value
    }
}

async fn handle_context_menu_container_events(
    project: state::Project,
    graph: state::Graph,
    messages: types::Messages,
    context_menu_active_container: ReadSignal<Option<ContextMenuActiveContainer>>,
    container_open: Channel<String>,
    container_duplicate: Channel<String>,
    container_trash: Channel<String>,
) {
    let mut container_open = container_open.fuse();
    let mut container_duplicate = container_duplicate.fuse();
    let mut container_trash = container_trash.fuse();
    loop {
        futures::select! {
            event = container_open.next() => match event {
                None => continue,
                Some(_id) => {
                    let data_root = project
                        .path()
                        .get_untracked()
                        .join(project.properties().data_root().get_untracked());

                    let container = context_menu_active_container.get_untracked().unwrap();
                    let container_path = graph.path(&container).unwrap();
                    let path = common::container_system_path(data_root, container_path);

                    if let Err(err) = commands::fs::open_file(path)
                        .await {
                            messages.update(|messages|{
                                let mut msg = Message::error("Could not open container folder.");
                                msg.body(format!("{err:?}"));
                                messages.push(msg.build());
                        });
                    }
                }
            },

            event = container_duplicate.next() => match event {
                None => continue,
                Some(_id) => {
                    let container = context_menu_active_container.get_untracked().unwrap();
                    let container_path = graph.path(&container).unwrap();
                    let path = common::normalize_path_sep(container_path);
                    let project_id = project.rid().get_untracked();
                    if let Err(err) =  duplicate_container(project_id, path).await {
                        messages.update(|messages|{
                            let mut msg = Message::error("Could not duplicate container.");
                            msg.body(format!("{err:?}"));
                            messages.push(msg.build());
                        });
                    }
                }
            },

            event = container_trash.next() => match event {
                None => continue,
                Some(_id) => {
                    let container = context_menu_active_container.get_untracked().unwrap();
                    let container_path = graph.path(&container).unwrap();
                    let path = common::normalize_path_sep(container_path);
                    let project_id = project.rid().get_untracked();
                    if let Err(err) =  trash_container(project_id, path).await {
                            messages.update(|messages|{
                                let mut msg = Message::error("Could not trash container.");
                                msg.body(format!("{err:?}"));
                                messages.push(msg.build());
                            });
                        }
                }
            }
        }
    }
}

async fn handle_context_menu_asset_events(
    project: state::Project,
    graph: state::Graph,
    messages: types::Messages,
    context_menu_active_asset: ReadSignal<Option<ContextMenuActiveAsset>>,
    asset_open: Channel<String>,
) {
    let mut asset_open = asset_open.fuse();
    loop {
        futures::select! {
            event = asset_open.next() => match event {
                None => continue,
                Some(_id) => {
                    let data_root = project
                        .path()
                        .get_untracked()
                        .join(project.properties().data_root().get_untracked());

                    let asset = context_menu_active_asset.get_untracked().unwrap();
                    let container = graph.find_by_asset_id(&*asset).unwrap();
                    let container_path = graph.path(&container).unwrap();
                    let container_path = common::container_system_path(data_root, container_path);
                    let db::state::DataResource::Ok(assets) = container.assets().get_untracked() else {
                        panic!("invalid state");
                    };
                    let asset_path = assets.with_untracked(|assets| assets.iter().find_map(|container_asset| {
                         container_asset.rid().with_untracked(|rid| if *rid == *asset {
                            Some(container_asset.path().get_untracked())
                        } else {
                            None
                        })
                    })).unwrap();
                    let path = container_path.join(asset_path);

                    if let Err(err) = commands::fs::open_file(path)
                        .await {
                            messages.update(|messages|{
                                let mut msg = Message::error("Could not open asset file.");
                                msg.body(format!("{err:?}"));
                            messages.push(msg.build());
                        });
                    }
            }
            }
        }
    }
}

async fn duplicate_container(project: ResourceId, container: PathBuf) -> Result<(), io::ErrorKind> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
    }

    tauri_sys::core::invoke_result::<(), lib::command::error::IoErrorKind>(
        "container_duplicate",
        Args { project, container },
    )
    .await
    .map_err(|err| err.into())
}

async fn trash_container(project: ResourceId, container: PathBuf) -> Result<(), io::ErrorKind> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
    }

    tauri_sys::core::invoke_result::<(), lib::command::error::IoErrorKind>(
        "container_trash",
        Args { project, container },
    )
    .await
    .map_err(|err| err.into())
}

async fn remove_asset(
    project: ResourceId,
    container: PathBuf,
    asset: PathBuf,
) -> Result<(), io::ErrorKind> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
        asset: PathBuf,
    }

    tauri_sys::core::invoke_result::<(), lib::command::error::IoErrorKind>(
        "asset_remove_file",
        Args {
            project,
            container,
            asset,
        },
    )
    .await
    .map_err(|err| err.into())
}
