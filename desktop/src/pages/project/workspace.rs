use super::{Settings, views};
use crate::commands;
use futures::stream::StreamExt;
use leptos::{
    either::{Either, either},
    prelude::*,
    task::spawn_local,
};
use leptos_router::{components::A, hooks::use_params_map};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_lib as lib;
use syre_desktop_ui_lib as ui_lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_project_watcher as db;

#[derive(Clone, Copy, derive_more::Deref, derive_more::From)]
struct ShowSettings(RwSignal<bool>);
impl ShowSettings {
    pub fn new() -> Self {
        Self(RwSignal::new(false))
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn Workspace() -> impl IntoView {
    let params = use_params_map();
    let id =
        move || params.with(|params| ResourceId::from_str(&params.get("id").unwrap()).unwrap());
    let active_user = LocalResource::new(commands::user::fetch_user);
    let resources = LocalResource::new(move || fetch_project_resources(id()));

    view! {
        <Suspense fallback=Loading>
            <ErrorBoundary fallback=|errors| {
                view! { <UserError errors /> }
            }>
                {move || Suspend::new(async move {
                    let user = active_user.await;
                    user.map(|user| match user {
                        None => Either::Left(view! { <NoUser /> }),
                        Some(user) => {
                            Either::Right(
                                view! {
                                    <Suspense fallback=Loading>
                                        {
                                            let user = user.clone();
                                            move || Suspend::new({
                                                let user = user.clone();
                                                async move {
                                                    let resources = resources.await;
                                                    resources
                                                        .map(|(project_path, project_data, graph)| {
                                                            Either::Left(
                                                                view! {
                                                                    <WorkspaceView user project_path project_data graph />
                                                                },
                                                            )
                                                        })
                                                        .unwrap_or(Either::Right(view! { <NoProject /> }))
                                                }
                                            })
                                        }

                                    </Suspense>
                                },
                            )
                        }
                    })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="pt-4 text-center">"Loading..."</div> }
}

#[component]
fn NoUser() -> impl IntoView {
    let messages = expect_context::<ui_lib::message::Messages>();
    let navigate = leptos_router::hooks::use_navigate();

    let msg = ui_lib::message::Builder::error("You are not logged in.");
    messages.push_message(msg.build());
    navigate("login", Default::default());

    view! {
        <div class="text-center">
            <p>"You are not logged in."</p>
            <p>"Taking you to the login page."</p>
        </div>
    }
}

#[component]
fn UserError(errors: ArcRwSignal<Errors>) -> impl IntoView {
    view! {
        <div class="text-center">
            <div class="text-large p4">"Error with user."</div>
            <div>{format!("{errors:?}")}</div>
        </div>
    }
}

#[component]
fn NoProject() -> impl IntoView {
    view! {
        <div>
            <div class="p-4 text-center">"Project state was not found."</div>
            <div class="text-center">
                <A href="/" attr:class="btn btn-primary">
                    "Dashboard"
                </A>
            </div>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn WorkspaceView(
    user: core::system::User,
    project_path: PathBuf,
    project_data: db::state::ProjectData,
    graph: db::state::FolderResource<db::state::Graph>,
) -> impl IntoView {
    assert!(project_data.properties().is_ok());

    let project = ui_lib::state::Project::new(project_path, project_data);
    provide_context(user);
    provide_context(ui_lib::state::Workspace::new());
    provide_context(project.clone());
    let user_settings = ui_lib::state::settings::User::new_store(lib::settings::User::default());
    let project_settings =
        ui_lib::state::settings::Project::new_store(lib::settings::Project::default());
    provide_context(user_settings);
    provide_context(project_settings);

    let show_settings = ShowSettings::new();
    provide_context(show_settings);

    spawn_local({
        let project = project.clone();
        async move {
            let rid = &project
                .rid()
                .with_untracked(|rid| lib::event::topic::graph(rid));
            let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(rid)
                .await
                .unwrap();

            while let Some(events) = listener.next().await {
                tracing::debug!(?events);
                for event in events.payload {
                    let lib::EventKind::Project(update) = event.kind() else {
                        panic!("invalid event kind");
                    };

                    match update {
                        db::event::Project::FolderRemoved
                        | db::event::Project::Moved(_)
                        | db::event::Project::Properties(_)
                        | db::event::Project::Settings(_)
                        | db::event::Project::Analyses(_)
                        | db::event::Project::AnalysisFile(_) => {
                            handle_event_project(event, project.clone())
                        }

                        db::event::Project::Graph(_)
                        | db::event::Project::Container { .. }
                        | db::event::Project::Asset { .. }
                        | db::event::Project::AssetFile(_) => continue, // handled elsewhere
                    }
                }
            }
        }
    });

    view! {
        <div class="select-none flex flex-col h-full relative">
            {move || {
                either!(
                    graph.as_ref(),
                    db::state::FolderResource::Absent => view! {
                        <project_nav::NoGraph />
                        <views::no_graph::Workspace />
                    },
                    db::state::FolderResource::Present(graph) => {
                        view! {
                            <project_nav::Graph/>
                            <WorkspaceGraph graph=graph.clone() />
                    }},
                )
            }}
            <div
                class=(["-right-full", "left-full"], move || !show_settings())
                class=(["right-0", "left-0"], move || show_settings())
                class="absolute top-0 bottom-0 transition-absolute-position z-20"
            >
                <Settings onclose=Callback::new(move |_| show_settings.set(false)) />
            </div>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn WorkspaceGraph(graph: db::state::Graph) -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let flags = ui_lib::state::Flags::new(&graph);
    let graph = ui_lib::state::Graph::new(graph);
    let workspace_graph_state = ui_lib::state::WorkspaceGraph::new(&graph);
    let display_state = ui_lib::state::Display::from(
        &graph,
        workspace_graph_state.container_visiblity().read_only(),
    );
    let data_view = RwSignal::new(ui_lib::types::DataView::default());
    let (db_loaded, set_db_loaded) = signal(false);

    provide_context(graph.clone());
    provide_context(flags);
    provide_context(workspace_graph_state.clone());
    provide_context(display_state.clone());
    provide_context(data_view);

    Effect::watch(
        data_view,
        move |view, _, _| {
            if !db_loaded.get_untracked() {
                if matches!(view, ui_lib::types::DataView::Database) {
                    set_db_loaded(true);
                }
            }
        },
        false,
    );

    spawn_local({
        let project = project.clone();
        let graph = graph.clone();
        let workspace_graph_state = workspace_graph_state.clone();
        async move {
            let rid = &project
                .rid()
                .with_untracked(|rid| lib::event::topic::graph(rid));
            let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(rid)
                .await
                .unwrap();

            while let Some(events) = listener.next().await {
                for event in events.payload {
                    let lib::EventKind::Project(update) = event.kind() else {
                        panic!("invalid event kind");
                    };

                    match update {
                        db::event::Project::FolderRemoved
                        | db::event::Project::Moved(_)
                        | db::event::Project::Properties(_)
                        | db::event::Project::Settings(_)
                        | db::event::Project::Analyses(_)
                        | db::event::Project::AnalysisFile(_) => continue, // handled elsewhere

                        db::event::Project::Graph(_)
                        | db::event::Project::Container { .. }
                        | db::event::Project::Asset { .. }
                        | db::event::Project::AssetFile(_) => handle_event_graph(
                            event,
                            graph.clone(),
                            workspace_graph_state.clone(),
                            display_state.clone(),
                            flags,
                            messages,
                        ),
                    }
                }
            }
        }
    });

    view! {
        <main class="grow min-h-0">
            <syre_desktop_workspace_graph::Workspace class:hidden=move || {
                !matches!(data_view(), ui_lib::types::DataView::Graph)
            } />
            {move || {
                db_loaded
                    .get()
                    .then_some(
                        view! {
                            <syre_desktop_workspace_db::Workspace class:hidden=move || {
                                !matches!(data_view(), ui_lib::types::DataView::Database)
                            } />
                        },
                    )
            }}
        </main>
    }
}

mod project_nav {
    use super::ShowSettings;
    use leptos::{ev::MouseEvent, prelude::*};
    use leptos_icons::Icon;
    use leptos_router::components::A;
    use syre_desktop_ui_components::Logo;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn NoGraph() -> impl IntoView {
        let show_settings = expect_context::<ShowSettings>();
        let open_settings = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            show_settings.set(true);
        };

        view! {
            <nav class="px-2 border-b dark:bg-secondary-900 flex items-center">
                <ol class="flex grow">
                    <li>
                        <A href="/" attr:title="Dashboard">
                            <Logo attr:class="h-4" />
                        </A>
                    </li>
                </ol>
                <ol>
                    <li>
                        <button
                            on:mousedown=open_settings
                            type="button"
                            class="align-middle p-1 hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded \
                            border border-transparent hover:border-black dark:hover:border-white"
                        >
                            <Icon icon=ui_lib::icon::Settings />
                        </button>
                    </li>
                </ol>
            </nav>
        }
    }

    #[component]
    pub fn Graph() -> impl IntoView {
        let show_settings = expect_context::<ShowSettings>();
        let open_settings = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            show_settings.set(true);
        };

        view! {
            <nav class="px-2 border-b dark:bg-secondary-900 flex items-center">
                <ol class="flex grow">
                    <li>
                        <A href="/" attr:title="Dashboard">
                            <Logo attr:class="h-4" />
                        </A>
                    </li>
                </ol>
                <ol class="flex gap-2">
                    <li>
                        <button
                            on:mousedown=open_settings
                            type="button"
                            class="align-middle p-1 hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded \
                            border border-transparent hover:border-black dark:hover:border-white cursor-pointer"
                            title="Settings"
                        >
                            <Icon icon=ui_lib::icon::Settings />
                        </button>
                    </li>
                </ol>
            </nav>
        }
    }
}

/// # Returns
/// Project's path, data, and graph.
async fn fetch_project_resources(
    project: ResourceId,
) -> Option<(
    PathBuf,
    db::state::ProjectData,
    db::state::FolderResource<db::state::Graph>,
)> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
    }

    let resources = tauri_sys::core::invoke::<
        Option<(
            PathBuf,
            db::state::ProjectData,
            db::state::FolderResource<db::state::Graph>,
        )>,
    >("project_resources", Args { project })
    .await;

    assert!(if let Some((_, data, _)) = resources.as_ref() {
        data.properties().is_ok()
    } else {
        true
    });

    resources
}

fn handle_event_project(event: lib::Event, project: ui_lib::state::Project) {
    let lib::EventKind::Project(update) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Project::Graph(_)
        | db::event::Project::Container { .. }
        | db::event::Project::Asset { .. }
        | db::event::Project::AssetFile(_) => unreachable!("handled elsewhere"),

        db::event::Project::FolderRemoved => todo!(),
        db::event::Project::Moved(_) => todo!(),
        db::event::Project::Properties(_) => handle_event_project_properties(event, project),
        db::event::Project::Settings(_) => todo!(),
        db::event::Project::Analyses(_) => handle_event_project_analyses(event, project),
        db::event::Project::AnalysisFile(_) => todo!(),
    }
}

fn handle_event_project_properties(event: lib::Event, project: ui_lib::state::Project) {
    let lib::EventKind::Project(db::event::Project::Properties(update)) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => todo!(),
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(io_serde) => todo!(),
        db::event::DataResource::Repaired(_) => todo!(),
        db::event::DataResource::Modified(_) => {
            handle_event_project_properties_modified(event, project)
        }
    }
}

fn handle_event_project_properties_modified(event: lib::Event, project: ui_lib::state::Project) {
    let lib::EventKind::Project(db::event::Project::Properties(db::event::DataResource::Modified(
        update,
    ))) = event.kind()
    else {
        panic!("invalid event kind");
    };

    if project
        .properties()
        .name()
        .with_untracked(|name| *name != update.name)
    {
        project.properties().name().set(update.name.clone());
    }

    if project
        .properties()
        .description()
        .with_untracked(|description| *description != update.description)
    {
        project
            .properties()
            .description()
            .set(update.description.clone());
    }

    if project
        .properties()
        .data_root()
        .with_untracked(|data_root| *data_root != update.data_root)
    {
        project
            .properties()
            .data_root()
            .set(update.data_root.clone());
    }

    if project
        .properties()
        .analysis_root()
        .with_untracked(|analysis_root| *analysis_root != update.analysis_root)
    {
        project
            .properties()
            .analysis_root()
            .set(update.analysis_root.clone());
    }
}

fn handle_event_project_analyses(event: lib::Event, project: ui_lib::state::Project) {
    let lib::EventKind::Project(db::event::Project::Analyses(update)) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => todo!(),
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => todo!(),
        db::event::DataResource::Repaired(_) => todo!(),
        db::event::DataResource::Modified(_) => {
            handle_event_project_analyses_modified(event, project)
        }
    }
}

fn handle_event_project_analyses_modified(event: lib::Event, project: ui_lib::state::Project) {
    let lib::EventKind::Project(db::event::Project::Analyses(db::event::DataResource::Modified(
        update,
    ))) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let analyses = project.analyses().with_untracked(|analyses| {
        let db::state::DataResource::Ok(analyses) = analyses else {
            panic!("invalid state");
        };

        analyses.clone()
    });

    analyses.update(|analyses| {
        analyses.retain(|analysis| {
            update.iter().any(|update_analysis| {
                analysis.properties().with_untracked(|properties| {
                    match (properties, update_analysis.properties()) {
                        (AnalysisKind::Script(properties), AnalysisKind::Script(update)) => {
                            properties.rid() == update.rid()
                        }

                        (
                            AnalysisKind::ExcelTemplate(properties),
                            AnalysisKind::ExcelTemplate(update),
                        ) => properties.rid() == update.rid(),

                        _ => false,
                    }
                })
            })
        });

        for update_analysis in update.iter() {
            if !analyses.iter().any(|analysis| {
                analysis.properties().with_untracked(|properties| {
                    match (properties, update_analysis.properties()) {
                        (AnalysisKind::Script(properties), AnalysisKind::Script(update)) => {
                            properties.rid() == update.rid()
                        }

                        (
                            AnalysisKind::ExcelTemplate(properties),
                            AnalysisKind::ExcelTemplate(update),
                        ) => properties.rid() == update.rid(),

                        _ => false,
                    }
                })
            }) {
                analyses.push(ui_lib::state::Analysis::from_state(update_analysis));
            }
        }
    });

    analyses.with_untracked(|analyses| {
        for update_analysis in update.iter() {
            let update_properties = update_analysis.properties();
            let analysis = analyses
                .iter()
                .find(|analysis| {
                    analysis.properties().with_untracked(|properties| {
                        match (properties, update_properties) {
                            (AnalysisKind::Script(properties), AnalysisKind::Script(update)) => {
                                properties.rid() == update.rid()
                            }

                            (
                                AnalysisKind::ExcelTemplate(properties),
                                AnalysisKind::ExcelTemplate(update),
                            ) => properties.rid() == update.rid(),

                            _ => false,
                        }
                    })
                })
                .unwrap();

            analysis.properties().update(|properties| {
                match (properties, update_analysis.properties()) {
                    (AnalysisKind::Script(properties), AnalysisKind::Script(update)) => {
                        *properties = update.clone();
                    }

                    (
                        AnalysisKind::ExcelTemplate(properties),
                        AnalysisKind::ExcelTemplate(update),
                    ) => {
                        *properties = update.clone();
                    }

                    _ => panic!("analysis kinds do not match"),
                }
            });

            analysis
                .fs_resource()
                .update(|present| *present = update_analysis.fs_resource().clone());
        }
    });
}

fn handle_event_graph(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    workspace_graph_state: ui_lib::state::WorkspaceGraph,
    display_state: ui_lib::state::Display,
    flags: ui_lib::state::Flags,
    messages: ui_lib::message::Messages,
) {
    let lib::EventKind::Project(update) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Project::FolderRemoved
        | db::event::Project::Moved(_)
        | db::event::Project::Properties(_)
        | db::event::Project::Settings(_)
        | db::event::Project::Analyses(_)
        | db::event::Project::AnalysisFile(_) => unreachable!("handled elsewhere"),

        db::event::Project::Graph(_) => {
            handle_event_graph_graph(event, graph, workspace_graph_state, display_state)
        }
        db::event::Project::Container { .. } => handle_event_graph_container(
            event,
            graph,
            workspace_graph_state.selection_resources(),
            flags,
            messages,
        ),
        db::event::Project::Asset { .. } => handle_event_graph_asset(event, graph),
        db::event::Project::AssetFile(_) => handle_event_graph_asset_file(event, graph),
    }
}

