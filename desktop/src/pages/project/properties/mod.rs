use super::state;
use leptos::*;

mod asset;
mod common;
mod container;

use asset::Editor as Asset;
use container::Editor as Container;

/// Debounce time in milliseconds for editor input.
pub const INPUT_DEBOUNCE: f64 = 200.0;

#[component]
pub fn PropertiesBar() -> impl IntoView {
    use state::workspace_graph::ResourceKind;

    let graph = expect_context::<state::Graph>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();

    move || {
        workspace_graph_state
            .selection()
            .with(|selection| match &selection[..] {
                [] => view! { "Project properties" }.into_view(),
                [resource] => match resource.kind() {
                    ResourceKind::Container => {
                        let container = graph.find_by_id(resource.rid()).unwrap();
                        view! { <Container container=container.state().clone()/> }.into_view()
                    }
                    ResourceKind::Asset => {
                        let asset = graph.find_asset_by_id(resource.rid()).unwrap();
                        view! { <Asset asset/> }.into_view()
                    }
                },

                _ => {
                    let mut kinds = selection
                        .iter()
                        .map(|resource| resource.kind())
                        .collect::<Vec<_>>();
                    kinds.sort_by(|a, b| sort_resource_kind(a, b));
                    kinds.dedup();

                    match kinds[..] {
                        [] => panic!("invalid state"),
                        [kind] => match kind {
                            ResourceKind::Container => view! { "Bulk container" }.into_view(),
                            ResourceKind::Asset => view! { "Bulk asset" }.into_view(),
                        },
                        _ => view! { "Bulk mixed" }.into_view(),
                    }
                }
            })
    }
}

fn sort_resource_kind(
    a: &state::workspace_graph::ResourceKind,
    b: &state::workspace_graph::ResourceKind,
) -> std::cmp::Ordering {
    use state::workspace_graph::ResourceKind;
    use std::cmp::Ordering;

    match (a, b) {
        (ResourceKind::Container, ResourceKind::Asset) => Ordering::Less,

        (ResourceKind::Asset, ResourceKind::Container) => Ordering::Greater,

        (ResourceKind::Container, ResourceKind::Container)
        | (ResourceKind::Asset, ResourceKind::Asset) => Ordering::Equal,
    }
}
