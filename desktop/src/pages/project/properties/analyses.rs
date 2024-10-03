use super::super::workspace::{DragOverWorkspaceResource, WorkspaceResource};
use crate::{
    commands, common,
    components::message::Builder as Message,
    pages::project::{actions, state},
    types,
};
use futures::StreamExt;
use leptos::{
    ev::{DragEvent, MouseEvent},
    *,
};
use leptos_icons::Icon;
use serde::Serialize;
use std::{path::PathBuf, rc::Rc};
use syre_core::{self as core, types::ResourceId};
use syre_desktop_lib as lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_local_database as db;
use tauri_sys::{core::Channel, menu};

/// Context menu for analyses that are `Ok`.
#[derive(derive_more::Deref, Clone)]
struct ContextMenuAnalysesOk(Rc<menu::Menu>);
impl ContextMenuAnalysesOk {
    pub fn new(menu: Rc<menu::Menu>) -> Self {
        Self(menu)
    }
}

/// Active analysis for the analysis context menu.
#[derive(derive_more::Deref, derive_more::From, Clone)]
struct ContextMenuActiveAnalysis(ResourceId);

#[component]
pub fn Editor() -> impl IntoView {
    let project = expect_context::<state::Project>();

    move || {
        project.analyses().with(|analyses| match analyses {
            db::state::DataResource::Ok(analyses) => {
                view! { <AnalysesOk analyses=analyses.read_only() /> }
            }

            db::state::DataResource::Err(err) => view! { <AnalysesErr error=err.clone() /> },
        })
    }
}

