use super::{
    super::{THROTTLE_DRAG_EVENT, editors},
    Canvas, NavBar, ProjectBar, PropertiesBar, canvas, properties,
};
use crate::{
    commands, common,
    components::{self, Drawer, Logo, drawer},
    pages::project::{Settings, state},
    types,
};
use futures::stream::StreamExt;
use leptos::{
    either::{Either, either},
    ev::MouseEvent,
    html,
    portal::Portal,
    prelude::*,
    task::spawn_local,
};
use leptos_icons::*;
use leptos_router::{components::A, hooks::use_params_map};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_lib as lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_project_watcher as db;
use tauri_sys::window::DragDropPayload;
use wasm_bindgen::JsCast;

#[component]
pub fn Workspace() -> impl IntoView {
    let graph = expect_context::<state::Graph>();
    let project = expect_context::<state::Project>();
    let messages = expect_context::<types::Messages>();
    let viewbox = ViewboxState::default();
    provide_context(viewbox.clone());
    provide_context(DragOverWorkspaceResource::new());
    provide_context(RwSignal::new(properties::EditorKind::default()));
    let analyze_node = NodeRef::<html::Div>::new();

    let (drag_over_event, set_drag_over_event) = signal(tauri_sys::window::DragDropEvent::Leave);
    let drag_over_event = leptos_use::signal_throttled(drag_over_event, THROTTLE_DRAG_EVENT);
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
        <div>
            <div class="border-b not-dark:border-b-secondary-900">
                <ProjectBar analyze_node />
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

                {move || {
                    if let Some(analyze_node) = analyze_node.get() {
                        let mount = (*analyze_node).clone();
                        Either::Left(
                            view! {
                                <Portal mount>
                                    <analyze::Analyze />
                                </Portal>
                            },
                        )
                    } else {
                        Either::Right(())
                    }
                }}
            </div>
        </div>
    }
}

mod analyze {
    use super::state;
    use crate::{
        components,
        types::{
            self,
            settings::{
                project::SettingsStoreFields as ProjectSettingsStoreFields,
                user::SettingsStoreFields as UserSettingsStoreFields,
            },
        },
    };
    use futures::stream::StreamExt;
    use leptos::{ev::MouseEvent, prelude::*, task::spawn_local};
    use leptos_icons::*;
    use reactive_stores::Store;
    use std::path::PathBuf;
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_local::types::AnalysisKind;
    use syre_project_watcher as db;

    enum AnalysisState {
        Idle,
        Pending,
        Running { completed: usize, remaining: usize },
        Cancelling { completed: usize, remaining: usize },
        Killing { completed: usize, remaining: usize },
    }

    impl AnalysisState {
        pub fn active(&self) -> bool {
            match self {
                AnalysisState::Idle | AnalysisState::Pending => false,
                AnalysisState::Running { .. }
                | AnalysisState::Cancelling { .. }
                | AnalysisState::Killing { .. } => true,
            }
        }

        pub fn running(&self) -> bool {
            matches!(self, AnalysisState::Running { .. })
        }
    }

    #[component]
    pub fn Analyze() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<types::Messages>();
        let user_settings = expect_context::<Store<types::settings::User>>();
        let project_settings = expect_context::<Store<types::settings::Project>>();
        let analysis_state = RwSignal::new(AnalysisState::Idle);
        provide_context(analysis_state);

        let disable_analysis_after = {
            let user_settings = user_settings.analysis();
            let project_settings = project_settings.analysis();
            move || {
                project_settings
                    .read_untracked()
                    .as_ref()
                    .ok()
                    .map(|settings| settings.disable_analysis_after)
                    .flatten()
                    .or(user_settings
                        .read_untracked()
                        .as_ref()
                        .ok()
                        .map(|settings| settings.disable_analysis_after))
                    .unwrap_or(lib::settings::analysis::DisableAnalysisAfter::default())
            }
        };

        let action: Action<_, _> = Action::new_unsync({
            let analyses = project.analyses();
            let project = project.rid().read_only();
            move |root: &PathBuf| {
                let graph = graph.clone();
                let root = root.clone();
                async move {
                    analysis_state.set(AnalysisState::Pending);
                    let rx: tauri_sys::core::Channel<lib::event::analysis::Update> =
                        match trigger_analysis(
                            project.get_untracked(),
                            root,
                            disable_analysis_after(),
                        )
                        .await
                        {
                            Ok(rx) => rx,
                            Err(err) => {
                                tracing::error!(?err);
                                analysis_state.set(AnalysisState::Idle);
                                let mut msg = types::message::Builder::error(
                                    "Could not initialize analysis.",
                                );
                                msg.body(format!("{err:?}"));
                                messages.update(|messages| messages.push(msg.build()));
                                return;
                            }
                        };

                    spawn_local(handle_analysis_updates(
                        rx,
                        analysis_state,
                        analyses,
                        graph.clone(),
                        messages,
                    ));
                }
            }
        });