fn handle_event_graph_graph(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    workspace_graph_state: ui_lib::state::WorkspaceGraph,
    display_state: ui_lib::state::Display,
) {
    let lib::EventKind::Project(db::event::Project::Graph(update)) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Graph::Created(_) => todo!(),
        db::event::Graph::Inserted { .. } => {
            handle_event_graph_graph_inserted(event, graph, workspace_graph_state, display_state)
        }
        db::event::Graph::Renamed { .. } => handle_event_graph_graph_renamed(event, graph),
        db::event::Graph::Moved { from, to } => todo!(),
        db::event::Graph::Removed(_) => {
            handle_event_graph_graph_removed(event, graph, workspace_graph_state, display_state)
        }
    }
}

fn handle_event_graph_graph_inserted(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    workspace_graph_state: ui_lib::state::WorkspaceGraph,
    display_state: ui_lib::state::Display,
) {
    let lib::EventKind::Project(db::event::Project::Graph(db::event::Graph::Inserted {
        parent,
        graph: subgraph,
    })) = event.kind()
    else {
        panic!("invalid event kind");
    };

    // NB: Must create visibility and selection resource signals first before inserting nodes into graph.
    // Downstream components expect a visibility signal to be present.
    let subgraph = ui_lib::state::Graph::new(subgraph.clone());

    let selection_resources = subgraph.nodes().with_untracked(|nodes| {
        nodes
            .iter()
            .flat_map(|node| {
                let mut resources = vec![];
                node.properties().with_untracked(|properties| {
                    if let db::state::DataResource::Ok(properties) = properties {
                        resources.push(ui_lib::state::workspace_graph::ResourceSelection::new(
                            properties.rid().read_only(),
                            ui_lib::state::workspace_graph::ResourceKind::Container,
                        ))
                    }
                });

                node.assets().with_untracked(|assets| {
                    if let db::state::DataResource::Ok(assets) = assets {
                        let assets = assets.with_untracked(|assets| {
                            assets
                                .iter()
                                .map(|asset| {
                                    ui_lib::state::workspace_graph::ResourceSelection::new(
                                        asset.rid().read_only(),
                                        ui_lib::state::workspace_graph::ResourceKind::Asset,
                                    )
                                })
                                .collect::<Vec<_>>()
                        });

                        resources.extend(assets);
                    }
                });

                resources
            })
            .collect::<Vec<_>>()
    });

    workspace_graph_state
        .selection_resources()
        .extend(selection_resources);

    let visibility_inserted = subgraph.nodes().with_untracked(|nodes| {
        nodes
            .iter()
            .cloned()
            .map(|container| (container, ArcRwSignal::new(true)))
            .collect::<Vec<_>>()
    });

    workspace_graph_state
        .container_visiblity()
        .update(|visibilities| {
            visibilities.extend(visibility_inserted);
        });

    let parent = ui_lib::utils::normalize_path_sep(parent);
    let display_graph = ui_lib::state::Display::from(
        &subgraph,
        workspace_graph_state.container_visiblity().read_only(),
    );
    let parent_node = graph.find(&parent).unwrap().unwrap();
    display_state.insert(&parent_node, display_graph).unwrap();

    graph.insert(&parent, subgraph).unwrap();
}

