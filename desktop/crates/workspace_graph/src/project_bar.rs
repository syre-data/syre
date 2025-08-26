use super::properties;
use leptos::{ev::MouseEvent, prelude::*};
use leptos_icons::Icon;
use syre_desktop_ui_lib as ui_lib;
use wasm_bindgen::{JsCast, closure::Closure};

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn ProjectBar() -> impl IntoView {
    view! {
        <div class="flex px-2 py-1">
            <div class="w-1/3 inline-flex gap-2">
                <PreviewSelector />
                <analyze::Analyze />
            </div>
            <div class="w-1/3 text-center">
                <ProjectInfo />
            </div>
            <div class="w-1/3 text-right">
                <Controls />
            </div>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn PreviewSelector() -> impl IntoView {
    const MENU_ID: &str = "graph-workspace-preview-menu";

    let workspace_state = expect_context::<ui_lib::state::Workspace>();
    let state = workspace_state.preview().clone();
    let (active, set_active) = signal_local::<Option<Closure<dyn FnMut(MouseEvent)>>>(None);

    let preview_list = move || {
        let mut out = vec![];
        state.with(|state| {
            if state.assets {
                out.push("Data");
            }
            if state.analyses {
                out.push("Analyses");
            }
            if state.kind {
                out.push("Type");
            }
            if state.description {
                out.push("Description");
            }
            if state.tags {
                out.push("Tags");
            }
            if state.metadata {
                out.push("Metadata");
            }
        });

        if out.is_empty() {
            "(no preview)".to_string()
        } else {
            out.join(", ").to_string()
        }
    };

    let activate = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }
        if active.read().is_some() {
            return;
        }
        e.stop_propagation();

        let cb: Closure<dyn FnMut(MouseEvent)> = Closure::new(move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            let target = e.target().unwrap();
            if let Some(target) = target.dyn_ref::<web_sys::HtmlElement>() {
                if target.closest(&format!("#{MENU_ID}")).unwrap().is_some() {
                    return;
                }
            } else if let Some(target) = target.dyn_ref::<web_sys::SvgElement>() {
                if target.closest(&format!("#{MENU_ID}")).unwrap().is_some() {
                    return;
                }
            };

            let window = web_sys::window().unwrap();
            active.with_untracked(|active| {
                let cb = active.as_ref().unwrap();
                window
                    .remove_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
                    .unwrap();
            });

            set_active.write().take();
        });

        let window = web_sys::window().unwrap();
        window
            .add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
            .unwrap();

        let _ = set_active.write().insert(cb);
    };

    let clear = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        state.write().clear();
    };

    const CLASS_FORM_DIV: &str = "px-2 w-full";
    const CLASS_CHECKBOX: &str = "w-4 h-4 rounded-sm";
    const CLASS_LABEL: &str = "pl-2";
    view! {
        <div class="relative z-10">
            <div
                on:mousedown=activate
                class=("rounded-b-none", move || active.read().is_some())
                class="cursor-pointer inline-flex w-40 px-2 rounded-sm border \
                border-secondary-600 dark:border-secondary-200"
            >
                <span class="grow truncate">{preview_list}</span>
                <span class="pl-2 inline-flex items-center">
                    <Icon icon=icondata::FaChevronDownSolid />
                </span>
            </div>
            <div
                id=MENU_ID
                class:hidden=move || active.read().is_none()
                class="absolute w-40 rounded-b bg-white dark:bg-secondary-900 border \
                border-t-0 border-secondary-600 dark:border-secondary-200"
            >
                <form on:submit=move |e| e.prevent_default()>
                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="assets"
                                on:input=move |_| {
                                    state.update(|state| state.assets = !state.assets)
                                }
                                prop:checked=move || state.with(|state| state.assets)
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Data"</span>
                        </label>
                    </div>

                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="analyses"
                                on:input=move |_| {
                                    state.update(|state| state.analyses = !state.analyses)
                                }
                                prop:checked=move || { state.with(|state| state.analyses) }
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Analyses"</span>
                        </label>
                    </div>

                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="kind"
                                on:input=move |_| { state.update(|state| state.kind = !state.kind) }
                                prop:checked=move || state.with(|state| state.kind)
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Type"</span>
                        </label>
                    </div>

                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="description"
                                on:input=move |_| {
                                    state.update(|state| state.description = !state.description)
                                }
                                prop:checked=move || { state.with(|state| state.description) }
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Description"</span>
                        </label>
                    </div>

                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="tags"
                                on:input=move |_| { state.update(|state| state.tags = !state.tags) }
                                prop:checked=move || state.with(|state| state.tags)
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Tags"</span>
                        </label>
                    </div>

                    <div class=CLASS_FORM_DIV>
                        <label class="cursor-pointer">
                            <input
                                type="checkbox"
                                name="metadata"
                                on:input=move |_| {
                                    state.update(|state| state.metadata = !state.metadata)
                                }
                                prop:checked=move || { state.with(|state| state.metadata) }
                                class=CLASS_CHECKBOX
                            />
                            <span class=CLASS_LABEL>"Metadata"</span>
                        </label>
                    </div>
                    <hr class="border-secondary-900 dark:border-secondary-200" />
                    <div class="px-2 text-center dark:border-secondary-200">
                        <button on:mousedown=clear class="w-full h-full cursor-pointer">
                            "Clear"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn ProjectInfo() -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let properties_editor = expect_context::<RwSignal<properties::EditorKind>>();

    let mousedown = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        if properties_editor.with(|editor| matches!(*editor, properties::EditorKind::Project)) {
            // TODO: Return properties to widget based on graph selection.
            // Currenlty the graph and selection state contexts are descendants, so can not access them.
            properties_editor.set(properties::EditorKind::Analyses.into());
        } else {
            properties_editor.set(properties::EditorKind::Project.into());
        }
    };

    view! {
        <div on:mousedown=mousedown class="grow text-center font-primary cursor-pointer">
            {project.properties().name()}
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn Controls() -> impl IntoView {
    const COMMAND_BUTTON_CLASS: &str = "btn-secondary p-1 rounded-xs cursor-pointer";

    let data_view = expect_context::<RwSignal<ui_lib::types::DataView>>();

    let toggle_data_view = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        data_view.set(ui_lib::types::DataView::Database)
    };

    let refresh = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        let window = web_sys::window().unwrap();
        window.location().reload().unwrap();
    };

    view! {
        <ol class="flex gap-1 justify-end">
            <li>
                <button
                    on:mousedown=toggle_data_view
                    type="button"
                    class=COMMAND_BUTTON_CLASS
                    title="Toggle data view"
                >
                    <Icon icon=icondata::VsBrowser />
                </button>
            </li>
            <li>
                <button
                    on:mousedown=refresh
                    type="button"
                    class=COMMAND_BUTTON_CLASS
                    title="Refresh"
                >
                    <Icon icon=ui_lib::icon::Refresh />
                </button>
            </li>
        </ol>
    }
}

