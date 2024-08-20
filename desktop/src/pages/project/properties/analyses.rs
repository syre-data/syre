use super::super::workspace::{DragOverWorkspaceResource, WorkspaceResource};
use crate::{
    common,
    pages::project::{actions, state},
};
use leptos::{ev::DragEvent, *};
use syre_core as core;
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
            AnalysisKind::Script(script) => {
                view! { <ScriptView script=script.clone()/> }
            }
            AnalysisKind::ExcelTemplate(template) => {
                view! { <ExcelTemplateView template=template.clone()/> }
            }
        })
    }
}

#[component]
fn ScriptView(script: core::project::Script) -> impl IntoView {
    let title = {
        let script = script.clone();
        move || {
            if let Some(ref name) = script.name {
                name.clone()
            } else {
                script.path.to_string_lossy().to_string()
            }
        }
    };

    let dragstart = {
        let script = script.rid().clone();
        move |e: DragEvent| {
            let data_transfer = e.data_transfer().unwrap();
            data_transfer.clear_data().unwrap();
            data_transfer
                .set_data(
                    common::APPLICATION_JSON,
                    &serde_json::to_string(&actions::container::Action::AddAnalysisAssociation(
                        script.clone(),
                    ))
                    .unwrap(),
                )
                .unwrap();
        }
    };

    view! {
        <div class="cursor-pointer" on:dragstart=dragstart draggable="true">
            {title}
        </div>
    }
}

#[component]
fn ExcelTemplateView(template: core::project::ExcelTemplate) -> impl IntoView {
    view! { <div>"template"</div> }
}
