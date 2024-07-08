use super::{state, Canvas, ProjectBar, PropertiesBar};
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
    provide_context(state::WorkspaceGraph::new());
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
            <div class="w-1/6 border-l-1">
                <PropertiesBar/>
            </div>
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
        db::event::Project::Container { path, update } => {
            handle_event_graph_container(event, graph)
        }
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
        db::event::Graph::Renamed { from, to } => graph.rename(from, to).unwrap(),
        db::event::Graph::Moved { from, to } => todo!(),
        db::event::Graph::Removed(path) => graph.remove(path).unwrap(),
    }
}

fn handle_event_graph_container(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container { path, update }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Container::Properties(_) => {
            handle_event_graph_container_properties(event, graph)
        }
        db::event::Container::Settings(_) => todo!(),
        db::event::Container::Assets(_) => todo!(),
    }
}

fn handle_event_graph_container_properties(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph.find(path).unwrap().unwrap();
    match update {
        db::event::DataResource::Created(_) => todo!(),
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => todo!(),
        db::event::DataResource::Repaired(_) => todo!(),
        db::event::DataResource::Modified(update) => {
            container.properties().update(|properties| {
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

                properties.metadata().update(|metadata| {
                    metadata.retain(|(key, value)| {
                        if let Some(value_new) = update.properties.metadata.get(key) {
                            if value.with_untracked(|value| value_new != value) {
                                value.set(value_new.clone());
                            }

                            true
                        } else {
                            false
                        }
                    });

                    for (key, value_new) in update.properties.metadata.iter() {
                        if !metadata.iter().any(|(k, _)| key == key) {
                            metadata.push((key.clone(), create_rw_signal(value_new.clone())));
                        }
                    }
                });
            });

            container.analyses().update(|analyses| {
                let db::state::DataResource::Ok(analyses) = analyses else {
                    panic!("invalid state");
                };

                analyses.update(|analyses| {
                    analyses.retain(|association| {
                        if let Some(association_update) = update
                            .analyses
                            .iter()
                            .find(|assoc| assoc.analysis() == association.analysis())
                        {
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

                            true
                        } else {
                            false
                        }
                    });

                    for association_update in update.analyses.iter() {
                        if !analyses.iter().any(|association| {
                            association.analysis() == association_update.analysis()
                        }) {
                            analyses
                                .push(state::AnalysisAssociation::new(association_update.clone()));
                        }
                    }
                });
            });
        }
    }
}
