use super::{
    state::{self, workspace_graph::SelectedResource},
    workspace,
};
use leptos::*;
use syre_core::types::ResourceId;

mod analyses;
mod asset;
mod asset_bulk;
mod common;
mod container;
mod container_bulk;
mod mixed_bulk;
mod project;

use analyses::Editor as Analyses;
use asset::Editor as Asset;
use asset_bulk::Editor as AssetBulk;
use container::Editor as Container;
use container_bulk::Editor as ContainerBulk;
use mixed_bulk::Editor as MixedBulk;
use project::Editor as Project;

/// Debounce time in milliseconds for editor input.
pub const INPUT_DEBOUNCE: f64 = 200.0;

/// Id for the analyses properties bar.
pub const ANALYSES_ID: &'static str = "analyses";

#[derive(Clone)]
pub enum EditorKind {
    Project,
    Analyses,
    Container(state::Container),
    Asset(state::Asset),
    ContainerBulk(Signal<Vec<ResourceId>>),
    AssetBulk(Signal<Vec<ResourceId>>),
    MixedBulk(Memo<Vec<SelectedResource>>),
}

impl Default for EditorKind {
    fn default() -> Self {
        Self::Analyses
    }
}

#[component]
pub fn PropertiesBar() -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
    let active_editor = expect_context::<RwSignal<workspace::PropertiesEditor>>();

    create_effect({
        let graph = graph.clone();
        move |_| {
            let editor_kind =
                active_editor_from_selection(workspace_graph_state.selection(), &graph);
            active_editor.set(editor_kind.into())
        }
    });

    move || {
        active_editor.with(|active_editor| match &**active_editor {
            EditorKind::Project => view! { <Project /> }.into_view(),
            EditorKind::Analyses => view! {
                <div id=ANALYSES_ID class="h-full">
                    <Analyses />
                </div>
            }
            .into_view(),
            EditorKind::Container(container) => {
                view! { <Container container=container.clone() /> }.into_view()
            }
            EditorKind::Asset(asset) => view! { <Asset asset=asset.clone() /> }.into_view(),
            EditorKind::ContainerBulk(containers) => {
                view! { <ContainerBulk containers=containers.clone() /> }.into_view()
            }
            EditorKind::AssetBulk(assets) => {
                view! { <AssetBulk assets=assets.clone() /> }.into_view()
            }
            EditorKind::MixedBulk(resources) => {
                view! { <MixedBulk resources=resources.clone() /> }.into_view()
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

fn active_editor_from_selection(
    selection: ReadSignal<Vec<SelectedResource>>,
    graph: &state::Graph,
) -> EditorKind {
    use state::workspace_graph::ResourceKind;

    selection.with(|selected| match &selected[..] {
        [] => EditorKind::Analyses,
        [resource] => match resource.kind() {
            ResourceKind::Container => {
                let container = graph.find_by_id(resource.rid()).unwrap();
                EditorKind::Container(container.state().clone())
            }
            ResourceKind::Asset => {
                let asset = graph.find_asset_by_id(resource.rid()).unwrap();
                EditorKind::Asset(asset)
            }
        },

        _ => {
            let mut kinds = selected
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
                            let selection = selected.clone();
                            move || {
                                selection
                                    .iter()
                                    .map(|resource| resource.rid().clone())
                                    .collect()
                            }
                        };

                        let containers = Signal::derive(containers);
                        EditorKind::ContainerBulk(containers)
                    }
                    ResourceKind::Asset => {
                        let assets = {
                            let selection = selected.clone();
                            move || {
                                selection
                                    .iter()
                                    .map(|resource| resource.rid().clone())
                                    .collect()
                            }
                        };

                        let assets = Signal::derive(assets);
                        EditorKind::AssetBulk(assets)
                    }
                },
                _ => {
                    let resources = create_memo(move |_| selection.get());
                    EditorKind::MixedBulk(resources)
                }
            }
        }
    })
}
