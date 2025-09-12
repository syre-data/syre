use super::super::workspace::{DragOverWorkspaceResource, WorkspaceResource};
use futures::StreamExt;
use leptos::{
    either::either,
    ev::{DragEvent, MouseEvent},
    prelude::*,
    task::spawn_local,
};
use leptos_icons::Icon;
use serde::Serialize;
use std::{path::PathBuf, sync::Arc};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_lib as lib;
use syre_desktop_ui_components as components;
use syre_desktop_ui_lib as ui_lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_project_daemon as db;
use tauri_sys::{core::Channel, menu};

/// Id for the analyses properties bar.
pub const ANALYSES_ID: &'static str = "analyses";

/// Context menu for analyses that are `Ok`.
#[derive(derive_more::Deref, Clone)]
struct ContextMenuAnalysesOk(Arc<menu::Menu>);
impl ContextMenuAnalysesOk {
    pub fn new(menu: Arc<menu::Menu>) -> Self {
        Self(menu)
    }
}

/// Active analysis for the analysis context menu.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct ContextMenuActiveAnalysis(ResourceId);
impl ContextMenuActiveAnalysis {
    pub fn into_inner(self) -> ResourceId {
        self.0
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn Editor() -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let graph = expect_context::<ui_lib::state::Graph>();

    move || {
        project.analyses().with(|analyses| either!(analyses,
            db::state::DataResource::Ok(analyses) => view! { <AnalysesOk analyses=analyses.read_only() /> },
            db::state::DataResource::Err(err) => view! { <error::AnalysesError error=err.clone() /> },
        ))
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn AnalysesOk(analyses: ReadSignal<Vec<ui_lib::state::project::Analysis>>) -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let drag_over_workspace_resource = expect_context::<ReadSignal<DragOverWorkspaceResource>>();

    let context_menu_active_analysis = ArcRwSignal::<Option<ContextMenuActiveAnalysis>>::new(None);
    provide_context(context_menu_active_analysis.clone());

    let highlight = move || {
        drag_over_workspace_resource
            .with(|resource| matches!(resource.as_ref(), Some(WorkspaceResource::Analyses)))
    };

    let context_menu_analyses_ok = LocalResource::new({
        let project = project.clone();
        let messages = messages.clone();
        move || {
            let project = project.clone();
            let messages = messages.clone();
            let context_menu_active_analysis = context_menu_active_analysis.clone();
            async move {
                let mut analysis_open = tauri_sys::menu::item::MenuItemOptions::new("Open");
                analysis_open.set_id("analyses:open");

                let mut analysis_enable_all =
                    tauri_sys::menu::item::MenuItemOptions::new("Enable all");
                analysis_enable_all.set_id("analyses:enable_all");

                let mut analysis_disable_all =
                    tauri_sys::menu::item::MenuItemOptions::new("Disable all");
                analysis_disable_all.set_id("analyses:disable_all");

                let (menu, mut listeners) = menu::Menu::with_id_and_items(
                    "analyses:context_menu",
                    vec![
                        analysis_open.into(),
                        analysis_enable_all.into(),
                        analysis_disable_all.into(),
                    ],
                )
                .await;

                spawn_local({
                    let analysis_disable_all = listeners.pop().unwrap().unwrap();
                    let analysis_enable_all = listeners.pop().unwrap().unwrap();
                    let analysis_open = listeners.pop().unwrap().unwrap();
                    let context_menu_active_analysis = context_menu_active_analysis.read_only();
                    handle_context_menu_analyses_events(
                        project,
                        messages,
                        context_menu_active_analysis,
                        analysis_open,
                        analysis_enable_all,
                        analysis_disable_all,
                    )
                });

                Arc::new(menu)
            }
        }
    });

    view! {
        <div class=(["border-4", "border-blue-400"], highlight) class="flex flex-col h-full">
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Analyses"</h3>
            </div>
            <div class="px-1 grow overflow-y-auto scrollbar-thin">
                <Suspense fallback=move || {
                    view! { <AnalysesLoading /> }
                }>
                    {move || Suspend::new(async move {
                        let context_menu_analyses_ok = context_menu_analyses_ok.await;
                        view! { <AnalysesOkView analyses context_menu_analyses_ok /> }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn AnalysesLoading() -> impl IntoView {
    view! { <div class="text-center">"Loading analyses"</div> }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn AnalysesOkView(
    analyses: ReadSignal<Vec<ui_lib::state::project::Analysis>>,
    context_menu_analyses_ok: Arc<menu::Menu>,
) -> impl IntoView {
    provide_context(ContextMenuAnalysesOk::new(context_menu_analyses_ok));

    view! {
        <Show
            when=move || analyses.with(|analyses| !analyses.is_empty())
            fallback=move || view! { <NoAnalyses /> }
        >
            <For
                each=analyses
                key=|analysis| {
                    analysis
                        .properties()
                        .with_untracked(|properties| match properties {
                            AnalysisKind::Script(script) => script.rid().clone(),
                            AnalysisKind::ExcelTemplate(template) => template.rid().clone(),
                        })
                }

                let:analysis
            >
                <Analysis analysis />
            </For>
        </Show>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn NoAnalyses() -> impl IntoView {
    view! { <div class="text-center">"(no analyses)"</div> }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn Analysis(analysis: ui_lib::state::project::Analysis) -> impl IntoView {
    move || {
        analysis.properties().with(|analyses| either!(analyses,
            AnalysisKind::Script(_) => view! { <ScriptView analysis=analysis.clone() /> },
            AnalysisKind::ExcelTemplate(template) => view! { <ExcelTemplateView template=template.clone() /> },
        ))
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn ScriptView(analysis: ui_lib::state::project::Analysis) -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    let messages = expect_context::<ui_lib::message::Messages>();
    let context_menu = expect_context::<ContextMenuAnalysesOk>();
    let context_menu_active_analysis =
        expect_context::<ArcRwSignal<Option<ContextMenuActiveAnalysis>>>();

    let script = {
        let properties = analysis.properties().clone();
        move || {
            properties.with(|properties| {
                let AnalysisKind::Script(script) = properties else {
                    panic!("invalid analysis kind");
                };
                script.clone()
            })
        }
    };

    let title = {
        let script = script.clone();
        move || {
            let script = script();
            if let Some(ref name) = script.name {
                name.clone()
            } else {
                script.path.to_string_lossy().to_string()
            }
        }
    };

    let dragstart = {
        let script = script.clone();
        move |e: DragEvent| {
            let script_id = script().rid().clone();
            let data_transfer = e.data_transfer().unwrap();
            data_transfer.clear_data().unwrap();
            data_transfer
                .set_data(
                    ui_lib::common::APPLICATION_JSON,
                    &serde_json::to_string(
                        &ui_lib::types::container::Action::AddAnalysisAssociation(script_id),
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    };

    let remove_analysis = {
        let script = script.clone();
        let project = project.clone();
        let messages = messages.clone();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            let script_id = script().rid().clone();
            let path = project.analyses().with_untracked(|analyses_state| {
                analyses_state.as_ref().unwrap().with_untracked(|analyses| {
                    analyses
                        .iter()
                        .find_map(|analysis| {
                            analysis.properties().with_untracked(|properties| {
                                let AnalysisKind::Script(script) = properties else {
                                    return None;
                                };

                                if *script.rid() != script_id {
                                    return None;
                                }

                                Some(script.path.clone())
                            })
                        })
                        .unwrap()
                })
            });

            let project = project.rid().get_untracked();
            let messages = messages.clone();
            spawn_local(async move {
                use lib::command::project::error::AnalysesUpdate;

                if let Err(err) = remove_analysis(project, path).await {
                    #[cfg(feature = "tracing")] 
                    tracing::warn!("could not remove analysis association: {err:?}");
                    let msg = match err {
                        AnalysesUpdate::AnalysesFile(err) => {
                            let msg = ui_lib::message::Builder::error("Could not save container.");
                            msg.body(format!("{err:?}"))
                        }
                        AnalysesUpdate::RemoveFile(err) => {
                            let msg =
                                ui_lib::message::Builder::error("Could not remove analysis file.");
                            msg.body(format!("{err:?}"))
                        }
                    };
                    messages.push_message(msg.build_str());
                }
            });
        }
    };

    let is_present = {
        let fs_resource = analysis.fs_resource().clone();
        move || fs_resource.with(|fs_resource| fs_resource.is_present())
    };

    let absent_title = {
        let is_present = is_present.clone();
        move || {
            if !is_present() {
                "Analysis file does not exist."
            } else {
                ""
            }
        }
    };

    let contextmenu = {
        let script = script.clone();
        move |e: MouseEvent| {
            e.prevent_default();

            context_menu_active_analysis.update(|active_analysis| {
                let id = script().rid().clone();
                let _ = active_analysis.insert(id.into());
            });

            let menu = context_menu.clone();
            spawn_local(async move {
                menu.popup().await.unwrap();
            });
        }
    };

    // TODO: Indicate file presence.
    view! {
        <div class="flex cursor-pointer">
            <span
                on:contextmenu=contextmenu
                on:dragstart=dragstart
                draggable="true"
                class="grow truncate"
                title=title
            >
                {title}
            </span>
            <span>
                <button
                    type="button"
                    title=absent_title
                    on:mousedown=remove_analysis
                    class="align-middle rounded-xs hover:bg-secondary-200 \
                    dark:hover:bg-secondary-900 cursor-pointer"
                >
                    <Icon icon=ui_lib::icon::Remove />
                </button>
            </span>
        </div>
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
fn ExcelTemplateView(template: core::project::ExcelTemplate) -> impl IntoView {
    view! { <div>"template"</div> }
}

/// # Arguments
/// + `path`: Relative path from the analysis root.
async fn remove_analysis(
    project: core::types::ResourceId,
    path: PathBuf,
) -> Result<(), lib::command::project::error::AnalysesUpdate> {
    #[derive(Serialize)]
    struct Args {
        project: core::types::ResourceId,
        path: PathBuf,
    }

    tauri_sys::core::invoke_result("project_analysis_remove", Args { project, path }).await
}

async fn handle_context_menu_analyses_events(
    project: ui_lib::state::Project,
    messages: ui_lib::message::Messages,
    context_menu_active_analysis: ArcReadSignal<Option<ContextMenuActiveAnalysis>>,
    analysis_open: Channel<String>,
    analysis_enable_all: Channel<String>,
    analysis_disable_all: Channel<String>,
) {
    let mut analysis_open = analysis_open.fuse();
    let mut analysis_enable_all = analysis_enable_all.fuse();
    let mut analysis_disable_all = analysis_disable_all.fuse();
    loop {
        futures::select! {
            event = analysis_open.next() => match event {
                None => continue,
                Some(_id) => {
                    handle_context_menu_analyses_events_analysis_open(
                        &project,
                        messages,
                        context_menu_active_analysis.clone()
                    ).await;
                },
            },
            event = analysis_enable_all.next() => match event {
                None => continue,
                Some(_id) => {
                    handle_context_menu_analyses_events_analysis_enable_all(
                        &project,
                        messages,
                        context_menu_active_analysis.clone()
                    ).await;
                }
            },
            event = analysis_disable_all.next() => match event {
                None => continue,
                Some(_id) => {
                    handle_context_menu_analyses_events_analysis_disable_all(
                        &project,
                        messages,
                        context_menu_active_analysis.clone()
                    ).await;
                }
            },
            complete => break,
        }
    }
}

async fn handle_context_menu_analyses_events_analysis_open(
    project: &ui_lib::state::Project,
    messages: ui_lib::message::Messages,
    context_menu_active_analysis: ArcReadSignal<Option<ContextMenuActiveAnalysis>>,
) {
    let analysis_root = project.path().get_untracked().join(
        project
            .properties()
            .analysis_root()
            .get_untracked()
            .unwrap(),
    );

    let analysis = context_menu_active_analysis.get_untracked().unwrap();
    let analysis_path = project.analyses().with_untracked(|analyses| {
        let db::state::DataResource::Ok(analyses) = analyses else {
            panic!("invalid state");
        };

        analyses.with_untracked(|analyses| {
            analyses
                .iter()
                .find_map(|analysis_state| {
                    analysis_state.properties().with_untracked(
                        |analysis_kind| match analysis_kind {
                            AnalysisKind::Script(script) => {
                                if script.rid() == &*analysis {
                                    Some(script.path.clone())
                                } else {
                                    None
                                }
                            }
                            AnalysisKind::ExcelTemplate(template) => {
                                if template.rid() == &*analysis {
                                    Some(template.template.path.clone())
                                } else {
                                    None
                                }
                            }
                        },
                    )
                })
                .unwrap()
        })
    });
    let path = analysis_root.join(analysis_path);

    if let Err(err) = ui_lib::commands::fs::open_file(path).await {
        let msg = ui_lib::message::Builder::error("Could not open analysis file.");
        let msg = msg.body(format!("{err:?}"));
        messages.push_message(msg.build_str());
    }
}

async fn handle_context_menu_analyses_events_analysis_enable_all(
    project: &ui_lib::state::Project,
    messages: ui_lib::message::Messages,
    context_menu_active_analysis: ArcReadSignal<Option<ContextMenuActiveAnalysis>>,
) {
    let analysis = context_menu_active_analysis
        .get_untracked()
        .unwrap()
        .into_inner();

    if let Err(err) =
        analysis_toggle_all_associations(project.path().get_untracked(), analysis, true).await
    {
        use lib::command::analyses::error::ToggleSubtreeAssociations;
        match err {
            ToggleSubtreeAssociations::ProjectNotFound
            | ToggleSubtreeAssociations::ProjectNotPresent
            | ToggleSubtreeAssociations::InvalidProject(_)
            | ToggleSubtreeAssociations::RootNotFound => panic!("invalid project state"),

            ToggleSubtreeAssociations::Container(errors) => {
                let msg = ui_lib::message::Builder::error("Could not update all associations.");
                let msg = msg.body(UpdateErrors { errors });
                messages.push_message(msg.build());
            }
        }
    }
}

async fn handle_context_menu_analyses_events_analysis_disable_all(
    project: &ui_lib::state::Project,
    messages: ui_lib::message::Messages,
    context_menu_active_analysis: ArcReadSignal<Option<ContextMenuActiveAnalysis>>,
) {
    let analysis = context_menu_active_analysis
        .get_untracked()
        .unwrap()
        .into_inner();

    if let Err(err) =
        analysis_toggle_all_associations(project.path().get_untracked(), analysis, false).await
    {
        use lib::command::analyses::error::ToggleSubtreeAssociations;
        match err {
            ToggleSubtreeAssociations::ProjectNotFound
            | ToggleSubtreeAssociations::ProjectNotPresent
            | ToggleSubtreeAssociations::InvalidProject(_)
            | ToggleSubtreeAssociations::RootNotFound => panic!("invalid project state"),

            ToggleSubtreeAssociations::Container(errors) => {
                let msg = ui_lib::message::Builder::error("Could not update all associations.");
                let msg = msg.body(UpdateErrors { errors });
                messages.push_message(msg.build());
            }
        }
    }
}

async fn analysis_toggle_all_associations(
    project: PathBuf,
    analysis: ResourceId,
    enable: bool,
) -> Result<(), lib::command::analyses::error::ToggleSubtreeAssociations> {
    #[derive(Serialize)]
    struct Args {
        project: PathBuf,
        root: PathBuf,
        analysis: ResourceId,
        enable: bool,
    }

    tauri_sys::core::invoke_result(
        "analysis_toggle_associations",
        Args {
            project,
            root: PathBuf::from("/"),
            analysis,
            enable,
        },
    )
    .await
}

#[derive(Clone)]
struct UpdateErrors {
    errors: Vec<(PathBuf, local::error::IoSerde)>,
}
impl ui_lib::message::AsAnyView for UpdateErrors {
    fn as_any_view(&self) -> AnyView {
        view! {
            <ul>
                {self
                    .errors
                    .iter()
                    .map(|(path, err)| view! { <li>{format!("{path:?}: {err:?}")}</li> })
                    .collect::<Vec<_>>()}
            </ul>
        }
        .into_any()
    }
}

mod error {
    use leptos::{ev::MouseEvent, html, prelude::*};
    use serde::Serialize;
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_desktop_ui_components as components;
    use syre_desktop_ui_lib as ui_lib;
    use syre_local as local;

    #[derive(Clone, Copy, Debug)]
    enum ConfirmationAction {
        Confirm,
        Cancel,
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    #[component]
    pub fn AnalysesError(error: local::error::IoSerde) -> impl IntoView {
        let project = expect_context::<ui_lib::state::Project>();

        let clear_associations_modal = NodeRef::<html::Dialog>::new();
        let (confirmation_action, set_confirmation_action) =
            signal(Option::<ConfirmationAction>::None);

        let show_clear_associations_confirmation = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            let dialog = clear_associations_modal.get_untracked().unwrap();
            dialog.show_modal().unwrap();
        };

        let clear_associations = Action::new_local({
            let project = project.rid().read_only();
            move |_| async move {
                if let Err(err) = clear_all_analyses(project.get_untracked()).await {
                    todo!("{err:?}");
                }
            }
        });

        Effect::watch(
            confirmation_action,
            move |confirmation_action, _, _| {
                let Some(action) = confirmation_action else {
                    return;
                };

                let dialog = clear_associations_modal.get_untracked().unwrap();
                dialog.close();

                if matches!(action, ConfirmationAction::Confirm) {
                    clear_associations.dispatch(());
                }
            },
            false,
        );

        view! {
            <div class="px-1">
                <h3>"Analyses"</h3>
                <div>
                    <div class="text-sm/4 text-secondary-400">
                        "Analyses could not be loaded" <div>
                            <small>{move || format!("{error:?}")}</small>
                        </div>
                    </div>
                    <div class="pt-4">
                        <button
                            class="btn btn-secondary text-sm"
                            on:mousedown=show_clear_associations_confirmation
                            disabled=clear_associations.pending()
                        >
                            "Clear all associations"
                        </button>
                    </div>
                </div>
            </div>
            <components::ModalDialog node_ref=clear_associations_modal>
                <ClearAssociationsConfirmation on_action=set_confirmation_action />
            </components::ModalDialog>
        }
    }

    #[component]
    fn ClearAssociationsConfirmation(
        on_action: WriteSignal<Option<ConfirmationAction>>,
    ) -> impl IntoView {
        let confirm = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            on_action.set(Some(ConfirmationAction::Confirm))
        };

        let close = move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            on_action.set(Some(ConfirmationAction::Cancel))
        };

        view! {
            <div class="bg-white border border-black rounded dark:bg-secondary-800 \
            dark:border-secondary-400 dark:text-white px-4 py-2">
                <div class="text-2xl pb-2">
                    "Are you sure you want to clear all analysis associations?"
                </div>
                <div class="pb-2">
                    <div class="flex gap-4 justify-center">
                        <button
                            on:mousedown=confirm
                            class="btn bg;syre-red-700 dark:bg-syre-red-500"
                        >
                            "Confirm"
                        </button>
                        <button on:mousedown=close class="btn btn-secondary">
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>
        }
    }

    pub async fn clear_all_analyses(
        project: ResourceId,
    ) -> Result<(), lib::command::container::error::RemoveProjectAnalysisAssociations> {
        #[derive(Serialize)]
        struct Args {
            project: ResourceId,
        }

        tauri_sys::core::invoke_result::<
            (),
            lib::command::container::error::RemoveProjectAnalysisAssociations,
        >("remove_all_project_analysis_associations", Args { project })
        .await
    }
}