        view! {
            <Show
                when=move || analysis_state.with(|state| state.active())
                fallback=move || view! { <Trigger action /> }
            >
                <Analyzing />
            </Show>
        }
    }

    #[component]
    fn Trigger(action: Action<PathBuf, ()>) -> impl IntoView {
        let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
        let graph = expect_context::<state::Graph>();

        let single_container_selected = {
            let selected_resources = workspace_graph_state.selection_resources().selected();
            move || {
                selected_resources.with(|resources| {
                    resources.len() == 1
                        && matches!(
                            resources[0].kind(),
                            state::workspace_graph::ResourceKind::Container
                        )
                        && resources[0].rid().with_untracked(|rid| {
                            graph.root().properties().with_untracked(|properties| {
                                properties.as_ref().map_or(true, |properties| {
                                    properties.rid().with_untracked(|root| root != rid)
                                })
                            })
                        })
                })
            }
        };

        let trigger_analysis = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            action.dispatch(PathBuf::from("/"));
        };

        view! {
            <div class="flex">
                <button
                    on:mousedown=trigger_analysis
                    class:rounded-r={
                        let single_container_selected = single_container_selected.clone();
                        move || !single_container_selected()
                    }
                    class="flex gap-2 items-center btn-primary rounded-l px-4 \
                    disabled:bg-primary-800 dark:disabled:bg-primary-400 \
                    disabled:cursor-not-allowed"
                    disabled={
                        let pending = action.pending();
                        move || pending.get()
                    }
                >
                    "Analyze"
                </button>
                <Show when=single_container_selected fallback=|| ()>
                    <TriggerActions action />
                </Show>
            </div>
        }
    }

    #[component]
    fn TriggerActions(action: Action<PathBuf, ()>) -> impl IntoView {
        let workspace_graph_state = expect_context::<state::WorkspaceGraph>();
        let graph = expect_context::<state::Graph>();
        let selected_resources = workspace_graph_state.selection_resources().selected();

        let trigger_analysis_from_root = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            action.dispatch(PathBuf::from("/"));
        };

        let trigger_analysis_from_container = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            let root = selected_resources.with_untracked(|resources| {
                resources[0].rid().with_untracked(|rid| {
                    let root = graph.find_by_id(rid).unwrap();
                    graph.path(&root).unwrap()
                })
            });

            action.dispatch(PathBuf::from(root));
        };

        view! {
            <div class="group relative">
                <div class="flex items-center h-full btn-primary rounded-r px-1 border-l border-l-primary-800">
                    <Icon icon=components::icon::ChevronDown />
                </div>
                <div class="absolute top-full left-0 group-[&:not(:hover)]:hidden \
                z-10 rounded-b rounded-r border border-secondary-800 dark:border-secondary-200 \
                bg-white dark:bg-secondary-700">
                    <ul>
                        <li class="hover:bg-100 dark:hover:bg-secondary-800">
                            <button
                                on:click=trigger_analysis_from_container
                                class="px-2 text-nowrap"
                                title="Only analyze the subtree from the selected root container."
                            >
                                "From container"
                            </button>
                        </li>
                        <li class="hover:bg-100 dark:hover:bg-secondary-800">
                            <button
                                on:click=trigger_analysis_from_root
                                class="px-2"
                                title="Aanlyze the entire project."
                            >
                                "Project"
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        }
    }

    #[component]
    fn Analyzing() -> impl IntoView {
        let analysis_state = expect_context::<RwSignal<AnalysisState>>();
        let title = move || {
            analysis_state.with(|state| match state {
                AnalysisState::Idle | AnalysisState::Pending => unreachable!(),
                AnalysisState::Running {
                    completed,
                    remaining,
                }
                | AnalysisState::Cancelling {
                    completed,
                    remaining,
                }
                | AnalysisState::Killing {
                    completed,
                    remaining,
                } => format!("{} of {} remaining", remaining, completed + remaining),
            })
        };

        let percent_complete = move || {
            analysis_state.with(|state| match state {
                AnalysisState::Idle | AnalysisState::Pending => unreachable!(),
                AnalysisState::Running {
                    completed,
                    remaining,
                }
                | AnalysisState::Cancelling {
                    completed,
                    remaining,
                }
                | AnalysisState::Killing {
                    completed,
                    remaining,
                } => {
                    let total = completed + remaining;
                    if total == 0 {
                        "100%".to_string()
                    } else {
                        let percent_complete = 100 * completed / total;
                        format!("{percent_complete}%")
                    }
                }
            })
        };

        let text = move || {
            analysis_state.with(|state| match state {
                AnalysisState::Running { .. } => "Analyzing",
                AnalysisState::Cancelling { .. } => "Cancelling",
                AnalysisState::Killing { .. } => "Killing",
                _ => panic!("invalid state"),
            })
        };

        view! {
            <div class="flex">
                <button
                    class="relative btn-primary rounded-l px-4 cursor-not-allowed"
                    class:rounded-r=move || analysis_state.with(|state| !state.running())
                    title=title
                    disabled=true
                >
                    <div class="flex gap-2 items-center">
                        {text} <span class="animate-spin">
                            <Icon icon=components::icon::Refresh />
                        </span>
                    </div>
                    <div class="absolute bottom-0 left-1 right-1 h-0.5 rounded-full bg-primary-800 dark:bg-primary-700">
                        <span
                            class="h-full block rounded-full bg-syre-green-800 dark:bg-syre-green-500"
                            style:width=percent_complete
                        ></span>
                    </div>
                </button>
                <Show when=move || analysis_state.with(|state| state.running()) fallback=|| ()>
                    <AnalyzingActions />
                </Show>
            </div>
        }
    }

    #[component]
    fn AnalyzingActions() -> impl IntoView {
        let analysis_state = expect_context::<RwSignal<AnalysisState>>();
        let cancel_analysis = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            analysis_state.update(|state| {
                let AnalysisState::Running {
                    completed,
                    remaining,
                } = state
                else {
                    panic!("invalid state");
                };

                *state = AnalysisState::Cancelling {
                    completed: *completed,
                    remaining: *remaining,
                }
            });
            spawn_local(async { cancel_analysis().await });
        };

        let kill_analysis = move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            analysis_state.update(|state| {
                let AnalysisState::Running {
                    completed,
                    remaining,
                } = state
                else {
                    panic!("invalid state");
                };

                *state = AnalysisState::Killing {
                    completed: *completed,
                    remaining: *remaining,
                }
            });
            spawn_local(async { kill_analysis().await });
        };

        view! {
            <div class="group relative">
                <div class="flex items-center h-full btn-primary rounded-r px-1 border-l border-l-primary-800">
                    <Icon icon=components::icon::ChevronDown />
                </div>
                <div class="absolute top-full left-0 group-[&:not(:hover)]:hidden \
                z-10 rounded-b rounded-r border border-secondary-800 dark:border-secondary-200 \
                bg-white dark:bg-secondary-700">
                    <ul>
                        <li class="hover:bg-100 dark:hover:bg-secondary-800">
                            <button
                                on:click=cancel_analysis
                                class="px-2"
                                title="Cancel all remaining analyses, allowing those currently running to finish."
                            >
                                "Cancel"
                            </button>
                        </li>
                        <li class="hover:bg-100 dark:hover:bg-secondary-800">
                            <button
                                on:click=kill_analysis
                                class="px-2"
                                title="Immediately kill all analyses, even those currently running."
                            >
                                "Kill"
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        }
    }

    async fn trigger_analysis(
        project: ResourceId,
        root: impl Into<PathBuf>,
        disable_analysis_after: lib::settings::analysis::DisableAnalysisAfter,
    ) -> Result<
        tauri_sys::core::Channel<lib::event::analysis::Update>,
        lib::command::project::error::TriggerAnalysis,
    > {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Args<'a> {
            rx: &'a tauri_sys::core::Channel<lib::event::analysis::Update>,
            project: ResourceId,
            root: PathBuf,
            disable_analysis_after: lib::settings::analysis::DisableAnalysisAfter,
        }

        let rx = tauri_sys::core::Channel::new();
        tauri_sys::core::invoke_result::<(), lib::command::project::error::TriggerAnalysis>(
            "trigger_analysis",
            Args {
                rx: &rx,
                project,
                root: root.into(),
                disable_analysis_after,
            },
        )
        .await?;

        Ok(rx)
    }

    async fn handle_analysis_updates(
        mut rx: tauri_sys::core::Channel<lib::event::analysis::Update>,
        analysis_state: RwSignal<AnalysisState>,
        analyses: RwSignal<db::state::DataResource<RwSignal<Vec<state::Analysis>>>>,
        graph: state::Graph,
        messages: types::Messages,
    ) {
        while let Some(event) = rx.next().await {
            match event {
                lib::event::analysis::Update::Progress {
                    completed: update_completed,
                    remaining: update_remaining,
                } => analysis_state.update(|state| match state {
                    AnalysisState::Idle => panic!("invalid state"),

                    AnalysisState::Pending => {
                        *state = AnalysisState::Running {
                            completed: update_completed,
                            remaining: update_remaining,
                        }
                    }

                    AnalysisState::Running {
                        completed,
                        remaining,
                    }
                    | AnalysisState::Cancelling {
                        completed,
                        remaining,
                    }
                    | AnalysisState::Killing {
                        completed,
                        remaining,
                    } => {
                        *completed = update_completed;
                        *remaining = update_remaining;
                    }
                }),

                lib::event::analysis::Update::Done(status) => {
                    analysis_state.set(AnalysisState::Idle);
                    let errors = status
                        .iter()
                        .filter(|status| {
                            status
                                .output()
                                .map(|output| !output.status.success())
                                .unwrap_or(false)
                        })
                        .collect::<Vec<_>>();

                    if errors.is_empty() {
                        let msg = types::message::Builder::success("Analysis complete.");
                        messages.update(|messages| messages.push(msg.build()));
                    } else {
                        let mut msg =
                            types::message::Builder::error("Errors occurred during analysis.");
                        msg.body(view! {
                            <ol class="list-decimal">
                                {errors
                                    .iter()
                                    .map(|err| {
                                        let analysis = analyses
                                            .with_untracked(|analyses| {
                                                analyses
                                                    .as_ref()
                                                    .unwrap()
                                                    .with_untracked(|analyses| {
                                                        analyses
                                                            .iter()
                                                            .find_map(|analysis| {
                                                                analysis
                                                                    .properties()
                                                                    .with_untracked(|analysis| {
                                                                        match analysis {
                                                                            AnalysisKind::Script(script) => {
                                                                                (script.rid() == err.analysis())
                                                                                    .then_some(script.path.to_string_lossy().to_string())
                                                                            }
                                                                            AnalysisKind::ExcelTemplate(template) => {
                                                                                (template.rid() == err.analysis())
                                                                                    .then_some(
                                                                                        template.template.path.to_string_lossy().to_string(),
                                                                                    )
                                                                            }
                                                                        }
                                                                    })
                                                            })
                                                            .unwrap()
                                                    })
                                            });
                                        let container = graph.find_by_id(err.container()).unwrap();
                                        let container = graph
                                            .path(&container)
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string();
                                        let stderr = err
                                            .output()
                                            .map_or(
                                                "Could not retrieve error message".to_string(),
                                                |output| {
                                                    String::from_utf8(output.stderr.clone()).unwrap()
                                                },
                                            );
                                        view! {
                                            <li class="pb-4">
                                                <div>
                                                    <strong>{analysis}</strong>
                                                    " running on "
                                                    <strong>{container}</strong>
                                                </div>
                                                ": "
                                                <div>{stderr}</div>
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ol>
                        });
                        messages.update(|messages| messages.push(msg.build()));
                    }
                    break;
                }
            }
        }
    }

    async fn cancel_analysis() {
        tauri_sys::core::invoke::<()>("cancel_analysis", ()).await
    }

    async fn kill_analysis() {
        tauri_sys::core::invoke::<()>("kill_analysis", ()).await
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
    project: &state::Project,
    graph: &state::Graph,
    messages: types::Messages,
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
    project: &state::Project,
    graph: &state::Graph,
    messages: types::Messages,
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
    graph: &state::Graph,
    messages: types::Messages,
) {
    use super::super::super::common::FS_RESOURCE_ACTION_NOTIFY_THRESHOLD;

    let container_node = graph.find_by_id(&container).unwrap();
    let container_path = graph.path(&container_node).unwrap();

    let transfer_size = match commands::fs::file_size(payload.paths().clone())
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
        let msg = types::message::Builder::info("Transferring files.");
        let msg = msg.build();
        messages.update(|messages| {
            messages.push(msg);
        });
    }

    match add_fs_resources_to_graph(project, container_path, payload.paths().clone()).await {
        Ok(_) => {
            if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
                let msg = types::message::Builder::success("File transfer complete.");
                let msg = msg.build();
                messages.update(|messages| {
                    messages.push(msg);
                });
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
    messages: types::Messages,
) {
    use super::super::super::common::FS_RESOURCE_ACTION_NOTIFY_THRESHOLD;

    let transfer_size = match commands::fs::file_size(payload.paths().clone()).await {
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
        let msg = types::message::Builder::info("Adding analyses.");
        let msg = msg.build();
        messages.update(|messages| messages.push(msg));
    }

    match add_fs_resources_to_analyses(payload.paths().clone(), project).await {
        Ok(_) => {
            if transfer_size > FS_RESOURCE_ACTION_NOTIFY_THRESHOLD {
                let msg = types::message::Builder::success("Analyses added.");
                let msg = msg.build();
                messages.update(|messages| messages.push(msg));
            }
        }
        Err(err) => {
            let mut msg = types::message::Builder::error("Could not add analyses.");
            msg.body(format!("{err:?}"));
            let msg = msg.build();
            messages.update(|messages| messages.push(msg));
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
