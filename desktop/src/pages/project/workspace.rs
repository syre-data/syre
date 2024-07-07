use super::{state, Canvas, ProjectBar};
use crate::invoke::invoke;
use futures::stream::StreamExt;
use leptos::*;
use leptos_router::use_params_map;
use serde::Serialize;
use std::{assert_matches::assert_matches, ops::Deref, str::FromStr};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local_database as db;

#[component]
pub fn Workspace() -> impl IntoView {
    let params = use_params_map();
    let id =
        move || params.with(|params| ResourceId::from_str(&params.get("id").unwrap()).unwrap());
    let resources = create_resource(id, |id| async move { fetch_project_resources(id).await });

    view! {
        <Suspense fallback=Loading>
            {move || {
                resources()
                    .map(|resources| {
                        resources
                            .map(|(project_data, graph)| {
                                view! { <WorkspaceView project_data graph/> }
                            })
                            .or_else(|| Some(view! { <NoProject/> }))
                    })
            }}

        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div>"Loading..."</div> }
}

#[component]
fn NoProject() -> impl IntoView {
    view! { <div>"Project state was not found."</div> }
}

#[component]
fn WorkspaceView(
    project_data: db::state::ProjectData,
    graph: db::state::FolderResource<db::state::Graph>,
) -> impl IntoView {
    assert!(project_data.properties().is_ok());

    provide_context(state::Workspace::new());
    provide_context(state::Project::new(project_data));

    view! {
        <div class="select-none">
            <div class="h-1/8 border-b-1">"tab bar"</div>
            <div class="h-1/6 border-b-1">
                <ProjectBar/>
            </div>
            {move || {
                match graph.as_ref() {
                    db::state::FolderResource::Present(graph) => {
                        view! { <WorkspaceGraph graph=graph.clone()/> }
                    }
                    db::state::FolderResource::Absent => view! { <NoGraph/> },
                }
            }}

        </div>
    }
}

#[component]
fn NoGraph() -> impl IntoView {
    view! { <div>"Data graph does not exist."</div> }
}

#[component]
fn WorkspaceGraph(graph: db::state::Graph) -> impl IntoView {
    let graph = state::Graph::new(graph);
    provide_context(graph.clone());
    let project = expect_context::<state::Project>();

    spawn_local(async move {
        let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(
            &project
                .rid()
                .with_untracked(|rid| lib::event::topic::graph(rid)),
        )
        .await
        .unwrap();

        while let Some(events) = listener.next().await {
            tracing::debug!(?events);
            for event in events.payload {
                tracing::trace!(?event);
                assert_matches!(event.kind(), lib::EventKind::Project(_));
                handle_event_graph(event, graph.clone());
            }
        }
    });

    view! {
        <div class="flex">
            <div class="w-1/6 border-r-1">"left"</div>
            <div class="flex-grow">
                <Canvas/>

            </div>
            <div class="w-1/6 border-l-1">"right"</div>
        </div>
    }
}

async fn fetch_project_resources(
    project: ResourceId,
) -> Option<(
    db::state::ProjectData,
    db::state::FolderResource<db::state::Graph>,
)> {
    let resources = invoke::<
        Option<(
            db::state::ProjectData,
            db::state::FolderResource<db::state::Graph>,
        )>,
    >("project_resources", ProjectArgs { project })
    .await;

    assert!(if let Some((data, _)) = resources.as_ref() {
        data.properties().is_ok()
    } else {
        true
    });

    resources
}

#[derive(Serialize)]
struct ProjectArgs {
    project: ResourceId,
}

fn handle_event_graph(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(update) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Project::Removed
        | db::event::Project::Moved(_)
        | db::event::Project::Properties(_)
        | db::event::Project::Settings(_)
        | db::event::Project::Analyses(_) => unreachable!("handled elsewhere"),
        db::event::Project::Graph(_) => handle_event_graph_graph(event, graph),
        db::event::Project::Container { path, update } => todo!(),
        db::event::Project::Asset {
            container,
            asset,
            update,
        } => todo!(),
        db::event::Project::AssetFile(_) => todo!(),
        db::event::Project::AnalysisFile(_) => todo!(),
        db::event::Project::Flag { resource, message } => todo!(),
    }
}

fn handle_event_graph_graph(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Graph(update)) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Graph::Created(_) => todo!(),
        db::event::Graph::Inserted {
            parent,
            graph: subgraph,
        } => graph
            .insert(parent, state::Graph::new(subgraph.clone()))
            .unwrap(),
        db::event::Graph::Renamed { from, to } => todo!(),
        db::event::Graph::Moved { from, to } => todo!(),
        db::event::Graph::Removed(_) => todo!(),
    }
}
