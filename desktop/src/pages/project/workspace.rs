use super::{canvas, properties, state, Canvas, LayersNav, ProjectBar, PropertiesBar};
use crate::{common, components::Logo};
use futures::stream::StreamExt;
use leptos::*;
use leptos_router::*;
use serde::Serialize;
use std::{io, path::PathBuf, str::FromStr};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_local_database as db;
use tauri_sys::window::DragDropPayload;
use tracing::instrument::WithSubscriber;
use wasm_bindgen::JsCast;

const THROTTLE_DRAG_EVENT: f64 = 50.0; // drag drop event debounce in ms.

#[derive(derive_more::Deref, derive_more::From, Clone)]
pub struct DragOverWorkspaceResource(Option<WorkspaceResource>);
impl DragOverWorkspaceResource {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn into_inner(self) -> Option<WorkspaceResource> {
        self.0
    }
}

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
                            .map(|(project_path, project_data, graph)| {
                                view! { <WorkspaceView project_path project_data graph /> }
                            })
                            .or_else(|| Some(view! { <NoProject /> }))
                    })
            }}

        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div class="pt-4 text-center">"Loading..."</div> }
}

#[component]
fn NoProject() -> impl IntoView {
    view! { <div class="pt-4 text-center">"Project state was not found."</div> }
}

