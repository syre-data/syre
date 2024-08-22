use super::state;
use leptos::*;

mod analyses;
mod asset;
mod asset_bulk;
mod common;
mod container;
mod container_bulk;
mod mixed_bulk;

use analyses::Editor as Analyses;
use asset::Editor as Asset;
use asset_bulk::Editor as AssetBulk;
use container::Editor as Container;
use container_bulk::Editor as ContainerBulk;
use mixed_bulk::Editor as MixedBulk;

/// Debounce time in milliseconds for editor input.
pub const INPUT_DEBOUNCE: f64 = 200.0;

/// Id for the analyses properties bar.
pub const ANALYSES_ID: &'static str = "analyses";

#[component]
pub fn PropertiesBar() -> impl IntoView {
    use state::workspace_graph::ResourceKind;

    let graph = expect_context::<state::Graph>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();

    move || {
        workspace_graph_state
            .selection()
            .with(|selection| match &selection[..] {
                [] => view! {
                    <div id=ANALYSES_ID class="h-full">
                        <Analyses/>
                    </div>
                }
                .into_view(),
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
                            ResourceKind::Container => {
                                let containers = {
                                    let selection = selection.clone();
                                    move || {
                                        selection
                                            .iter()
                                            .map(|resource| resource.rid().clone())
                                            .collect()
                                    }
                                };

                                view! { <ContainerBulk containers=Signal::derive(containers)/> }
                                    .into_view()
                            }
                            ResourceKind::Asset => {
                                let assets = {
                                    let selection = selection.clone();
                                    move || {
                                        selection
                                            .iter()
                                            .map(|resource| resource.rid().clone())
                                            .collect()
                                    }
                                };

                                view! { <AssetBulk assets=Signal::derive(assets)/> }.into_view()
                            }
                        },
                        _ => {
                            let resources = create_memo({
                                let workspace_graph_state = workspace_graph_state.clone();
                                move |_| workspace_graph_state.selection().get()
                            });
                            view! { <MixedBulk resources/> }.into_view()
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
