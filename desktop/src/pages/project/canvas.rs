use super::state;
use crate::{commands, components::ModalDialog, invoke::invoke_result, types};

use leptos::{
    ev::{MouseEvent, SubmitEvent, WheelEvent},
    *,
};
use leptos_icons::*;
use std::{cmp, ops::Deref, path::PathBuf};
use syre_local_database as db;

const CONTAINER_WIDTH: usize = 100;
const CONTAINER_HEIGHT: usize = 60;
const PADDING_X_SIBLING: usize = 20;
const PADDING_Y_CHILDREN: usize = 20;
const PADDING_X_KEBAB: usize = 10;
const PADDING_Y_KEBAB: usize = 5;
const RADIUS_ADD_CHILD: usize = 10;
const ZOOM_FACTOR_IN: f32 = 0.9; // zoom in should reduce viewport.
const ZOOM_FACTOR_OUT: f32 = 1.1;
const VB_WIDTH_MIN: usize = 500;
const VB_WIDTH_MAX: usize = 10_000;
const VB_HEIGHT_MIN: usize = 500;
const VB_HEIGHT_MAX: usize = 10_000;

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
    let graph = expect_context::<state::Graph>();
    let portal_ref = create_node_ref();
    provide_context(PortalRef(portal_ref.clone()));
    let (vb_x, set_vb_x) = create_signal(0);
    let (vb_y, set_vb_y) = create_signal(0);
    let (vb_width, set_vb_width) = create_signal(1000);
    let (vb_height, set_vb_height) = create_signal(1000);
    let (pan_drag, set_pan_drag) = create_signal(None);

    let mousedown = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary as i16 {
            set_pan_drag(Some((e.client_x(), e.client_y())));
        }
    };

    let mouseup = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary as i16 && pan_drag.with(|c| c.is_some()) {
            set_pan_drag(None);
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

                let x = vb_x() - dx;
                let y = vb_y() - dy;
                let x_max = (graph.root().subtree_width().get()
                    * (CONTAINER_WIDTH + PADDING_X_SIBLING)) as i32
                    - vb_width() / 2;
                let y_max = cmp::max(
                    (graph.root().subtree_height().get() * (CONTAINER_HEIGHT + PADDING_Y_CHILDREN))
                        as i32
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
        <div>
            <svg
                viewBox=move || {
                    format!("{} {} {} {}", vb_x.get(), vb_y.get(), vb_width.get(), vb_height.get())
                }

                class=("cursor-grabbing", move || pan_drag.with(|c| c.is_some()))
                on:mousedown=mousedown
                on:mouseup=mouseup
                on:mousemove=mousemove
                on:mouseleave=mouseleave
                on:wheel=wheel
            >
                <Graph root=graph.root().clone()/>
            </svg>

            <div ref=portal_ref></div>
        </div>
    }
}

