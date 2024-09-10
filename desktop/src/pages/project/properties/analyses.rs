use super::super::workspace::{DragOverWorkspaceResource, WorkspaceResource};
use crate::{
    common,
    components::message::Builder as Message,
    pages::project::{actions, state},
    types,
};
use leptos::{
    ev::{DragEvent, MouseEvent},
    *,
};
use leptos_icons::Icon;
use serde::Serialize;
use std::path::PathBuf;
use syre_core as core;
use syre_desktop_lib as lib;
use syre_local::{self as local, types::AnalysisKind};
use syre_local_database as db;

#[component]
pub fn Editor() -> impl IntoView {
    let project = expect_context::<state::Project>();

    move || {
        project.analyses().with(|analyses| match analyses {
            db::state::DataResource::Ok(analyses) => {
                view! { <AnalysesOk analyses=analyses.read_only()/> }
            }

            db::state::DataResource::Err(err) => view! { <AnalysesErr error=err.clone()/> },
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
    let drag_over_workspace_resource = expect_context::<Signal<DragOverWorkspaceResource>>();
    let highlight = move || {
        drag_over_workspace_resource
            .with(|resource| matches!(resource.as_ref(), Some(WorkspaceResource::Analyses)))
    };

    view! {
        <div class=(["border-4", "border-blue-400"], highlight) class="h-full">
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Analyses"</h3>
            </div>
            <div class="px-1">
                <Show
                    when=move || analyses.with(|analyses| !analyses.is_empty())
                    fallback=move || view! { <NoAnalyses/> }
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
                        <Analysis analysis/>
                    </For>
                </Show>
            </div>
        </div>
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
                view! { <ScriptView analysis=analysis.clone()/> }
            }
            AnalysisKind::ExcelTemplate(template) => {
                view! { <ExcelTemplateView template=template.clone()/> }
            }
        })
    }
}

#[component]
fn ScriptView(analysis: state::project::Analysis) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let messages = expect_context::<types::Messages>();

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
            if e.button() != types::MouseButton::Primary as i16 {
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

    // TODO: Indicate file presence.
    view! {
        <div class="flex cursor-pointer" on:dragstart=dragstart draggable="true">
            <span class="grow">{title}</span>
            <span>
                <button
                    type="button"
                    title=absent_title
                    on:mousedown=remove_analysis
                    class="aspect-square h-full rounded-sm hover:bg-secondary-200 dark:hover:bg-secondary-700"
                    >
                    <Icon icon=icondata::AiMinusOutlined/>
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
