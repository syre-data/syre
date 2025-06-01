use super::{Canvas, NavBar, ProjectBar, PropertiesBar, canvas, properties};
use futures::stream::StreamExt;
use leptos::{prelude::*, task::spawn_local};
use serde::Serialize;
use std::{io, path::PathBuf, str::FromStr};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_desktop_ui_components::{Drawer, drawer};
use syre_desktop_ui_lib as ui_lib;
use syre_local as local;
use tauri_sys::window::DragDropPayload;
use wasm_bindgen::JsCast;

#[component]
pub fn Workspace() -> impl IntoView {
    let graph = expect_context::<ui_lib::state::Graph>();
    let project = expect_context::<ui_lib::state::Project>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let viewbox = ViewboxState::default();
    provide_context(viewbox.clone());
    provide_context(DragOverWorkspaceResource::new());
    provide_context(RwSignal::new(properties::EditorKind::default()));

    let (drag_over_event, set_drag_over_event) = signal(tauri_sys::window::DragDropEvent::Leave);
    let drag_over_event =
        leptos_use::signal_throttled(drag_over_event, ui_lib::common::THROTTLE_DRAG_EVENT);
    let drag_over_container_elm = RwSignal::new_local(None);
    let drag_over_workspace_resource = RwSignal::new(DragOverWorkspaceResource::new());
    provide_context(drag_over_workspace_resource.read_only());

    let _ = Effect::watch(
        move || drag_over_event.get(),
        {
            let project = project.clone();
            let graph = graph.clone();
            move |event, _, _| {
                handle_drag_drop_event(
                    event,
                    drag_over_container_elm,
                    drag_over_workspace_resource,
                    &project,
                    &graph,
                    messages,
                )
            }
        },
        false,
    );

    let _ = Effect::watch(
        drag_over_container_elm.read_only(),
        move |elm, prev_container, _| {
            if let Some(elm) = prev_container {
                if let Some(container) = elm.as_ref() {
                    let event = web_sys::Event::new("dragleave_windows").unwrap();
                    container.dispatch_event(&event).unwrap();
                }
            }

            if let Some(container) = elm.as_ref() {
                let event = web_sys::Event::new("dragenter_windows").unwrap();
                container.dispatch_event(&event).unwrap();
            }

            elm.clone()
        },
        false,
    );

    spawn_local(async move {
        let window = tauri_sys::window::get_current();
        let mut listener = window.on_drag_drop_event().await.unwrap();
        while let Some(event) = listener.next().await {
            set_drag_over_event(event.payload);
        }
    });

    view! {
        <div class="flex flex-col h-full">
            <div class="border-b not-dark:border-b-secondary-900">
                <ProjectBar />
            </div>
            <div class="flex grow min-h-0">
                <div class="grow flex min-h-0 relative overflow-hidden">
                    <Drawer
                        dock=drawer::Dock::East
                        absolute=true
                        class="min-w-28 max-w-[40%] bg-white dark:bg-secondary-800 w-1/6 border-r \
                        not-dark:border-r-secondary-900"
                    >
                        <NavBar />
                    </Drawer>
                    <div class="grow">
                        <Canvas />
                    </div>
                    <Drawer
                        dock=drawer::Dock::West
                        absolute=true
                        class="min-w-28 max-w-[40%] bg-white dark:bg-secondary-800 w-1/6 border-l \
                        not-dark:border-l-secondary-900"
                    >
                        <PropertiesBar />
                    </Drawer>
                </div>
            </div>
        </div>
    }
}

#[derive(derive_more::Deref, derive_more::From, Clone, PartialEq)]
pub struct DragOverWorkspaceResource(Option<WorkspaceResource>);
impl DragOverWorkspaceResource {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn into_inner(self) -> Option<WorkspaceResource> {
        self.0
    }
}

#[derive(PartialEq, Clone)]
pub enum WorkspaceResource {
    /// Analyses properties bar.
    Analyses,

    /// Container canvas ui.
    Container(ResourceId),

    /// Asset canvas ui.
    Asset(ResourceId),
}

/// State of an `svg` `viewbox` atribute.
///
/// Used for [`Canvas`].
#[derive(Debug, Clone)]
pub struct ViewboxState {
    x: RwSignal<isize>,
    y: RwSignal<isize>,
    width: RwSignal<usize>,
    height: RwSignal<usize>,
}

impl ViewboxState {
    pub fn x(&self) -> &RwSignal<isize> {
        &self.x
    }

    pub fn y(&self) -> &RwSignal<isize> {
        &self.y
    }