fn handle_event_graph_graph_renamed(event: lib::Event, graph: ui_lib::state::Graph) {
    let lib::EventKind::Project(db::event::Project::Graph(db::event::Graph::Renamed { from, to })) =
        event.kind()
    else {
        panic!("invalid event kind");
    };

    graph.rename(from, to).unwrap();
}

fn handle_event_graph_graph_removed(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    workspace_graph_state: ui_lib::state::WorkspaceGraph,
    display_state: ui_lib::state::Display,
) {
    let lib::EventKind::Project(db::event::Project::Graph(db::event::Graph::Removed(path))) =
        event.kind()
    else {
        panic!("invalid event kind");
    };

    // NB: Must remove nodes first, then remove visibility signals.
    // Downstream components expect a visibility signal to be present.
    let path = ui_lib::utils::normalize_path_sep(path);
    let root = graph.find(&path).unwrap().unwrap();
    let removed = graph.remove(&path).unwrap();
    display_state.remove(&root).unwrap();

    let removed_ids = removed
        .iter()
        .flat_map(|node| {
            let mut resources = vec![];
            node.properties().with_untracked(|properties| {
                if let db::state::DataResource::Ok(properties) = properties {
                    resources.push(properties.rid().get_untracked());
                }
            });

            node.assets().with_untracked(|assets| {
                if let db::state::DataResource::Ok(assets) = assets {
                    assets.with_untracked(|assets| {
                        let assets = assets.iter().map(|asset| asset.rid().get_untracked());

                        resources.extend(assets);
                    })
                }
            });

            resources
        })
        .collect::<Vec<_>>();

    workspace_graph_state
        .selection_resources()
        .remove(&removed_ids);

    workspace_graph_state
        .container_visiblity()
        .update(|visibilities| {
            visibilities.retain(|(container, _)| {
                graph
                    .nodes()
                    .with_untracked(|nodes| nodes.iter().any(|node| Arc::ptr_eq(node, container)))
            });
        });
}