#[component]
fn Graph(root: state::graph::Node) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let portal_ref = expect_context::<PortalRef>();
    let create_child_ref = NodeRef::<html::Dialog>::new();
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
            root.subtree_width()
                .with(|width| width * (CONTAINER_WIDTH + PADDING_X_SIBLING) - PADDING_X_SIBLING)
        }
    };

    let height = {
        let root = root.clone();
        move || {
            root.subtree_height().with(|height| {
                height * (CONTAINER_HEIGHT + PADDING_Y_CHILDREN) - PADDING_Y_CHILDREN
            })
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
                                .sum::<usize>()
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
                CONTAINER_HEIGHT + PADDING_Y_CHILDREN
            }
        }
    };

    let x_node = {
        let width = width.clone();
        move || (width() - CONTAINER_WIDTH) / 2
    };

    let x_child_offset = {
        let children = children.clone();
        move |index: usize| -> usize {
            children.with(|children| {
                children
                    .iter()
                    .take(index)
                    .map(|child| child.subtree_width().get())
                    .sum::<usize>()
            })
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

    // NB [ RE: kebab menu]: It must currently be wrapped in an out `svg`
    // for placement.
    // See https://github.com/carloskiki/leptos-icons/issues/49.
    view! {
        <svg width=width height=height x=x y=y>
            <g class="group">
                <foreignObject width=CONTAINER_WIDTH height=CONTAINER_HEIGHT x=x_node.clone() y=0>
                    <Container container=root.clone()/>
                </foreignObject>
                <g class="group-[:not(:hover)]:hidden hover:cursor-pointer">
                    <svg
                        x={
                            let x_node = x_node.clone();
                            move || { x_node() + CONTAINER_WIDTH - PADDING_X_KEBAB }
                        }

                        y=PADDING_Y_KEBAB
                    >
                        <Icon icon=icondata::ChMenuKebab/>
                    </svg>

                    <circle
                        cx={
                            let x_node = x_node.clone();
                            move || { x_node() + CONTAINER_WIDTH / 2 }
                        }

                        cy=CONTAINER_HEIGHT - RADIUS_ADD_CHILD
                        r=RADIUS_ADD_CHILD
                        on:mousedown=create_child_dialog_show
                    ></circle>
                </g>
            </g>
            <g>
                <For each=children key=child_key let:child>
                    <polyline
                        fill="none"
                        stroke="black"
                        // TODO: Extract points function for aesthetics.
                        points={
                            let x_node = x_node.clone();
                            let x_child_offset = x_child_offset.clone();
                            move || {
                                let parent_x = x_node() + CONTAINER_WIDTH / 2;
                                let parent_y = CONTAINER_HEIGHT - RADIUS_ADD_CHILD;
                                let midway_y = CONTAINER_HEIGHT - RADIUS_ADD_CHILD
                                    + PADDING_Y_CHILDREN / 2;
                                let child_y = CONTAINER_HEIGHT + PADDING_Y_CHILDREN;
                                let child_x_offset = x_child_offset(child.sibling_index().get());
                                let child_x = (child_x_offset + child.subtree_width().get() / 2)
                                    * (CONTAINER_WIDTH + PADDING_X_SIBLING) + CONTAINER_WIDTH / 2;
                                format!(
                                    "{},{} {},{} {},{} {},{}",
                                    parent_x,
                                    parent_y,
                                    parent_x,
                                    midway_y,
                                    child_x,
                                    midway_y,
                                    child_x,
                                    child_y,
                                )
                            }
                        }
                    >
                    </polyline>
                </For>

            </g>
            <g>
                <For each=children key=child_key let:child>
                    <Graph root=child/>
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
        <div>
            <h1>"Create a new child"</h1>
            <form on:submit=move |e| {
                e.prevent_default();
                create_child.dispatch(name())
            }>
                <div>
                    <input
                        placeholder="Name"
                        on:input=move |e| set_name(event_target_value(&e))
                        prop:value=name
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
                <div>
                    <button disabled=create_child.pending()>"Create"</button>
                    <button type="button" on:mousedown=close disabled=create_child.pending()>
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

#[component]
fn Container(container: state::graph::Node) -> impl IntoView {
    move || {
        container.properties().with(|properties| {
            if properties.is_ok() {
                view! { <ContainerOk container=container.clone()/> }
            } else {
                view! { <ContainerErr container=container.clone()/> }
            }
        })
    }
}

#[component]
fn ContainerOk(container: state::graph::Node) -> impl IntoView {
    assert!(container
        .properties()
        .with_untracked(|properties| properties.is_ok()));

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
                if let db::state::DataResource::Ok(properties) = properties {
                    properties.rid().with(|rid| rid.to_string())
                } else {
                    "".to_string()
                }
            })
        }
    };

    view! {
        <div class="border-2" data-rid=rid>
            <div>
                <span>{title}</span>
            </div>

            <div>
                <ContainerPreview
                    properties=container.properties().read_only()
                    assets=container.assets().read_only()
                    analyses=container.analyses().read_only()
                />
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

    let kind = move || {
        properties.with(|properties| {
            properties
                .as_ref()
                .unwrap()
                .kind()
                .with(|kind| kind.clone().unwrap_or("(no type)".to_string()))
        })
    };

    let description = move || {
        properties.with(|properties| {
            properties
                .as_ref()
                .unwrap()
                .description()
                .with(|description| {
                    description
                        .clone()
                        .unwrap_or("(no description)".to_string())
                })
        })
    };

    let tags = move || {
        properties.with(|properties| {
            properties.as_ref().unwrap().tags().with(|tags| {
                if tags.is_empty() {
                    "(no tags)".to_string()
                } else {
                    tags.join(", ")
                }
            })
        })
    };

    let metadata =
        properties.with_untracked(|properties| properties.as_ref().unwrap().metadata().read_only());

    view! {
        <div>
            <Assets assets/>

            <Analyses analyses=analyses
                .with_untracked(|analyses| analyses.as_ref().unwrap().read_only())/>

            <div class:hidden=move || {
                workspace_state.preview.with(|preview| !preview.kind)
            }>{kind}</div>

            <div class:hidden=move || {
                workspace_state.preview.with(|preview| !preview.description)
            }>{description}</div>

            <div class:hidden=move || {
                workspace_state.preview.with(|preview| !preview.tags)
            }>{tags}</div>

            <Metadata metadata/>
        </div>
    }
}