#[component]
fn WorkspaceView(
    project_path: PathBuf,
    project_data: db::state::ProjectData,
    graph: db::state::FolderResource<db::state::Graph>,
) -> impl IntoView {
    assert!(project_data.properties().is_ok());

    let project = state::Project::new(project_path, project_data);
    provide_context(state::Workspace::new());
    provide_context(project.clone());
    provide_context(DragOverWorkspaceResource::new());

    spawn_local({
        let project = project.clone();
        async move {
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
                    let lib::EventKind::Project(update) = event.kind() else {
                        panic!("invalid event kind");
                    };

                    match update {
                        db::event::Project::Removed
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
                        | db::event::Project::AssetFile(_)
                        | db::event::Project::Flag { .. } => continue, // handled elsewhere
                    }
                }
            }
        }
    });

    view! {
        <div class="select-none h-full">
            <ProjectNav />
            <div class="border-b">
                <ProjectBar />
            </div>
            {move || {
                match graph.as_ref() {
                    db::state::FolderResource::Present(graph) => {
                        view! { <WorkspaceGraph graph=graph.clone() /> }
                    }
                    db::state::FolderResource::Absent => view! { <NoGraph /> },
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
    let project = expect_context::<state::Project>();
    let graph = state::Graph::new(graph);
    provide_context(graph.clone());
    provide_context(state::WorkspaceGraph::new());

    let (drag_over_workspace_resource, set_drag_over_workspace_resource) =
        create_signal(DragOverWorkspaceResource::new());
    let drag_over_workspace_resource =
        leptos_use::signal_throttled(drag_over_workspace_resource, THROTTLE_DRAG_EVENT);
    provide_context(drag_over_workspace_resource);

    spawn_local({
        let project = project.clone();
        let graph = graph.clone();
        async move {
            let mut listener = tauri_sys::event::listen::<Vec<lib::Event>>(
                &project
                    .rid()
                    .with_untracked(|rid| lib::event::topic::graph(rid)),
            )
            .await
            .unwrap();

            while let Some(events) = listener.next().await {
                for event in events.payload {
                    let lib::EventKind::Project(update) = event.kind() else {
                        panic!("invalid event kind");
                    };

                    match update {
                        db::event::Project::Removed
                        | db::event::Project::Moved(_)
                        | db::event::Project::Properties(_)
                        | db::event::Project::Settings(_)
                        | db::event::Project::Analyses(_)
                        | db::event::Project::AnalysisFile(_) => continue, // handled elsewhere

                        db::event::Project::Graph(_)
                        | db::event::Project::Container { .. }
                        | db::event::Project::Asset { .. }
                        | db::event::Project::AssetFile(_)
                        | db::event::Project::Flag { .. } => {
                            handle_event_graph(event, graph.clone())
                        }
                    }
                }
            }
        }
    });

    {
        // TODO: Only tested on Linux.
        // Need to test on Windows and Mac.
        let project = project.clone();
        let graph = graph.clone();
        spawn_local(async move {
            use tauri_sys::window::DragDropEvent;

            let window = tauri_sys::window::get_current();
            let mut listener = window.on_drag_drop_event().await.unwrap();
            while let Some(event) = listener.next().await {
                match event.payload {
                    DragDropEvent::Enter(payload) => {
                        if payload.paths().is_empty() {
                            return;
                        }

                        let resource = resource_from_position(payload.position()).await;
                        set_drag_over_workspace_resource(resource.into());
                    }
                    DragDropEvent::Over(payload) => {
                        let resource = resource_from_position(payload.position()).await;
                        set_drag_over_workspace_resource(resource.into());
                    }
                    DragDropEvent::Drop(payload) => {
                        if let Some(resource) =
                            drag_over_workspace_resource.get_untracked().into_inner()
                        {
                            set_drag_over_workspace_resource(None.into());
                            handle_drop_event(resource, payload, &project, &graph).await;
                        }
                    }
                    DragDropEvent::Leave => {
                        set_drag_over_workspace_resource(None.into());
                    }
                }
            }
        });
    }

    view! {
        <div class="h-full flex">
            <div class="w-1/6 border-r">
                <LayersNav />
            </div>
            <div class="flex-grow">
                <Canvas />
            </div>
            <div class="w-1/6 border-l">
                <PropertiesBar />
            </div>
        </div>
    }
}

#[component]
fn ProjectNav() -> impl IntoView {
    view! {
        <nav class="h-1/8 p-2 border-b dark:bg-secondary-900">
            <ol class="flex">
                <li>
                    <A href="/">
                        <Logo class="h-4" />
                    </A>
                </li>
            </ol>
        </nav>
    }
}

#[derive(Clone)]
pub enum WorkspaceResource {
    /// Analyses properties bar.
    Analyses,

    /// Container canvas ui.
    Container(ResourceId),

    /// Asset canvas ui.
    Asset(ResourceId),
}

async fn resource_from_position(
    position: &tauri_sys::dpi::PhysicalPosition,
) -> Option<WorkspaceResource> {
    let monitor = tauri_sys::window::current_monitor().await.unwrap();
    let position = position.as_logical(monitor.scale_factor());
    let (x, y) = (position.x(), position.y());
    if analyses_from_point(x, y) {
        Some(WorkspaceResource::Analyses)
    } else if let Some(id) = container_from_point(x, y) {
        Some(WorkspaceResource::Container(id))
    } else {
        None
    }
}

/// Is the point within the analyses properties bar.
///
/// # Arguments
/// `x`, `y`: Logical size.
fn analyses_from_point(x: isize, y: isize) -> bool {
    document()
        .elements_from_point(x as f32, y as f32)
        .iter()
        .find(|elm| {
            let elm = elm.dyn_ref::<web_sys::Element>().unwrap();
            elm.id() == properties::ANALYSES_ID
        })
        .is_some()
}

/// Container the point is over.
///
/// # Arguments
/// `x`, `y`: Logical size.
fn container_from_point(x: isize, y: isize) -> Option<ResourceId> {
    document()
        .elements_from_point(x as f32, y as f32)
        .iter()
        .find_map(|elm| {
            let elm = elm.dyn_ref::<web_sys::Element>().unwrap();
            if let Some(kind) = elm.get_attribute("data-resource") {
                if kind == canvas::DATA_KEY_CONTAINER {
                    if let Some(rid) = elm.get_attribute("data-rid") {
                        let rid = ResourceId::from_str(&rid).unwrap();
                        return Some(rid);
                    }
                }

                None
            } else {
                None
            }
        })
}

async fn handle_drop_event(
    resource: WorkspaceResource,
    payload: DragDropPayload,
    project: &state::Project,
    graph: &state::Graph,
) {
    match resource {
        WorkspaceResource::Analyses => {
            handle_drop_event_analyses(payload, project.rid().get_untracked()).await
        }
        WorkspaceResource::Container(container) => {
            handle_drop_event_container(container, payload, project.rid().get_untracked(), graph)
                .await
        }
        WorkspaceResource::Asset(_) => todo!(),
    }
}

/// Handle drop event on a container.
async fn handle_drop_event_container(
    container: ResourceId,
    payload: DragDropPayload,
    project: ResourceId,
    graph: &state::Graph,
) {
    let container_node = graph.find_by_id(&container).unwrap();
    let container_path = graph.path(&container_node).unwrap();
    for res in add_fs_resources_to_graph(project, container_path, payload.paths().clone()).await {
        if let Err(err) = res {
            tracing::error!(?err);
            todo!();
        }
    }
}

/// Adds file system resources (file or folder) to the project's data graph.
async fn add_fs_resources_to_graph(
    project: ResourceId,
    parent: PathBuf,
    paths: Vec<PathBuf>,
) -> Vec<Result<(), io::ErrorKind>> {
    #[derive(Serialize)]
    struct Args {
        resources: Vec<lib::types::AddFsGraphResourceData>,
    }

    let resources = paths
        .into_iter()
        .map(|path| lib::types::AddFsGraphResourceData {
            project: project.clone(),
            path,
            parent: parent.clone(),
            action: local::types::FsResourceAction::Copy, // TODO: Get from user preferences.
        })
        .collect();

    tauri_sys::core::invoke::<Vec<Result<(), lib::command::error::IoErrorKind>>>(
        "add_file_system_resources",
        Args { resources },
    )
    .await
    .into_iter()
    .map(|res| res.map_err(|err| err.0))
    .collect()
}

/// Handle a drop event on the project analyses bar.
async fn handle_drop_event_analyses(payload: DragDropPayload, project: ResourceId) {
    for res in add_fs_resources_to_analyses(payload.paths().clone(), project).await {
        if let Err(err) = res {
            tracing::error!(?err);
            todo!();
        }
    }
}

async fn add_fs_resources_to_analyses(
    paths: Vec<PathBuf>,
    project: ResourceId,
) -> Vec<Result<(), local::error::IoSerde>> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        resources: Vec<lib::types::AddFsAnalysisResourceData>,
    }

    let resources = paths
        .into_iter()
        .map(|path| lib::types::AddFsAnalysisResourceData {
            path: path.clone(),
            parent: PathBuf::from("/"),
            action: local::types::FsResourceAction::Copy,
        })
        .collect();

    tauri_sys::core::invoke("add_scripts", Args { project, resources }).await
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

fn handle_event_project(event: lib::Event, project: state::Project) {
    let lib::EventKind::Project(update) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Project::Graph(_)
        | db::event::Project::Container { .. }
        | db::event::Project::Asset { .. }
        | db::event::Project::AssetFile(_)
        | db::event::Project::Flag { .. } => unreachable!("handled elsewhere"),

        db::event::Project::Removed => todo!(),
        db::event::Project::Moved(_) => todo!(),
        db::event::Project::Properties(_) => todo!(),
        db::event::Project::Settings(_) => todo!(),
        db::event::Project::Analyses(_) => handle_event_project_analyses(event, project),
        db::event::Project::AnalysisFile(_) => todo!(),
    }
}

fn handle_event_project_analyses(event: lib::Event, project: state::Project) {
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

fn handle_event_project_analyses_modified(event: lib::Event, project: state::Project) {
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
                analyses.push(state::Analysis::from_state(update_analysis));
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

fn handle_event_graph(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(update) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Project::Removed
        | db::event::Project::Moved(_)
        | db::event::Project::Properties(_)
        | db::event::Project::Settings(_)
        | db::event::Project::Analyses(_)
        | db::event::Project::AnalysisFile(_) => unreachable!("handled elsewhere"),

        db::event::Project::Graph(_) => handle_event_graph_graph(event, graph),
        db::event::Project::Container { .. } => handle_event_graph_container(event, graph),
        db::event::Project::Asset { .. } => handle_event_graph_asset(event, graph),
        db::event::Project::AssetFile(_) => handle_event_graph_asset_file(event, graph),
        db::event::Project::Flag { .. } => todo!(),
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
            .insert(
                common::normalize_path_sep(parent),
                state::Graph::new(subgraph.clone()),
            )
            .unwrap(),
        db::event::Graph::Renamed { from, to } => graph.rename(from, to).unwrap(),
        db::event::Graph::Moved { from, to } => todo!(),
        db::event::Graph::Removed(path) => graph.remove(common::normalize_path_sep(path)).unwrap(),
    }
}

fn handle_event_graph_container(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container { update, .. }) = event.kind() else {
        panic!("invalid event kind");
    };

    match update {
        db::event::Container::Properties(_) => {
            handle_event_graph_container_properties(event, graph)
        }
        db::event::Container::Settings(_) => handle_event_graph_container_settings(event, graph),
        db::event::Container::Assets(_) => handle_event_graph_container_assets(event, graph),
    }
}

fn handle_event_graph_container_properties(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        update: db::event::Container::Properties(update),
        ..
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => {
            handle_event_graph_container_properties_created(event, graph)
        }
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => todo!(),
        db::event::DataResource::Repaired(_) => todo!(),
        db::event::DataResource::Modified(update) => {
            handle_event_graph_container_properties_modified(event, graph)
        }
    }
}