    pub fn width(&self) -> &RwSignal<usize> {
        &self.width
    }

    pub fn height(&self) -> &RwSignal<usize> {
        &self.height
    }
}

impl Default for ViewboxState {
    fn default() -> Self {
        use super::canvas;

        Self {
            x: RwSignal::new(0),
            y: RwSignal::new(0),
            width: RwSignal::new(canvas::VB_BASE),
            height: RwSignal::new(canvas::VB_BASE),
        }
    }
}

impl std::fmt::Display for ViewboxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.x.get_untracked(),
            self.y.get_untracked(),
            self.width.get_untracked(),
            self.height.get_untracked()
        )
    }
}

/// Get a resource from a location on screen.
///
/// # Returns
/// `Some(resource, Option<element>)` if the position represents a resource.
/// `Option<element>` is `Some` for container resources, where `element` is the DOM
/// element representing the container.
async fn resource_from_position(
    position: &tauri_sys::dpi::PhysicalPosition,
) -> Option<(WorkspaceResource, Option<web_sys::Element>)> {
    let monitor = tauri_sys::window::current_monitor().await.unwrap();
    let position = position.as_logical(monitor.scale_factor());
    let (x, y) = (position.x(), position.y());
    if analyses_from_point(x, y) {
        Some((WorkspaceResource::Analyses, None))
    } else if let Some((id, elm)) = container_from_point(x, y) {
        Some((WorkspaceResource::Container(id), Some(elm)))
    } else {
        None
    }
}

/// Is the point within the analyses properties bar.
///
/// # Arguments
/// `x`, `y`: Logical size.
fn analyses_from_point(x: isize, y: isize) -> bool {
    use super::properties::analyses::ANALYSES_ID;

    document()
        .elements_from_point(x as f32, y as f32)
        .iter()
        .find(|elm| {
            let elm = elm.dyn_ref::<web_sys::Element>().unwrap();
            elm.id() == ANALYSES_ID
        })
        .is_some()
}

/// Container the point is over.
///
/// # Arguments
/// `x`, `y`: Logical size.
///
/// # Returns
/// `Some((id, elm))` if the point is over a valid container.`
fn container_from_point(x: isize, y: isize) -> Option<(ResourceId, web_sys::Element)> {
    document()
        .elements_from_point(x as f32, y as f32)
        .iter()
        .find_map(|elm| {
            let elm = elm.dyn_ref::<web_sys::Element>().unwrap();
            if let Some(kind) = elm.get_attribute("data-resource") {
                if kind == canvas::DATA_KEY_CONTAINER {
                    if let Some(rid) = elm.get_attribute("data-rid") {
                        let rid = ResourceId::from_str(&rid).unwrap();
                        return Some((rid, elm.clone()));
                    }
                }

                None
            } else {
                None
            }
        })
}

// TODO: Tested on Linux and Windows.
// Need to test on Mac.
// Check if needed on unix systems.
fn handle_drag_drop_event(
    event: &tauri_sys::window::DragDropEvent,
    drag_over_container_elm: RwSignal<Option<web_sys::Element>, LocalStorage>,
    drag_over_workspace_resource: RwSignal<DragOverWorkspaceResource>,
    project: &ui_lib::state::Project,
    graph: &ui_lib::state::Graph,
    messages: ui_lib::message::Messages,
) {
    use tauri_sys::window::DragDropEvent;

    match event {
        DragDropEvent::Enter(payload) => {
            // Cursor entered window
            if payload.paths().is_empty() {
                return;
            }

            let payload = payload.clone();
            spawn_local(async move {
                let (resource, elm) = match resource_from_position(payload.position()).await {
                    None => (None, None),
                    Some((resource, elm)) => (Some(resource), elm),
                };
                if **drag_over_workspace_resource.read_untracked() != resource {
                    drag_over_workspace_resource.set(resource.into());
                    if let Some(container) = elm {
                        let _ = drag_over_container_elm.write().insert(container);
                    }
                }
            });
        }
        DragDropEvent::Over(payload) => {
            let payload = payload.clone();
            spawn_local(async move {
                let (resource, elm) = match resource_from_position(payload.position()).await {
                    None => (None, None),
                    Some((resource, elm)) => (Some(resource), elm),
                };
                if **drag_over_workspace_resource.read_untracked() != resource {
                    drag_over_workspace_resource.set(resource.into());
                    if let Some(container) = elm {
                        let _ = drag_over_container_elm.write().insert(container);
                    } else {
                        let _ = drag_over_container_elm.write().take();
                    }
                }
            });
        }
        DragDropEvent::Leave => {
            // Cursor exited window
            if drag_over_workspace_resource.read_untracked().is_some() {
                drag_over_workspace_resource.set(None.into());
                let _ = drag_over_container_elm.write().take();
            }
        }
        DragDropEvent::Drop(payload) => {
            if let Some(resource) = drag_over_workspace_resource.get_untracked().into_inner() {
                drag_over_workspace_resource.set(None.into());
                let _ = drag_over_container_elm.write().take();

                // NB: Spawn seperate task to handle large copies.
                spawn_local({
                    let project = project.clone();
                    let graph = graph.clone();
                    let payload = payload.clone();
                    async move { handle_drop_event(resource, payload, &project, &graph, messages).await }
                });
            }
        }
    }
}