#[component]
fn Assets(assets: ReadSignal<state::container::AssetsState>) -> impl IntoView {
    move || {
        assets.with(|assets| match assets {
            Ok(assets) => view! { <AssetsPreview assets=assets.read_only()/> }.into_view(),
            Err(err) => "(error)".into_view(),
        })
    }
}

#[component]
fn AssetsPreview(assets: ReadSignal<Vec<state::Asset>>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    view! {
        <div class:hidden=move || workspace_state.preview.with(|preview| !preview.assets)>
            <Show
                when=move || assets.with(|assets| !assets.is_empty())
                fallback=|| view! { "(no data)" }
            >
                <For each=assets key=|asset| asset.rid().get() let:asset>
                    <div>
                        <span>{asset.name()}</span>
                    </div>
                </For>
            </Show>
        </div>
    }
}

#[component]
fn Analyses(analyses: ReadSignal<Vec<state::AnalysisAssociation>>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    view! {
        <div class:hidden=move || workspace_state.preview.with(|preview| !preview.analyses)>
            <Show
                when=move || analyses.with(|analyses| !analyses.is_empty())
                fallback=|| view! { "(no analyses)" }
            >
                <For each=analyses key=|association| association.analysis().clone() let:association>
                    <div>
                        <span>{association.analysis().to_string()}</span>
                        <span>{association.autorun()}</span>
                        <span>{association.priority()}</span>
                    </div>
                </For>
            </Show>
        </div>
    }
}

#[component]
fn Metadata(metadata: ReadSignal<state::container::Metadata>) -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    view! {
        <div class:hidden=move || { workspace_state.preview.with(|preview| !preview.metadata) }>
            <Show
                when=move || metadata.with(|analyses| !analyses.is_empty())
                fallback=|| view! { "(no metadata)" }
            >
                <For each=metadata key=|(key, _)| key.clone() let:datum>
                    <div>
                        <span>{datum.0} ":"</span>
                        <span>{move || datum.1.with(|value| value.to_string())}</span>
                    </div>
                </For>
            </Show>
        </div>
    }
}

#[component]
fn ContainerErr(container: state::graph::Node) -> impl IntoView {
    assert!(container
        .properties()
        .with_untracked(|properties| properties.is_err()));

    view! {
        <div>
            <div>
                <span>{container.name().with(|name| name.to_string_lossy().to_string())}</span>
            </div>

            <div>
                <div>"Error"</div>
            </div>
        </div>
    }
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
