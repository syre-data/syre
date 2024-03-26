use super::properties_bar::PropertiesBarWidget;
use super::{CanvasStateAction, CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::graph::{get_or_load_project_graph, load_project_graph};
use crate::commands::project::analyze;
use crate::common::DisplayName;
use crate::components::canvas::canvas_state::ResourceType;
use crate::constants::MESSAGE_TIMEOUT;
use crate::hooks::use_project;
use std::io;
use syre_core::error::Runner as RunnerError;
use syre_core::types::ResourceId;
use syre_desktop_lib::error::Analysis as AnalysisError;
use syre_local::types::AnalysisKind;
use syre_local_database::error::server::LoadProjectGraph;
use syre_ui::types::ContainerPreview;
use syre_ui::types::Message;
use syre_ui::widgets::container::container_tree::ContainerPreviewSelect;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

const ICON_SIZE: u32 = 16;

#[derive(PartialEq)]
enum AnalysisState {
    Standby,
    Analyzing,
    Complete,
    Paused,
    Error,
}

#[derive(Properties, PartialEq, Debug)]
pub struct ProjectControlsProps {
    pub project: ResourceId,
}

#[function_component(ProjectControls)]
pub fn project_controls(props: &ProjectControlsProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let analysis_state = use_state(|| AnalysisState::Standby);
    let show_analyze_options = use_state(|| false);
    let reloading_project_graph_state = use_state(|| false);

    let project = use_project(&props.project);
    let project = project.as_ref().unwrap();

    let set_preview = use_callback((), {
        let canvas_state = canvas_state.dispatcher();
        move |preview: ContainerPreview, _| {
            canvas_state.dispatch(CanvasStateAction::SetPreview(preview));
        }
    });

    let onclick_project_title = use_callback((), {
        let canvas_state = canvas_state.dispatcher();
        move |e: MouseEvent, _| {
            e.stop_propagation();
            canvas_state.dispatch(CanvasStateAction::SetPropertiesBarWidget(
                PropertiesBarWidget::ProjectActions,
            ))
        }
    });

    let analyze_cb = use_callback(
        (
            app_state.clone(),
            projects_state.clone(),
            canvas_state.clone(),
            graph_state.clone(),
        ),
        {
            let analysis_state = analysis_state.setter();
            move |_: MouseEvent, (app_state, projects_state, canvas_state, graph_state)| {
                let app_state = app_state.clone();
                let projects_state = projects_state.clone();
                let graph_state = graph_state.clone();
                let analysis_state = analysis_state.clone();
                let project_id = canvas_state.project.clone();

                canvas_state.dispatch(CanvasStateAction::ClearFlags);

                spawn_local(async move {
                    let root = graph_state.graph.root();

                    analysis_state.set(AnalysisState::Analyzing);
                    app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                        Message::info("Running analysis"),
                        MESSAGE_TIMEOUT,
                        app_state.clone(),
                    ));

                    let analysis_result = analyze(root.clone()).await;

                    // update tree
                    let update = match get_or_load_project_graph(project_id.clone()).await {
                        Ok(graph) => graph,
                        Err(err) => {
                            tracing::debug!(?err);
                            panic!("{err:?}");
                        }
                    };

                    graph_state.dispatch(GraphStateAction::SetGraph(update));
                    analysis_state.set(AnalysisState::Complete);

                    match analysis_result {
                        Ok(_) => {
                            app_state.dispatch(AppStateAction::AddMessage(Message::success(
                                "Analysis complete",
                            )));
                        }
                        Err(err) => {
                            tracing::debug!(?err);
                            let details = detail_message_from_analysis_error(
                                err,
                                &project_id,
                                &graph_state,
                                &projects_state,
                            );

                            let mut msg = Message::error("Error while analyzing");
                            msg.set_details(details);
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                        }
                    }
                })
            }
        },
    );

    let analyze_container = use_callback(
        (
            app_state.clone(),
            projects_state.clone(),
            canvas_state.clone(),
            graph_state.clone(),
        ),
        {
            let analysis_state = analysis_state.setter();
            move |_: MouseEvent, (app_state, projects_state, canvas_state, graph_state)| {
                let app_state = app_state.clone();
                let projects_state = projects_state.clone();
                let graph_state = graph_state.clone();
                let analysis_state = analysis_state.clone();
                let project_id = canvas_state.project.clone();

                let selected = canvas_state.selected.clone();
                let selected_rid = selected
                    .iter()
                    .next()
                    .expect("a container should be selected")
                    .clone();

                if let Some(descendants) = graph_state.graph.descendants(&selected_rid) {
                    for descendant in descendants {
                        let descendant = graph_state.graph.get(&descendant).unwrap();
                        for asset in descendant.assets.keys() {
                            canvas_state
                                .dispatch(CanvasStateAction::ClearResourceFlags(asset.clone()));
                        }

                        canvas_state.dispatch(CanvasStateAction::ClearResourceFlags(
                            descendant.rid.clone(),
                        ));
                    }
                }

                spawn_local(async move {
                    let root = selected_rid;
                    analysis_state.set(AnalysisState::Analyzing);
                    app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                        Message::info("Running analysis"),
                        MESSAGE_TIMEOUT,
                        app_state.clone(),
                    ));

                    let analysis_result = analyze(root.clone()).await;

                    // update tree
                    let update = match get_or_load_project_graph(project_id.clone()).await {
                        Ok(graph) => graph,
                        Err(err) => {
                            tracing::debug!(?err);
                            panic!("{err:?}");
                        }
                    };

                    graph_state.dispatch(GraphStateAction::SetGraph(update));
                    analysis_state.set(AnalysisState::Complete);

                    match analysis_result {
                        Ok(_) => {
                            app_state.dispatch(AppStateAction::AddMessage(Message::success(
                                "Analysis complete",
                            )));
                        }
                        Err(err) => {
                            let details = detail_message_from_analysis_error(
                                err,
                                &project_id,
                                &graph_state,
                                &projects_state,
                            );

                            let mut msg = Message::error("Error while analyzing");
                            msg.set_details(details);
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                        }
                    }
                })
            }
        },
    );

    use_effect_with(canvas_state.clone(), {
        let show_analyze_options = show_analyze_options.clone();
        move |canvas_state| {
            if canvas_state.selected.len() != 1 {
                show_analyze_options.set(false);
                return;
            }
            let item = canvas_state
                .selected
                .iter()
                .next()
                .clone()
                .expect("selected has 1 item");

            let item_type = canvas_state
                .resource_type(item)
                .expect("item should have type");

            if item_type != ResourceType::Container {
                show_analyze_options.set(false);
                return;
            }
            show_analyze_options.set(true);
        }
    });

    let reload_project_graph = use_callback((props.project.clone(), app_state.clone()), {
        let canvas_state = canvas_state.dispatcher();
        let graph_state = graph_state.dispatcher();
        let reloading_project_graph_state = reloading_project_graph_state.setter();

        move |_: MouseEvent, (project, app_state)| {
            let app_state = app_state.clone();
            let canvas_state = canvas_state.clone();
            let graph_state = graph_state.clone();
            let project = project.clone();
            let reloading_project_graph_state = reloading_project_graph_state.clone();

            spawn_local(async move {
                reloading_project_graph_state.set(true);
                match load_project_graph(project).await {
                    Ok(graph) => {
                        graph_state.dispatch(GraphStateAction::SetGraph(graph));
                        app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                            Message::success("Graph reloaded."),
                            MESSAGE_TIMEOUT,
                            app_state.clone(),
                        ));
                    }

                    Err(LoadProjectGraph::InsertAssets { errors, graph }) => {
                        tracing::error!(?errors);
                        for (asset, err) in errors {
                            let message = match err {
                                io::ErrorKind::NotFound => "File not found".to_string(),
                                _ => format!("{err:?}"),
                            };

                            canvas_state.dispatch(CanvasStateAction::AddFlag {
                                resource: asset,
                                message,
                            });
                        }

                        let msg = Message::error("Asset files are missing.");
                        app_state.dispatch(AppStateAction::AddMessage(msg));

                        graph_state.dispatch(GraphStateAction::SetGraph(graph));
                        app_state.dispatch(AppStateAction::AddMessageWithTimeout(
                            Message::success("Graph reloaded."),
                            MESSAGE_TIMEOUT,
                            app_state.clone(),
                        ));
                    }

                    Err(err) => {
                        let mut msg = Message::error("Could not reload graph.");
                        msg.set_details(format!("{err:?}"));
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }

                reloading_project_graph_state.set(false);
            });
        }
    });

    let mut primary_analyze_btn_classes = classes!("btn-primary", "primary-analyze-btn");
    if *show_analyze_options {
        primary_analyze_btn_classes.push("with_options");
    }

    html! {
    <div class={"project-controls"}>
        <div class={"column-section left"}>
            <ContainerPreviewSelect onchange={set_preview} />
            <div class={"analyze-commands-group ml-xl"}>
                <button
                    class={primary_analyze_btn_classes}
                    onclick={analyze_cb.clone()}
                    disabled={*analysis_state == AnalysisState::Analyzing}>

                    { "Analyze" }
                </button>
                if *show_analyze_options && *analysis_state != AnalysisState::Analyzing {
                    <div class={classes!("dropdown")}>
                        <button class={classes!("btn-primary", "dropdown-btn")}>
                            <Icon
                                icon_id={IconId::FontAwesomeSolidAngleDown}
                                height={"12px"} />
                        </button>
                        <ul class={classes!("dropdown-content")}>
                            <li class={classes!("clickable")}
                                onclick={analyze_cb.clone()}>
                                { "Project" }
                            </li>
                            <li class={classes!("clickable")}
                                onclick={analyze_container}>
                                { "Container" }
                            </li>
                        </ul>
                    </div>
                }
            </div>
        </div>

        <div class={"column-section middle"}>
            <div class={"title clickable"}
                onclick={onclick_project_title}>

                <h1 class={classes!("title", "inline-block")}>{
                    &project.name
                }</h1>
            </div>
        </div>

        <div class={"column-section right"}>
            <button type={"button"}
                class={"reload-project-graph"}
                onclick={reload_project_graph}
                disabled={*reloading_project_graph_state}>

                <Icon icon_id={IconId::OcticonsSync24}
                    class={classes!((*reloading_project_graph_state).then(|| "spinner"))}
                    width={ICON_SIZE.to_string()}
                    height={ICON_SIZE.to_string()} />
            </button>
        </div>
    </div>
    }
}