async fn handle_drop_event(
    resource: WorkspaceResource,
    payload: DragDropPayload,
    project: &ui_lib::state::Project,
    graph: &ui_lib::state::Graph,
    messages: ui_lib::message::Messages,
) {
    match resource {
        WorkspaceResource::Analyses => {
            handle_drop_event_analyses(payload, project.rid().get_untracked(), messages).await
        }
        WorkspaceResource::Container(container) => {
            handle_drop_event_container(
                container,
                payload,
                project.rid().get_untracked(),
                graph,
                messages,
            )
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
    graph: &ui_lib::state::Graph,
    messages: ui_lib::message::Messages,
) {
    use ui_lib::common::FS_RESOURCE_ACTION_NOTIFY_THRESHOLD;

    let container_node = graph.find_by_id(&container).unwrap();
    let container_path = graph.path(&container_node).unwrap();

    let transfer_size = match ui_lib::commands::fs::file_size(payload.paths().clone())
        .await
        .map(|sizes| {
            sizes
                .into_iter()
                .reduce(|total, size| total + size)
                .unwrap_or(0)
        }) {
        Ok(size) => size,
        Err(err) => {
            tracing::error!(?err);
            0
        }
    };

    if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
        let msg = ui_lib::message::Builder::info("Transferring files.");
        messages.push_message(msg.build());
    }

    match add_fs_resources_to_graph(project, container_path, payload.paths().clone()).await {
        Ok(_) => {
            if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
                let msg = ui_lib::message::Builder::success("File transfer complete.");
                messages.push_message(msg.build());
            }
        }
        Err(errors) => {
            tracing::error!(?errors);
            todo!();
        }
    }
}

/// Adds file system resources (file or folder) to the project's data graph.
async fn add_fs_resources_to_graph(
    project: ResourceId,
    parent: PathBuf,
    paths: Vec<PathBuf>,
) -> Result<(), Vec<(PathBuf, io::ErrorKind)>> {
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

    tauri_sys::core::invoke_result::<(), Vec<(PathBuf, lib::command::error::IoErrorKind)>>(
        "add_file_system_resources",
        Args { resources },
    )
    .await
    .map_err(|errors| {
        errors
            .into_iter()
            .map(|(path, err)| (path, err.0))
            .collect()
    })
}

/// Handle a drop event on the project analyses bar.
async fn handle_drop_event_analyses(
    payload: DragDropPayload,
    project: ResourceId,
    messages: ui_lib::message::Messages,
) {
    use ui_lib::common::FS_RESOURCE_ACTION_NOTIFY_THRESHOLD;

    let transfer_size = match ui_lib::commands::fs::file_size(payload.paths().clone()).await {
        Ok(sizes) => sizes
            .into_iter()
            .reduce(|total, size| total + size)
            .unwrap_or(0),
        Err(err) => {
            tracing::error!(?err);
            0
        }
    };

    if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
        let msg = ui_lib::message::Builder::info("Adding analyses.");
        messages.push_message(msg.build());
    }

    match add_fs_resources_to_analyses(payload.paths().clone(), project).await {
        Ok(_) => {
            if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
                let msg = ui_lib::message::Builder::success("Analyses added.");
                messages.push_message(msg.build());
            }
        }
        Err(err) => {
            let msg = ui_lib::message::Builder::error("Could not add analyses.");
            let msg = msg.body(format!("{err:?}"));
            messages.push_message(msg.build_str());
        }
    }
}

async fn add_fs_resources_to_analyses(
    paths: Vec<PathBuf>,
    project: ResourceId,
) -> Result<(), lib::command::analyses::error::AddAnalyses> {
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

    tauri_sys::core::invoke_result("project_add_analyses", Args { project, resources }).await
}
