use futures::stream::StreamExt;
use leptos::{prelude::*, task::spawn_local};
use project_bar::ProjectBar;
use properties::PropertiesBar;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_desktop_ui_components::{Drawer, drawer};
use syre_desktop_ui_lib as ui_lib;
use syre_local as local;
use tauri_sys::window::DragDropPayload;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
enum EditorKind {
    Project,
    Analyses,
}

impl Default for EditorKind {
    fn default() -> Self {
        Self::Analyses
    }
}

#[component]
pub fn Workspace() -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let messages = expect_context::<ui_lib::message::Messages>();
    provide_context(DragOverWorkspaceResource::new());
    provide_context(RwSignal::new(EditorKind::default()));

    let (drag_over_event, set_drag_over_event) = signal(tauri_sys::window::DragDropEvent::Leave);
    let drag_over_event =
        leptos_use::signal_throttled(drag_over_event, ui_lib::common::THROTTLE_DRAG_EVENT);
    let drag_over_workspace_resource = RwSignal::new(DragOverWorkspaceResource::new());
    provide_context(drag_over_workspace_resource.read_only());

    let _ = Effect::watch(
        move || drag_over_event.get(),
        {
            let project = project.clone();
            move |event, _, _| {
                handle_drag_drop_event(event, drag_over_workspace_resource, &project, messages)
            }
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
        <div class="grow flex flex-col">
            <div class="border-b not-dark:border-b-secondary-900">
                <ProjectBar />
            </div>
            <div class="flex grow min-h-0">
                <div class="grow flex min-h-0 relative overflow-hidden">
                    <div class="grow text-center pt-4">"Data graph does not exist."</div>
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

mod project_bar {
    use super::EditorKind;
    use leptos::{ev::MouseEvent, prelude::*};
    use leptos_icons::Icon;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn ProjectBar() -> impl IntoView {
        view! {
            <div class="flex px-2 py-1">
                <div class="w-1/3"></div>
                <div class="w-1/3 text-center">
                    <ProjectInfo />
                </div>
                <div class="w-1/3 text-right">
                    <Controls />
                </div>
            </div>
        }
    }

    #[component]
    fn ProjectInfo() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let properties_editor = expect_context::<RwSignal<EditorKind>>();

        let mousedown = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            if properties_editor.with(|editor| matches!(*editor, EditorKind::Project)) {
                // TODO: Return properties to widget based on graph selection.
                // Currenlty the graph and selection state contexts are descendants, so can not access them.
                properties_editor.set(EditorKind::Analyses.into());
            } else {
                properties_editor.set(EditorKind::Project.into());
            }
        };

        view! {
            <div on:mousedown=mousedown class="grow text-center font-primary cursor-pointer">
                {project.properties().name()}
            </div>
        }
    }

    #[component]
    fn Controls() -> impl IntoView {
        let refresh = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            let window = web_sys::window().unwrap();
            window.location().reload().unwrap();
        };

        view! {
            <button
                on:mousedown=refresh
                type="button"
                class="btn-secondary p-1 rounded-xs cursor-pointer"
                title="Refresh"
            >
                <Icon icon=ui_lib::icon::Refresh />
            </button>
        }
    }
}

mod properties {
    use super::{
        super::analyses::{ANALYSES_ID, Editor as Analyses},
        EditorKind,
    };
    use leptos::{either::either, prelude::*};
    use reactive_stores::Store;
    use syre_desktop_lib as lib;
    use syre_desktop_ui_lib as ui_lib;

    #[derive(derive_more::Deref, Clone, Copy)]
    pub struct InputDebounce(Signal<f64>);

    #[component]
    pub fn PropertiesBar() -> impl IntoView {
        let user_settings = expect_context::<Store<ui_lib::state::settings::User>>();
        let active_editor = expect_context::<RwSignal<EditorKind>>();
        provide_context(InputDebounce(Signal::derive(move || {
            user_settings.with(|settings| {
                let debounce = match &settings.desktop {
                    Ok(settings) => settings.input_debounce_ms,
                    Err(_) => lib::settings::user::Desktop::default().input_debounce_ms,
                };

                debounce as f64
            })
        })));

        let widget = {
            move || {
                either!( *active_editor.read(),
                    EditorKind::Project => syre_desktop_editors::project::Editor,
                    EditorKind::Analyses => view! {
                        <div id=ANALYSES_ID class="h-full">
                            <Analyses/>
                        </div>
                    },
                )
            }
        };

        view! { <div class="h-full relative">{widget}</div> }
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
    } else {
        None
    }
}

/// Is the point within the analyses properties bar.
///
/// # Arguments
/// `x`, `y`: Logical size.
fn analyses_from_point(x: isize, y: isize) -> bool {
    use super::analyses::ANALYSES_ID;

    document()
        .elements_from_point(x as f32, y as f32)
        .iter()
        .find(|elm| {
            let elm = elm.dyn_ref::<web_sys::Element>().unwrap();
            elm.id() == ANALYSES_ID
        })
        .is_some()
}

// TODO: Tested on Linux and Windows.
// Need to test on Mac.
// Check if needed on unix systems.
fn handle_drag_drop_event(
    event: &tauri_sys::window::DragDropEvent,
    drag_over_workspace_resource: RwSignal<DragOverWorkspaceResource>,
    project: &ui_lib::state::Project,
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
                let resource = resource_from_position(payload.position())
                    .await
                    .map(|(resource, _)| resource);
                if **drag_over_workspace_resource.read_untracked() != resource {
                    drag_over_workspace_resource.set(resource.into());
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
                }
            });
        }
        DragDropEvent::Leave => {
            // Cursor exited window
            if drag_over_workspace_resource.read_untracked().is_some() {
                drag_over_workspace_resource.set(None.into());
            }
        }
        DragDropEvent::Drop(payload) => {
            if let Some(resource) = drag_over_workspace_resource.get_untracked().into_inner() {
                drag_over_workspace_resource.set(None.into());

                // NB: Spawn seperate task to handle large copies.
                spawn_local({
                    let project = project.clone();
                    let payload = payload.clone();
                    async move { handle_drop_event(resource, payload, &project, messages).await }
                });
            }
        }
    }
}

async fn handle_drop_event(
    resource: WorkspaceResource,
    payload: DragDropPayload,
    project: &ui_lib::state::Project,
    messages: ui_lib::message::Messages,
) {
    match resource {
        WorkspaceResource::Analyses => {
            handle_drop_event_analyses(payload, project.rid().get_untracked(), messages).await
        }
    }
}

/// Handle a drop event on the project analyses bar.
async fn handle_drop_event_analyses(
    payload: DragDropPayload,
    project: ResourceId,
    messages: ui_lib::message::Messages,
) {
    use syre_desktop_ui_lib::common::FS_RESOURCE_ACTION_NOTIFY_THRESHOLD;

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
        let msg = msg.build();
        messages.push_message(msg);
    }

    match add_fs_resources_to_analyses(payload.paths().clone(), project).await {
        Ok(_) => {
            if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
                let msg = ui_lib::message::Builder::success("Analyses added.");
                let msg = msg.build();
                messages.push_message(msg);
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