fn handle_event_graph_container_properties_created(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Created(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    match update {
        Ok(update) => {
            if container
                .properties()
                .with_untracked(|properties| properties.is_err())
            {
                container.properties().update(|properties| {
                    *properties = db::state::DataResource::Ok(state::container::Properties::new(
                        update.rid.clone(),
                        update.properties.clone(),
                    ));
                });
            } else {
                update_container_properties(container, update);
            }
        }

        Err(err) => {
            if !container.properties().with(|properties| {
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

            if !container.analyses().with(|analyses| {
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

fn handle_event_graph_container_properties_modified(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Properties(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    update_container_properties(container, update);
}

fn update_container_properties(
    container: state::graph::Node,
    update: &local::types::StoredContainerProperties,
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
                if !metadata.iter().any(|(k, _)| k == key) {
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
                if !analyses
                    .iter()
                    .any(|association| association.analysis() == association_update.analysis())
                {
                    analyses.push(state::AnalysisAssociation::new(association_update.clone()));
                }
            }
        });
    });
}

fn handle_event_graph_container_settings(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Settings(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(path))
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
                        state::container::Settings::new(update.clone()),
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

fn handle_event_graph_container_assets(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => {
            handle_event_graph_container_assets_created(event, graph)
        }
        db::event::DataResource::Removed => todo!(),
        db::event::DataResource::Corrupted(_) => {
            handle_event_graph_container_assets_corrupted(event, graph)
        }
        db::event::DataResource::Repaired(_) => {
            handle_event_graph_container_assets_repaired(event, graph)
        }
        db::event::DataResource::Modified(_) => {
            handle_event_graph_container_assets_modified(event, graph)
        }
    }
}

fn handle_event_graph_container_assets_created(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Created(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(path))
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
                    .map(|asset| state::Asset::new(asset.clone()))
                    .collect();

                container
                    .assets()
                    .set(db::state::DataResource::Ok(create_rw_signal(assets)));
            } else {
                container.assets().update(|assets| {
                    let db::state::DataResource::Ok(assets) = assets else {
                        panic!("invalid state");
                    };

                    assets.update(|assets| {
                        assets.retain(|asset| {
                            update
                                .iter()
                                .any(|update| asset.rid().with_untracked(|rid| update.rid() == rid))
                        });

                        for asset_update in update.iter() {
                            if let Some(asset) = assets.iter().find(|asset| {
                                asset.rid().with_untracked(|rid| rid == asset_update.rid())
                            }) {
                                update_asset(asset, asset_update);
                            } else {
                                assets.push(state::Asset::new(asset_update.clone()));
                            }
                        }
                    });
                });
            }
        }
    }
}

fn handle_event_graph_container_assets_modified(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(path))
        .unwrap()
        .unwrap();

    container.assets().update(|assets| {
        let db::state::DataResource::Ok(assets) = assets else {
            panic!("invalid state");
        };

        assets.update(|assets| {
            assets.retain(|asset| {
                update
                    .iter()
                    .any(|update| asset.rid().with_untracked(|rid| update.rid() == rid))
            });

            for asset_update in update.iter() {
                if let Some(asset) = assets
                    .iter()
                    .find(|asset| asset.rid().with_untracked(|rid| rid == asset_update.rid()))
                {
                    update_asset(asset, asset_update);
                } else {
                    assets.push(state::Asset::new(asset_update.clone()));
                }
            }
        });
    });
}

fn update_asset(asset: &state::Asset, update: &db::state::Asset) {
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

    asset.metadata().update(|metadata| {
        metadata.retain(|(key, _)| update.properties.metadata.contains_key(key));

        for (update_key, value_update) in update.properties.metadata.iter() {
            if let Some(value) =
                metadata.iter().find_map(
                    |(key, value)| {
                        if key == update_key {
                            Some(value)
                        } else {
                            None
                        }
                    },
                )
            {
                if value.with_untracked(|value| value != value_update) {
                    value.update(|value| *value = value_update.clone());
                }
            } else {
                metadata.push((update_key.clone(), RwSignal::new(value_update.clone())));
            }
        }
    });
}

/// Workspace resource that is currently dragged over.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct DragOverCanvasResource(Option<WorkspaceResource>);
impl DragOverCanvasResource {
    pub fn new() -> Self {
        Self(None)
    }
}

fn handle_event_graph_container_assets_corrupted(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Corrupted(err)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph.find(path).unwrap().unwrap();
    container.assets().update(|container_assets| {
        *container_assets = db::state::DataResource::Err(err.clone());
    });
}

fn handle_event_graph_container_assets_repaired(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Repaired(assets)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let assets = assets
        .into_iter()
        .map(|asset| state::Asset::new(asset.clone()))
        .collect();

    let container = graph.find(path).unwrap().unwrap();
    container.assets().update(|container_assets| {
        *container_assets = db::state::DataResource::Ok(create_rw_signal(assets));
    });
}

fn handle_event_graph_asset(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Asset {
        container,
        asset,
        update,
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph
        .find(common::normalize_path_sep(container))
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
                        .unwrap()
                        .fs_resource()
                })
            });

            match update {
                db::event::Asset::FileCreated => fs_resource.set(db::state::FileResource::Present),
                db::event::Asset::FileRemoved => fs_resource.set(db::state::FileResource::Absent),
                _ => unreachable!(),
            };
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
                                if update_key == key {
                                    Some(value)
                                } else {
                                    None
                                }
                            }) {
                                if value.with_untracked(|value| value != update_value) {
                                    value.set(update_value.clone())
                                }
                            } else {
                                metadata.push((
                                    update_key.clone(),
                                    create_rw_signal(update_value.clone()),
                                ));
                            }
                        });
                });
            });
        }
    }
}

fn handle_event_graph_asset_file(event: lib::Event, graph: state::Graph) {
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