fn handle_event_graph_container(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
    flags: ui_lib::state::Flags,
    messages: ui_lib::message::Messages,
) {
    let lib::EventKind::Project(db::event::Project::Container { update, .. }) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Container::Properties(_) => {
            handle_event_graph_container_properties(event, graph, selection_resources)
        }
        db::event::Container::Settings(_) => handle_event_graph_container_settings(event, graph),
        db::event::Container::Assets(_) => {
            handle_event_graph_container_assets(event, graph, selection_resources)
        }
        db::event::Container::Flags(_) => {
            handle_event_graph_container_flags(event, graph, flags, messages)
        }
    }
}

fn handle_event_graph_container_properties(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        update: db::event::Container::Properties(update),
        ..
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => {
            handle_event_graph_container_properties_created(event, graph, selection_resources)
        }
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => {
            handle_event_graph_container_properties_corrupted(event, graph, selection_resources)
        }
        db::event::DataResource::Repaired(_) => {
            handle_event_graph_container_properties_repaired(event, graph, selection_resources)
        }
        db::event::DataResource::Modified(_) => {
            handle_event_graph_container_properties_modified(event, graph)
        }
    }
}

fn handle_event_graph_container_properties_created(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Created(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };
    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    match update {
        Ok(update) => {
            if container
                .properties()
                .with_untracked(|properties| properties.is_err())
            {
                let properties = ui_lib::state::container::Properties::new(
                    update.rid.clone(),
                    update.properties.clone(),
                );

                selection_resources.push(ui_lib::state::workspace_graph::ResourceSelection::new(
                    properties.rid().read_only(),
                    ui_lib::state::workspace_graph::ResourceKind::Container,
                ));

                container.properties().update(|container_properties| {
                    *container_properties = db::state::DataResource::Ok(properties);
                });
            } else {
                update_container_properties(container, update);
            }
        }

        Err(err) => {
            if !container.properties().with_untracked(|properties| {
                if let Err(properties_err) = properties {
                    properties_err == err
                } else {
                    false
                }
            }) {
                container
                    .properties()
                    .update(|properties| *properties = Err(err.clone()));
            }

            if !container.analyses().with_untracked(|analyses| {
                if let Err(analyses_err) = analyses {
                    analyses_err == err
                } else {
                    false
                }
            }) {
                container
                    .analyses()
                    .update(|analyses| *analyses = Err(err.clone()));
            }
        }
    }
}