#[component]
fn AnalysesErr(error: local::error::IoSerde) -> impl IntoView {
    view! {
        <div>
            <h3>"Analyses"</h3>
            <div>
                "Analyses can not be loaded" <div>
                    <small>{move || format!("{error:?}")}</small>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AnalysesOk(analyses: ReadSignal<Vec<state::project::Analysis>>) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let messages = expect_context::<types::Messages>();
    let drag_over_workspace_resource = expect_context::<Signal<DragOverWorkspaceResource>>();

    let context_menu_active_analysis = create_rw_signal::<Option<ContextMenuActiveAnalysis>>(None);
    provide_context(context_menu_active_analysis.clone());

    let highlight = move || {
        drag_over_workspace_resource
            .with(|resource| matches!(resource.as_ref(), Some(WorkspaceResource::Analyses)))
    };

    let context_menu_analyses_ok = create_local_resource(|| (), {
        let project = project.clone();
        let messages = messages.clone();

        move |_| {
            let project = project.clone();
            let messages = messages.clone();
            async move {
                let mut analysis_open = tauri_sys::menu::item::MenuItemOptions::new("Open");
                analysis_open.set_id("analyses:open");

                let (menu, mut listeners) = menu::Menu::with_id_and_items(
                    "analyses:context_menu",
                    vec![analysis_open.into()],
                )
                .await;

                spawn_local({
                    let analysis_open = listeners.pop().unwrap().unwrap();
                    handle_context_menu_analyses_events(
                        project,
                        messages,
                        context_menu_active_analysis.read_only(),
                        analysis_open,
                    )
                });

                Rc::new(menu)
            }
        }
    });

    view! {
        <div
            class=(["border-4", "border-blue-400"], highlight)
            class="h-full overflow-x-hidden overflow-y-auto"
        >
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Analyses"</h3>
            </div>
            <div class="px-1">
                <Suspense fallback=move || {
                    view! { <AnalysesLoading /> }
                }>
                    {move || {
                        let Some(context_menu_analyses_ok) = context_menu_analyses_ok.get() else {
                            return None;
                        };
                        Some(view! { <AnalysesOkView analyses context_menu_analyses_ok /> })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn AnalysesLoading() -> impl IntoView {
    view! { <div class="text-center">"Loading analyses"</div> }
}

#[component]
fn AnalysesOkView(
    analyses: ReadSignal<Vec<state::project::Analysis>>,
    context_menu_analyses_ok: Rc<menu::Menu>,
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

#[component]
fn NoAnalyses() -> impl IntoView {
    view! { <div class="text-center">"(no analyses)"</div> }
}

#[component]
fn Analysis(analysis: state::project::Analysis) -> impl IntoView {
    move || {
        analysis.properties().with(|properties| match properties {
            AnalysisKind::Script(_) => {
                view! { <ScriptView analysis=analysis.clone() /> }
            }
            AnalysisKind::ExcelTemplate(template) => {
                view! { <ExcelTemplateView template=template.clone() /> }
            }
        })
    }
}

#[component]
fn ScriptView(analysis: state::project::Analysis) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let messages = expect_context::<types::Messages>();
    let context_menu = expect_context::<ContextMenuAnalysesOk>();
    let context_menu_active_analysis =
        expect_context::<RwSignal<Option<ContextMenuActiveAnalysis>>>();

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
                    common::APPLICATION_JSON,
                    &serde_json::to_string(&actions::container::Action::AddAnalysisAssociation(
                        script_id,
                    ))
                    .unwrap(),
                )
                .unwrap();
        }
    };

    let remove_analysis = {
        let fs_resource = analysis.fs_resource().clone();
        let script = script.clone();
        let project = project.clone();
        let messages = messages.clone();
        move |e: MouseEvent| {
            if e.button() != types::MouseButton::Primary {
                return;
            }

            let script_id = script().rid().clone();
            let path = project.analyses().with_untracked(|analyses_state| {
                analyses_state.as_ref().unwrap().with_untracked(|analyses| {
                    analyses
                        .iter()
                        .find_map(|analysis| {
                            analysis.properties().with(|properties| {
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
                    tracing::error!(?err);
                    let msg = match err {
                        AnalysesUpdate::AnalysesFile(err) => {
                            let mut msg = Message::error("Could not save container.");
                            msg.body(format!("{err:?}"));
                            msg
                        }
                        AnalysesUpdate::RemoveFile(err) => {
                            let mut msg = Message::error("Could not remove analysis file.");
                            msg.body(format!("{err:?}"));
                            msg
                        }
                    };
                    messages.update(|messages| messages.push(msg.build()));
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
            <span on:contextmenu=contextmenu on:dragstart=dragstart draggable="true" class="grow">

                {title}
            </span>
            <span>
                <button
                    type="button"
                    title=absent_title
                    on:mousedown=remove_analysis
                    class="aspect-square h-full rounded-sm hover:bg-secondary-200 dark:hover:bg-secondary-700"
                >
                    <Icon icon=icondata::AiMinusOutlined />
                </button>
            </span>
        </div>
    }
}

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
    project: state::Project,
    messages: types::Messages,
    context_menu_active_analysis: ReadSignal<Option<ContextMenuActiveAnalysis>>,
    analysis_open: Channel<String>,
) {
    let mut analysis_open = analysis_open.fuse();
    loop {
        futures::select! {
            event = analysis_open.next() => match event {
                None => continue,
                Some(_id) => {
                    let analysis_root = project
                        .path()
                        .get_untracked()
                        .join(project.properties().analysis_root().get_untracked().unwrap());

                    let analysis = context_menu_active_analysis.get_untracked().unwrap();
                    let analysis_path = project.analyses().with_untracked(|analyses| {
                        let db::state::DataResource::Ok(analyses) = analyses else {
                            panic!("invalid state");
                        };

                        analyses.with_untracked(|analyses| {
                            analyses.iter().find_map(|analysis_state| {
                                analysis_state.properties().with_untracked(|analysis_kind| match analysis_kind {
                                 AnalysisKind::Script(script) => {
                                    if script.rid() == &*analysis {
                                        Some(script.path.clone())
                                    } else {
                                        None
                                    }
                                 },
                                 AnalysisKind::ExcelTemplate(template) => {
                                    if template.rid() == &*analysis {
                                        Some(template.template.path.clone())
                                    } else {
                                        None
                                    }
                                 },
                                })

                            }).unwrap()
                        })
                    });
                    let path = analysis_root.join(analysis_path);

                    if let Err(err) = commands::fs::open_file(path)
                        .await {
                            messages.update(|messages|{
                                let mut msg = Message::error("Could not open analysis file.");
                                msg.body(format!("{err:?}"));
                            messages.push(msg.build());
                        });
                    }
            }
            }
        }
    }
}