fn detail_message_from_analysis_error(
    error: AnalysisError,
    project: &ResourceId,
    graph_state: &GraphStateReducer,
    projects_state: &ProjectsStateReducer,
) -> String {
    match error {
        AnalysisError::ZMQ(err) => {
            format!("Could not connect to database: {err:?}")
        }

        AnalysisError::GraphNotFound => "Container was not found in the graph".to_string(),

        AnalysisError::Analysis(err) => match err {
            RunnerError::LoadScripts(errs) => errs
                .iter()
                .map(|(script, description)| {
                    let s_name =
                        match analysis_name(script, project, &projects_state.project_analyses) {
                            None => format!("{script}"),
                            Some(name) => name,
                        };

                    format!("{s_name}: {description}")
                })
                .collect::<Vec<_>>()
                .join("\n"),

            RunnerError::ContainerNotFound(container) => {
                let mut ancestors = ancestor_names(&container, &graph_state);
                let container_name = if ancestors.len() == 0 {
                    format!("{container}")
                } else {
                    ancestors.reverse();
                    ancestors.join("/")
                };

                format!("Could not get {container_name}.")
            }

            RunnerError::CommandError {
                script: _,
                container,
                cmd,
            } => {
                let mut ancestors = ancestor_names(&container, &graph_state);
                let container_name = if ancestors.len() == 0 {
                    format!("{container}")
                } else {
                    ancestors.reverse();
                    ancestors.join("/")
                };

                format!("Could run command `{cmd}` on {container_name}.")
            }

            RunnerError::ScriptError {
                script,
                container,
                description,
            } => {
                let s_name = match analysis_name(&script, project, &projects_state.project_analyses)
                {
                    None => format!("{script}"),
                    Some(name) => name,
                };

                let mut ancestors = ancestor_names(&container, &graph_state);
                let container_name = if ancestors.len() == 0 {
                    format!("{container}")
                } else {
                    ancestors.reverse();
                    ancestors.join("/")
                };

                format!("Error while running `{s_name}` on {container_name}: {description}")
            }
        },
    }
}

/// Gets the names of the Container's ancestors.
fn ancestor_names(container: &ResourceId, graph_state: &GraphStateReducer) -> Vec<String> {
    graph_state
        .graph
        .ancestors(container)
        .iter()
        .map(|cid| graph_state.graph.get(cid).unwrap().properties.name.clone())
        .collect()
}

fn analysis_name(
    analysis: &ResourceId,
    project: &ResourceId,
    project_scripts: &crate::app::projects_state::ProjectAnalysesMap,
) -> Option<String> {
    match project_scripts.get(project) {
        None => None,

        Some(analyses) => match analyses.get(analysis) {
            None => None,
            Some(analysis) => Some(analysis.display_name()),
        },
    }
}
