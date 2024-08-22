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
                                view! { <WorkspaceView project_path project_data graph/> }
                            })
                            .or_else(|| Some(view! { <NoProject/> }))
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
            <ProjectNav/>
            <div class="border-b">
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
                        set_drag_over_workspace_resource(
                            resource_from_point(payload.position().x(), payload.position().y())
                                .into(),
                        );
                    }
                    DragDropEvent::Over(payload) => {
                        set_drag_over_workspace_resource(
                            resource_from_point(payload.position().x(), payload.position().y())
                                .into(),
                        );
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
                <LayersNav/>
            </div>
            <div class="flex-grow">
                <Canvas/>
            </div>
            <div class="w-1/6 border-l">
                <PropertiesBar/>
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
                        <Logo class="h-4"/>
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

fn resource_from_point(x: isize, y: isize) -> Option<WorkspaceResource> {
    if analyses_from_point(x, y) {
        Some(WorkspaceResource::Analyses)
    } else if let Some(id) = container_from_point(x, y) {
        Some(WorkspaceResource::Container(id))
    } else {
        None
    }
}

/// Is the point within the analyses properties bar.
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
            handle_drop_event_container(container, payload, project, graph).await
        }
        WorkspaceResource::Asset(_) => todo!(),
    }
}

/// Handle drop event on a container.
async fn handle_drop_event_container(
    container: ResourceId,
    payload: DragDropPayload,
    project: &state::Project,
    graph: &state::Graph,
) {
    let data_root = project
        .path()
        .get_untracked()
        .join(project.properties().data_root().get_untracked());

    let container_node = graph.find_by_id(&container).unwrap();
    let container_path = graph.path(&container_node).unwrap();
    let container_path = common::container_system_path(data_root, container_path);
    for res in add_fs_resources_to_graph(container_path, payload.paths().clone()).await {
        if let Err(err) = res {
            tracing::error!(?err);
            todo!();
        }
    }
}

/// Adds file system resources (file or folder) to the project's data graph.
async fn add_fs_resources_to_graph(
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
            if let Some(update_analysis) = update.iter().find(|update_analysis| {
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

                true
            } else {
                false
            }
        });

        for update_analysis in update.iter() {
            let update_properties = update_analysis.properties();
            if !analyses.iter().any(|analysis| {
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
            }) {
                analyses.push(state::Analysis::from_state(update_analysis));
            }
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
        db::event::Project::Container { path, update } => {
            handle_event_graph_container(event, graph)
        }
        db::event::Project::Asset {
            container,
            asset,
            update,
        } => todo!(),
        db::event::Project::AssetFile(_) => handle_event_graph_asset(event, graph),
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
        db::event::Container::Assets(_) => handle_event_graph_container_assets(event, graph),
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

                tracing::debug!("cp 2a");
                if properties
                    .kind()
                    .with_untracked(|kind| update.properties.kind != *kind)
                {
                    tracing::debug!("cp 2a1");
                    properties.kind().set(update.properties.kind.clone());
                    tracing::debug!("cp 2a2");
                }

                tracing::debug!("cp 2b");
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

fn handle_event_graph_container_assets(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(update),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    match update {
        db::event::DataResource::Created(_) => todo!(),
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

fn handle_event_graph_container_assets_modified(event: lib::Event, graph: state::Graph) {
    let lib::EventKind::Project(db::event::Project::Container {
        path,
        update: db::event::Container::Assets(db::event::DataResource::Modified(update)),
    }) = event.kind()
    else {
        panic!("invalid event kind");
    };

    let container = graph.find(path).unwrap().unwrap();
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

    match update {
        syre_local_database::event::Asset::FileCreated => todo!(),
        syre_local_database::event::Asset::FileRemoved => todo!(),
    }
}