mod analyze {
    use futures::stream::StreamExt;
    use leptos::{ev::MouseEvent, prelude::*, task::spawn_local};
    use leptos_icons::*;
    use reactive_stores::Store;
    use std::path::PathBuf;
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_desktop_ui_lib::{
        self as ui_lib,
        state::settings::{
            project::SettingsStoreFields as ProjectSettingsStoreFields,
            user::SettingsStoreFields as UserSettingsStoreFields,
        },
    };
    use syre_local::types::AnalysisKind;
    use syre_project_daemon as db;

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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    #[component]
    pub fn Analyze() -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let messages = expect_context::<ui_lib::message::Messages>();
        let user_settings = expect_context::<Store<ui_lib::state::settings::User>>();
        let project_settings = expect_context::<Store<ui_lib::state::settings::Project>>();
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
                                let msg = ui_lib::message::Builder::error(
                                    "Could not initialize analysis.",
                                );
                                let msg = msg.body(format!("{err:?}"));
                                messages.push_message(msg.build_str());
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    #[component]
    fn Trigger(action: Action<PathBuf, ()>) -> impl IntoView {
        let workspace_graph_state = expect_context::<ui_lib::state::WorkspaceGraph>();
        let graph = expect_context::<ui_lib::state::Graph>();

        let single_container_selected = {
            let selected_resources = workspace_graph_state.selection_resources().selected();
            move || {
                selected_resources.with(|resources| {
                    resources.len() == 1
                        && matches!(
                            resources[0].kind(),
                            ui_lib::state::workspace_graph::ResourceKind::Container
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
            if e.button() != ui_lib::types::MouseButton::Primary {
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    #[component]
    fn TriggerActions(action: Action<PathBuf, ()>) -> impl IntoView {
        let workspace_graph_state = expect_context::<ui_lib::state::WorkspaceGraph>();
        let graph = expect_context::<ui_lib::state::Graph>();
        let selected_resources = workspace_graph_state.selection_resources().selected();

        let trigger_analysis_from_root = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            action.dispatch(PathBuf::from("/"));
        };

        let trigger_analysis_from_container = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
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
                    <Icon icon=ui_lib::icon::ChevronDown />
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
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
                            <Icon icon=ui_lib::icon::Refresh />
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    #[component]
    fn AnalyzingActions() -> impl IntoView {
        let analysis_state = expect_context::<RwSignal<AnalysisState>>();
        let cancel_analysis = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
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
            if e.button() != ui_lib::types::MouseButton::Primary {
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
                    <Icon icon=ui_lib::icon::ChevronDown />
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
        analyses: RwSignal<db::state::DataResource<RwSignal<Vec<ui_lib::state::Analysis>>>>,
        graph: ui_lib::state::Graph,
        messages: ui_lib::message::Messages,
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
                        let msg = ui_lib::message::Builder::success("Analysis complete.");
                        messages.push_message(msg.build());
                    } else {
                        let errors = errors
                            .into_iter()
                            .map(|err| {
                                let analysis = analyses
                                    .read_untracked()
                                    .as_ref()
                                    .unwrap()
                                    .read_untracked()
                                    .iter()
                                    .find_map(|analysis| {
                                        analysis.properties().with_untracked(|analysis| {
                                            match analysis {
                                                AnalysisKind::Script(script) => {
                                                    (script.rid() == err.analysis()).then_some(
                                                        script.path.to_string_lossy().to_string(),
                                                    )
                                                }
                                                AnalysisKind::ExcelTemplate(template) => {
                                                    (template.rid() == err.analysis()).then_some(
                                                        template
                                                            .template
                                                            .path
                                                            .to_string_lossy()
                                                            .to_string(),
                                                    )
                                                }
                                            }
                                        })
                                    })
                                    .unwrap();

                                let container = graph.find_by_id(err.container()).unwrap();
                                let container = graph
                                    .path(&container)
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string();

                                let stderr = err.output().map_or(
                                    "Could not retrieve error message".to_string(),
                                    |output| String::from_utf8(output.stderr.clone()).unwrap(),
                                );

                                ErrorInfo {
                                    analysis,
                                    container,
                                    message: stderr,
                                }
                            })
                            .collect();

                        tracing::error!(?errors);
                        let msg =
                            ui_lib::message::Builder::error("Errors occurred during analysis.");
                        let msg = msg.body(UpdateErrors { errors });
                        messages.push_message(msg.build());
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

    #[derive(Clone, Debug)]
    struct ErrorInfo {
        analysis: String,
        container: String,
        message: String,
    }

    #[derive(Clone)]
    struct UpdateErrors {
        errors: Vec<ErrorInfo>,
    }
    impl ui_lib::message::AsAnyView for UpdateErrors {
        fn as_any_view(&self) -> AnyView {
            view! {
                <ol class="list-decimal">
                    {self
                        .errors
                        .iter()
                        .map(|err| {
                            view! {
                                <li class="pb-4">
                                    <div>
                                        <strong>{err.analysis.clone()}</strong>
                                        " running on "
                                        <strong>{err.container.clone()}</strong>
                                    </div>
                                    ": "
                                    <div>{err.message.clone()}</div>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()}
                </ol>
            }
            .into_any()
        }
    }
}
