use super::state;
use crate::{commands, components::ModalDialog, invoke::invoke_result};
use ev::SubmitEvent;
use leptos::*;
use std::{cmp, ops::Deref, path::PathBuf};
use syre_local_database as db;

const CONTAINER_WIDTH: usize = 100;
const CONTAINER_HEIGHT: usize = 60;
const PADDING_X_SIBLING: usize = 20;
const PADDING_Y_CHILDREN: usize = 20;
const RADIUS_ADD_CHILD: usize = 10;

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

    let (tree_width, set_tree_width) = create_signal(vec![1]);
    let (tree_height, set_tree_height) = create_signal(vec![1]);

    view! {
        <div>
            <svg viewBox="0 0 1000 1000">
                <Graph
                    root=graph.root().clone()
                    index=0
                    set_width=set_tree_width
                    set_height=set_tree_height
                />
            </svg>

            <div ref=portal_ref></div>
        </div>
    }
}

#[component]
fn Graph(
    root: state::graph::Node,
    index: usize,
    set_width: WriteSignal<Vec<usize>>,
    set_height: WriteSignal<Vec<usize>>,
) -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let portal_ref = expect_context::<PortalRef>();
    let create_child_ref = NodeRef::<html::Dialog>::new();

    let children = graph.children(&root).unwrap().read_only();
    let (descendants_width, set_descendants_width) =
        create_signal(children.with_untracked(|children| vec![1, children.len()]));

    let (descendants_height, set_descendants_height) =
        create_signal(children.with_untracked(|children| vec![1, children.len()]));

    let create_child_dialog_show = move |_| {
        let dialog = create_child_ref.get().unwrap();
        dialog.show_modal().unwrap();
    };

    let width = move || {
        let width = cmp::max(descendants_width.with(|children| children.iter().sum()), 1);
        set_width.update(|siblings| siblings[index] = width);
        width * (CONTAINER_WIDTH + PADDING_X_SIBLING) - PADDING_X_SIBLING
    };

    let height = move || {
        let height = descendants_height
            .with(|children| children.iter().max().cloned())
            .unwrap_or(0)
            + 1;

        set_height.update(|siblings| siblings[index] = height);
        height * (CONTAINER_HEIGHT + PADDING_Y_CHILDREN) - PADDING_Y_CHILDREN
    };

    view! {
        <svg
            width=width
            height=height
            x=index * (CONTAINER_WIDTH + PADDING_X_SIBLING)
            y=CONTAINER_HEIGHT + PADDING_Y_CHILDREN
        >
            <g>
                <foreignObject width=CONTAINER_WIDTH height=CONTAINER_HEIGHT x=0 y=0>
                    <Container container=root.clone()/>
                </foreignObject>
                <circle
                    cx=CONTAINER_WIDTH / 2
                    cy=CONTAINER_HEIGHT - RADIUS_ADD_CHILD
                    r=RADIUS_ADD_CHILD
                    on:mousedown=create_child_dialog_show
                ></circle>
            </g>
            <g></g>
            <g>
                <For
                    each=move || {
                        children
                            .with(|children| {
                                children.iter().cloned().enumerate().collect::<Vec<_>>()
                            })
                    }

                    key=|(_, child)| {
                        child
                            .properties()
                            .with(|properties| {
                                properties
                                    .as_ref()
                                    .map(|properties| properties.rid().with(|rid| rid.to_string()))
                                    .unwrap_or_else(|_| {
                                        child.name().with(|name| name.to_string_lossy().to_string())
                                    })
                            })
                    }

                    let:child
                >
                    <Graph
                        root=child.1
                        index=child.0
                        set_width=set_descendants_width
                        set_height=set_descendants_height
                    />
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

    view! {
        <div>
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
