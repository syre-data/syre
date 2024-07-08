use super::state;
use leptos::*;

mod container;

#[component]
pub fn PropertiesBar() -> impl IntoView {
    use state::workspace_graph::ResourceKind;

    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();

    move || {
        workspace_graph_state
            .selection()
            .with(|selection| match &selection[..] {
                [] => view! {
                    "Properties"
                },
                [resource] => match resource.kind() {
                    ResourceKind::Container => {
                        view! {"single container"}
                    }
                    ResourceKind::Asset => {
                        view! {"single asset"}
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
                            ResourceKind::Container => view! {
                                "Bulk container"
                            },
                            ResourceKind::Asset => view! {
                                "Bulk asset"
                            },
                        },
                        _ => {
                            view! {
                                "Bulk mixed"
                            }
                        }
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
