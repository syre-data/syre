use super::{
    common::{interpret_resource_selection_action, SelectionAction},
    state,
};
use crate::types;
use leptos::{ev::MouseEvent, *};
use syre_local_database as db;

#[component]
pub fn LayersNav() -> impl IntoView {
    let graph = expect_context::<state::Graph>();

    view! {
        <div>
            <ContainerLayer root=graph.root().clone()/>
        </div>
    }
}

#[component]
fn ContainerLayer(root: state::graph::Node, #[prop(optional)] depth: usize) -> impl IntoView {
    let graph = expect_context::<state::Graph>();

    view! {
        <div>

            {
                let root = root.clone();
                move || {
                    if root.properties().with(|properties| properties.is_ok()) {
                        view! { <ContainerLayerTitleOk container=root.clone() depth/> }
                    } else {
                        view! { <ContainerLayerTitleErr container=root.clone() depth/> }
                    }
                }
            }
            <div>
                <For
                    each={
                        let root = root.clone();
                        let graph = graph.clone();
                        move || graph.children(&root).unwrap().get()
                    }

                    key={
                        let graph = graph.clone();
                        move |child| graph.path(&child)
                    }

                    let:child
                >
                    <ContainerLayer root=child depth=depth + 1/>
                </For>
            </div>
        </div>
    }
}

#[component]
fn ContainerLayerTitleOk(container: state::graph::Node, depth: usize) -> impl IntoView {
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
    let properties = {
        let container = container.clone();
        move || {
            container.properties().with(|properties| {
                let db::state::DataResource::Ok(properties) = properties else {
                    panic!("invalid state");
                };

                properties.clone()
            })
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

    let title = {
        let properties = properties.clone();
        move || properties().name().get()
    };

    let mousedown = {
        let properties = properties.clone();
        move |e: MouseEvent| {
            if e.button() == types::MouseButton::Primary as i16 {
                e.stop_propagation();
                properties().rid().with(|rid| {
                    let action = workspace_graph_state
                        .selection()
                        .with(|selection| interpret_resource_selection_action(rid, &e, selection));
                    match action {
                        SelectionAction::Remove => workspace_graph_state.select_remove(&rid),
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
        }
    };

    view! {
        <div
            on:mousedown=mousedown
            style:padding-left=move || { depth_to_padding(depth) }

            class=(
                ["bg-slate-300"],
                {
                    let selected = selected.clone();
                    move || selected()
                },
            )
        >

            {title}
        </div>
    }
}

#[component]
fn ContainerLayerTitleErr(container: state::graph::Node, depth: usize) -> impl IntoView {
    let title = {
        let container = container.clone();
        move || {
            container
                .name()
                .with(|name| name.to_string_lossy().to_string())
        }
    };

    view! {
        <div style:padding-left=move || {
            depth_to_padding(depth)
        }>

            {title}
        </div>
    }
}

fn depth_to_padding(depth: usize) -> String {
    const LAYER_PADDING_SCALE: usize = 9;

    format!("{}px", depth * LAYER_PADDING_SCALE)
}