fn handle_event_graph_container_properties_repaired(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Repaired(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };
    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    assert!(
        container
            .properties()
            .with_untracked(|properties| properties.is_err())
    );

    let properties =
        ui_lib::state::container::Properties::new(update.rid.clone(), update.properties.clone());

    selection_resources.push(ui_lib::state::workspace_graph::ResourceSelection::new(
        properties.rid().read_only(),
        ui_lib::state::workspace_graph::ResourceKind::Container,
    ));

    container.properties().update(|container_properties| {
        *container_properties = db::state::DataResource::Ok(properties);
    });
}

fn handle_event_graph_container_properties_corrupted(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Corrupted(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };
    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    assert!(
        container
            .properties()
            .with_untracked(|properties| properties.is_ok())
    );

    let rid = container
        .properties()
        .with_untracked(|properties| properties.as_ref().unwrap().rid().get_untracked());
    selection_resources.remove(&vec![rid]);

    container.properties().update(|properties| {
        *properties = db::state::DataResource::Err(update.clone());
    });
}

fn handle_event_graph_container_properties_modified(
    event: lib::Event,
    graph: ui_lib::state::Graph,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    update_container_properties(container, update);
}

fn update_container_properties(
    container: ui_lib::state::graph::Node,
    update: &local::project::container::StoredProperties,
) {
    container.properties().with_untracked(|properties| {
        let db::state::DataResource::Ok(properties) = properties else {
            panic!("invalid state");
        };

        if properties.rid().with_untracked(|rid| update.rid != *rid) {
            properties.rid().set(update.rid.clone());
        }

        if properties
            .name()
            .with_untracked(|name| update.properties.name != *name)
        {
            properties.name().set(update.properties.name.clone());
        }

        if properties
            .kind()
            .with_untracked(|kind| update.properties.kind != *kind)
        {
            properties.kind().set(update.properties.kind.clone());
        }

        if properties
            .description()
            .with_untracked(|description| update.properties.description != *description)
        {
            properties
                .description()
                .set(update.properties.description.clone());
        }

        if properties
            .tags()
            .with_untracked(|tags| update.properties.tags != *tags)
        {
            properties.tags().set(update.properties.tags.clone());
        }

        update_metadata(properties.metadata(), &update.properties.metadata);
    });

    // NB: Can not nest signal updates or borrow error will occur.
    container.analyses().with_untracked(|analyses| {
        let db::state::DataResource::Ok(analyses) = analyses else {
            panic!("invalid state");
        };

        analyses.update(|analyses| {
            analyses.retain(|association| {
                update
                    .analyses
                    .iter()
                    .any(|assoc| assoc.analysis() == association.analysis())
            });

            let new = update
                .analyses
                .iter()
                .filter_map(|association_update| {
                    if !analyses
                        .iter()
                        .any(|association| association.analysis() == association_update.analysis())
                    {
                        Some(ui_lib::state::AnalysisAssociation::new(
                            association_update.clone(),
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            analyses.extend(new);
        });

        analyses.with_untracked(|analyses| {
            for association in analyses.iter() {
                let Some(association_update) = update
                    .analyses
                    .iter()
                    .find(|update| update.analysis() == association.analysis())
                else {
                    continue;
                };

                if association
                    .autorun()
                    .with_untracked(|autorun| association_update.autorun != *autorun)
                {
                    association.autorun().set(association_update.autorun);
                }

                if association
                    .priority()
                    .with_untracked(|priority| association_update.priority != *priority)
                {
                    association.priority().set(association_update.priority);
                }
            }
        });
    });
}

fn handle_event_graph_container_settings(event: lib::Event, graph: ui_lib::state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Settings(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    match update {
        db::event::DataResource::Created(update) => match update {
            db::state::DataResource::Err(err) => {
                container
                    .settings()
                    .set(db::state::DataResource::Err(err.clone()));
            }

            db::state::DataResource::Ok(update) => {
                if container
                    .settings()
                    .with_untracked(|settings| settings.is_err())
                {
                    container.settings().set(db::state::DataResource::Ok(
                        ui_lib::state::container::Settings::new(update.clone()),
                    ));
                } else {
                    container.settings().with_untracked(|settings| {
                        let db::state::DataResource::Ok(settings) = settings else {
                            unreachable!("invalid state");
                        };

                        settings.creator().set(update.creator.clone());
                        settings.created().set(update.created.clone());
                        settings.permissions().set(update.permissions.clone());
                    });
                }
            }
        },
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => todo!(),
        db::event::DataResource::Repaired(_) => todo!(),
        db::event::DataResource::Modified(update) => {
            container.settings().with_untracked(|settings| {
                let db::state::DataResource::Ok(settings) = settings else {
                    panic!("invalid state");
                };

                settings.creator().set(update.creator.clone());
                settings.created().set(update.created.clone());
                settings.permissions().set(update.permissions.clone());
            });
        }
    }
}

fn handle_event_graph_container_assets(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path: _path,
        update: db::event::Container::Assets(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => {
            handle_event_graph_container_assets_created(event, graph, selection_resources)
        }
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => {
            handle_event_graph_container_assets_corrupted(event, graph, selection_resources)
        }
        db::event::DataResource::Repaired(_) => {
            handle_event_graph_container_assets_repaired(event, graph, selection_resources)
        }
        db::event::DataResource::Modified(_) => {
            handle_event_graph_container_assets_modified(event, graph, selection_resources)
        }
    }
}

fn handle_event_graph_container_assets_created(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Created(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    match update {
        db::state::DataResource::Err(err) => {
            container
                .assets()
                .set(db::state::DataResource::Err(err.clone()));
        }

        db::state::DataResource::Ok(update) => {
            if container.assets().with_untracked(|assets| assets.is_err()) {
                let assets = update
                    .iter()
                    .map(|asset| ui_lib::state::Asset::new(asset.clone()))
                    .collect::<Vec<_>>();

                let resources = assets
                    .iter()
                    .map(|asset| {
                        ui_lib::state::workspace_graph::ResourceSelection::new(
                            asset.rid().read_only(),
                            ui_lib::state::workspace_graph::ResourceKind::Asset,
                        )
                    })
                    .collect::<Vec<ui_lib::state::workspace_graph::ResourceSelection>>();
                selection_resources.extend(resources);

                container
                    .assets()
                    .set(db::state::DataResource::Ok(RwSignal::new(assets)));
            } else {
                let removed = container.assets().with_untracked(|assets| {
                    assets.as_ref().unwrap().with_untracked(|assets| {
                        assets
                            .iter()
                            .filter_map(|asset| {
                                (!update.iter().any(|update| {
                                    asset.rid().with_untracked(|rid| update.rid() == rid)
                                }))
                                .then_some(asset.rid().read_only())
                            })
                            .collect::<Vec<_>>()
                    })
                });

                let (modified, added): (Vec<_>, Vec<_>) = update.iter().partition(|update| {
                    container.assets().with_untracked(|assets| {
                        assets.as_ref().unwrap().with_untracked(|assets| {
                            assets
                                .iter()
                                .any(|asset| asset.rid().with_untracked(|rid| update.rid() == rid))
                        })
                    })
                });

                let added = added
                    .into_iter()
                    .map(|update| ui_lib::state::Asset::new(update.clone()))
                    .collect::<Vec<_>>();

                let removed_ids = removed.iter().map(|asset| asset.get_untracked()).collect();
                selection_resources.remove(&removed_ids);

                let added_selection_resources = added
                    .iter()
                    .map(|asset| {
                        ui_lib::state::workspace_graph::ResourceSelection::new(
                            asset.rid().read_only(),
                            ui_lib::state::workspace_graph::ResourceKind::Asset,
                        )
                    })
                    .collect();
                selection_resources.extend(added_selection_resources);

                container.assets().update(|assets| {
                    let db::state::DataResource::Ok(assets) = assets else {
                        panic!("invalid state");
                    };

                    assets.update(|assets| {
                        assets.retain(|asset| {
                            !removed.iter().any(|removed| {
                                removed.with_untracked(|removed| {
                                    asset.rid().with_untracked(|asset| removed == asset)
                                })
                            })
                        });

                        modified.into_iter().for_each(|update| {
                            let Some(asset) = assets.iter().find(|asset| {
                                asset.rid().with_untracked(|rid| rid == update.rid())
                            }) else {
                                panic!("invalid state");
                            };

                            update_asset(asset, update);
                        });

                        assets.extend(added);
                    });
                });
            }
        }
    }
}

fn handle_event_graph_container_assets_modified(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    // NB: Can not nest signal updates or borrow error will occur.
    let (assets_update, assets_new): (Vec<_>, Vec<_>) =
        container.assets().with_untracked(|assets| {
            let db::state::DataResource::Ok(assets) = assets else {
                panic!("invalid state");
            };

            assets.with_untracked(|assets| {
                update.iter().partition(|update_asset| {
                    assets
                        .iter()
                        .any(|asset| asset.rid().with_untracked(|rid| rid == update_asset.rid()))
                })
            })
        });

    let assets_new = assets_new
        .into_iter()
        .map(|asset| ui_lib::state::Asset::new(asset.clone()))
        .collect::<Vec<_>>();

    let removed = container.assets().with_untracked(|assets| {
        let db::state::DataResource::Ok(assets) = assets else {
            panic!("invalid state");
        };

        assets.with_untracked(|assets| {
            assets
                .iter()
                .filter_map(|asset| {
                    (!update
                        .iter()
                        .any(|update| asset.rid().with_untracked(|rid| update.rid() == rid)))
                    .then_some(asset.rid().read_only())
                })
                .collect::<Vec<_>>()
        })
    });

    let removed_ids = removed
        .iter()
        .map(|removed| removed.get_untracked())
        .collect();
    selection_resources.remove(&removed_ids);

    let selection_resources_new = assets_new
        .iter()
        .map(|asset| {
            ui_lib::state::workspace_graph::ResourceSelection::new(
                asset.rid().read_only(),
                ui_lib::state::workspace_graph::ResourceKind::Asset,
            )
        })
        .collect();
    selection_resources.extend(selection_resources_new);

    container.assets().with_untracked(|assets| {
        let db::state::DataResource::Ok(assets) = assets else {
            panic!("invalid state");
        };

        assets.update(|assets| {
            assets.retain(|asset| {
                !removed.iter().any(|removed| {
                    asset
                        .rid()
                        .with_untracked(|rid| removed.with_untracked(|removed| removed == rid))
                })
            });

            assets.extend(assets_new);
        });
    });

    for asset_update in assets_update {
        let asset = container.assets().with_untracked(|assets| {
            let db::state::DataResource::Ok(assets) = assets else {
                panic!("invalid state");
            };

            assets
                .with_untracked(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.rid().with_untracked(|rid| rid == asset_update.rid()))
                        .cloned()
                })
                .unwrap()
        });

        update_asset(&asset, asset_update);
    }
}

fn update_asset(asset: &ui_lib::state::Asset, update: &db::state::Asset) {
    assert!(asset.rid().with_untracked(|rid| rid == update.rid()));

    if asset
        .name()
        .with_untracked(|name| name != &update.properties.name)
    {
        asset
            .name()
            .update(|name| *name = update.properties.name.clone());
    }

    if asset
        .kind()
        .with_untracked(|kind| kind != &update.properties.kind)
    {
        asset
            .kind()
            .update(|kind| *kind = update.properties.kind.clone());
    }

    if asset
        .description()
        .with_untracked(|description| description != &update.properties.description)
    {
        asset
            .description()
            .update(|description| *description = update.properties.description.clone());
    }

    if asset
        .tags()
        .with_untracked(|tags| tags != &update.properties.tags)
    {
        asset
            .tags()
            .update(|tags| *tags = update.properties.tags.clone());
    }

    if asset.path().with_untracked(|path| path != &update.path) {
        asset.path().update(|path| *path = update.path.clone());
    }

    if asset
        .fs_resource()
        .with_untracked(|fs_resource| fs_resource.is_present() != update.is_present())
    {
        asset.fs_resource().update(|fs_resource| {
            *fs_resource = if update.is_present() {
                db::state::FileResource::Present
            } else {
                db::state::FileResource::Absent
            }
        });
    }

    if asset
        .created()
        .with_untracked(|created| created != update.properties.created())
    {
        asset
            .created()
            .update(|created| *created = (*update).properties.created().clone());
    }

    if asset
        .creator()
        .with_untracked(|creator| creator != &update.properties.creator)
    {
        asset
            .creator()
            .update(|creator| *creator = (*update).properties.creator.clone());
    }

    update_metadata(asset.metadata(), &update.properties.metadata);
}

fn handle_event_graph_container_assets_corrupted(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Corrupted(err)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    let rids = container.assets().with_untracked(|assets| {
        assets.as_ref().unwrap().with_untracked(|assets| {
            assets
                .iter()
                .map(|asset| asset.rid().get_untracked())
                .collect::<Vec<_>>()
        })
    });
    selection_resources.remove(&rids);

    container.assets().update(|container_assets| {
        *container_assets = db::state::DataResource::Err(err.clone());
    });
}

fn handle_event_graph_container_assets_repaired(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    selection_resources: &ui_lib::state::workspace_graph::SelectionResources,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Repaired(assets)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    let assets = assets
        .into_iter()
        .map(|asset| ui_lib::state::Asset::new(asset.clone()))
        .collect::<Vec<_>>();

    let selections = assets
        .iter()
        .map(|asset| {
            ui_lib::state::workspace_graph::ResourceSelection::new(
                asset.rid().read_only(),
                ui_lib::state::workspace_graph::ResourceKind::Asset,
            )
        })
        .collect::<Vec<_>>();
    selection_resources.extend(selections);

    container.assets().update(|container_assets| {
        *container_assets = db::state::DataResource::Ok(RwSignal::new(assets));
    });
}

fn handle_event_graph_asset(event: lib::Event, graph: ui_lib::state::Graph) {
    let lib::EventKind::Project(db::event::Project::Asset {
        container,
        asset,
        update,
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(ui_lib::utils::normalize_path_sep(container))
        .unwrap()
        .unwrap();

    match update {
        db::event::Asset::FileCreated | db::event::Asset::FileRemoved => {
            let fs_resource = container.assets().with_untracked(|assets| {
                let db::state::DataResource::Ok(assets) = assets else {
                    todo!();
                };
                assets.with_untracked(|assets| {
                    assets
                        .iter()
                        .find(|asset_state| asset_state.rid().with_untracked(|rid| rid == asset))
                        .map(|asset| asset.fs_resource())
                })
            });

            debug_assert!(fs_resource.is_some());
            if let Some(fs_resource) = fs_resource {
                match update {
                    db::event::Asset::FileCreated => {
                        fs_resource.set(db::state::FileResource::Present)
                    }
                    db::event::Asset::FileRemoved => {
                        fs_resource.set(db::state::FileResource::Absent)
                    }
                    _ => unreachable!(),
                };
            }
        }
        db::event::Asset::Properties(update) => {
            container.assets().with_untracked(|assets| {
                let db::state::DataResource::Ok(assets) = assets else {
                    todo!();
                };

                let asset = assets.with_untracked(|assets| {
                    assets
                        .iter()
                        .find(|asset_state| asset_state.rid().with_untracked(|rid| rid == asset))
                        .unwrap()
                        .clone()
                });

                if asset
                    .fs_resource()
                    .with_untracked(|fs_resource| fs_resource.is_present() != update.is_present())
                {
                    let fs_resource = if update.is_present() {
                        db::state::FileResource::Present
                    } else {
                        db::state::FileResource::Absent
                    };
                    asset.fs_resource().set(fs_resource);
                }

                if asset
                    .name()
                    .with_untracked(|name| *name != update.properties.name)
                {
                    asset.name().set(update.properties.name.clone());
                }

                if asset
                    .kind()
                    .with_untracked(|kind| *kind != update.properties.kind)
                {
                    asset.kind().set(update.properties.kind.clone());
                }

                if asset
                    .description()
                    .with_untracked(|description| *description != update.properties.description)
                {
                    asset
                        .description()
                        .set(update.properties.description.clone());
                }

                if asset
                    .tags()
                    .with_untracked(|tags| *tags != update.properties.tags)
                {
                    asset.tags().set(update.properties.tags.clone());
                }

                asset.metadata().update(|metadata| {
                    metadata.retain(|(key, _)| {
                        update
                            .properties
                            .metadata
                            .iter()
                            .any(|(update_key, _)| key == update_key)
                    });

                    update
                        .properties
                        .metadata
                        .iter()
                        .for_each(|(update_key, update_value)| {
                            if let Some(value) = metadata.iter().find_map(|(key, value)| {
                                if update_key == key { Some(value) } else { None }
                            }) {
                                if value.with_untracked(|value| value != update_value) {
                                    value.set(update_value.clone())
                                }
                            } else {
                                metadata.push((
                                    update_key.clone(),
                                    RwSignal::new(update_value.clone()),
                                ));
                            }
                        });
                });
            });
        }
    }
}

fn handle_event_graph_asset_file(event: lib::Event, graph: ui_lib::state::Graph) {
    let lib::EventKind::Project(db::event::Project::AssetFile(kind)) = event.kind() else {
        panic!("invalid event kind");
    };

    match kind {
        db::event::AssetFile::Created(path) => todo!(),
        db::event::AssetFile::Removed(path) => todo!(),
        db::event::AssetFile::Renamed { from, to } => todo!(),
        db::event::AssetFile::Moved { from, to } => todo!(),
    }
}

fn update_metadata(
    metadata: RwSignal<ui_lib::state::Metadata>,
    update: &syre_core::project::Metadata,
) {
    // NB: Can not nest signal updates or borrow error will occur.
    let (keys_update, keys_new): (Vec<_>, Vec<_>) = metadata.with_untracked(|metadata| {
        update
            .keys()
            .partition(|key| metadata.iter().any(|(k, _)| k == *key))
    });

    metadata.update(|metadata| {
        metadata.retain(|(key, _)| keys_update.contains(&key));

        let new = update
            .iter()
            .filter_map(|(key, value)| {
                if keys_new.contains(&key) {
                    Some((key.clone(), RwSignal::new(value.clone())))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        metadata.extend(new);
    });

    metadata.with_untracked(|metadata| {
        for (update_key, update_value) in update.iter().filter(|(key, _)| keys_update.contains(key))
        {
            let value = metadata
                .iter()
                .find_map(
                    |(key, value)| {
                        if key == update_key { Some(value) } else { None }
                    },
                )
                .unwrap();

            if value.with_untracked(|value| update_value != value) {
                value.set(update_value.clone());
            }
        }
    })
}

fn handle_event_graph_container_flags(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    flags: ui_lib::state::Flags,
    messages: ui_lib::message::Messages,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path: _path,
        update: db::event::Container::Flags(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => {
            handle_event_graph_container_flags_created(event, flags)
        }
        db::event::DataResource::Removed => {
            handle_event_graph_container_flags_removed(event, graph, flags)
        }
        db::event::DataResource::Corrupted(_) => {
            handle_event_graph_container_flags_corrupted(event, graph, flags, messages)
        }
        db::event::DataResource::Repaired(_) => {
            handle_event_graph_container_flags_repaired(event, flags)
        }
        db::event::DataResource::Modified(_) => {
            handle_event_graph_container_flags_modified(event, graph, flags)
        }
    }
}

fn handle_event_graph_container_flags_created(event: lib::Event, flags: ui_lib::state::Flags) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Flags(db::event::DataResource::Created(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::state::DataResource::Ok(update) => {
            insert_graph_container_flags(path, update, flags);
        }
        Err(err) => {
            tracing::debug!("corrupt flags file created: {err:?}");
        }
    }
}

fn handle_event_graph_container_flags_removed(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    flags: ui_lib::state::Flags,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Flags(db::event::DataResource::Removed),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    remove_graph_container_flags(path, graph, flags);
}

fn handle_event_graph_container_flags_corrupted(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    flags: ui_lib::state::Flags,
    messages: ui_lib::message::Messages,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Flags(db::event::DataResource::Corrupted(error)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    remove_graph_container_flags(path, graph, flags);
    let msg = ui_lib::message::Builder::error("Flags file corrupted.");
    let msg = msg.body(format!("{error:?}"));
    messages.push_message(msg.build_str());
}

fn handle_event_graph_container_flags_repaired(event: lib::Event, flags: ui_lib::state::Flags) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Flags(db::event::DataResource::Repaired(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    insert_graph_container_flags(path, update, flags);
}

fn handle_event_graph_container_flags_modified(
    event: lib::Event,
    graph: ui_lib::state::Graph,
    flags: ui_lib::state::Flags,
) {
    let lib::EventKind::Project(db::event::Project::Container {
        path: container_path,
        update: db::event::Container::Flags(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };
    let root_dir = PathBuf::from("/");

    let update = update
        .into_iter()
        .map(|(path, flags)| {
            let path = if *path == root_dir {
                container_path.clone()
            } else {
                container_path.join(path)
            };

            (ui_lib::utils::normalize_path_sep(path), flags)
        })
        .collect::<Vec<_>>();

    let container_path = ui_lib::utils::normalize_path_sep(container_path);
    let container = graph.find(&container_path).unwrap().unwrap();
    let asset_paths = container
        .assets()
        .read_untracked()
        .as_ref()
        .map(|assets| {
            assets
                .read_untracked()
                .iter()
                .map(|asset| {
                    ui_lib::utils::normalize_path_sep(
                        container_path.join(asset.path().get_untracked()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or(vec![]);
    let all_paths = asset_paths
        .into_iter()
        .chain(std::iter::once(ui_lib::utils::normalize_path_sep(
            container_path,
        )))
        .collect::<Vec<_>>();

    let update_paths = update.iter().map(|(path, _)| path).collect::<Vec<_>>();
    flags
        .write()
        .retain(|(path, _)| !all_paths.contains(&path) || update_paths.contains(&path));

    update.into_iter().for_each(|(path, flags_update)| {
        let _flags = flags.read_untracked();
        if let Some((_, state_flags)) = _flags.iter().find(|(state_path, _)| *state_path == path) {
            state_flags.set(flags_update.clone());
        } else {
            drop(_flags);
            flags
                .write()
                .push((path, ArcRwSignal::new(flags_update.clone())));
        }
    });
}

fn insert_graph_container_flags(
    container: &PathBuf,
    update: &Vec<(PathBuf, Vec<local::project::Flag>)>,
    flags: ui_lib::state::Flags,
) {
    let root_dir = PathBuf::from("/");

    let update = update
        .into_iter()
        .map(|(path, flags)| {
            let path = if *path == root_dir {
                container.clone()
            } else {
                container.join(path)
            };

            (
                ui_lib::utils::normalize_path_sep(path),
                ArcRwSignal::new(flags.clone()),
            )
        })
        .collect::<Vec<_>>();

    assert!(!update.iter().any(|(update_path, _)| {
        flags
            .read_untracked()
            .iter()
            .find(|(flag_path, _)| update_path == flag_path)
            .is_some()
    }));

    flags.write().extend(update)
}

fn remove_graph_container_flags(
    container: impl AsRef<Path>,
    graph: ui_lib::state::Graph,
    flags: ui_lib::state::Flags,
) {
    let container_path = container.as_ref();
    let container = graph
        .find(ui_lib::utils::normalize_path_sep(container_path))
        .unwrap()
        .unwrap();
    let assets = container.assets();
    let mut paths = assets
        .read_untracked()
        .as_ref()
        .map(|assets| {
            assets
                .read_untracked()
                .iter()
                .map(|asset| {
                    ui_lib::utils::normalize_path_sep(
                        container_path.join(&*asset.path().read_untracked()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or(vec![]);
    paths.push(ui_lib::utils::normalize_path_sep(container_path));

    flags
        .write()
        .retain(|(path, _)| !paths.contains(&ui_lib::utils::normalize_path_sep(path)));
}
