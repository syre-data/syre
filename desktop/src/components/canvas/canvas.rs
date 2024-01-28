//! Project canvas.
use super::details_bar::DetailsBar;
use super::layers_bar::LayersBar;
use super::project::Project as ProjectUi;
use super::{
    canvas_state::CanvasState, graph_state::GraphState, CanvasStateAction, CanvasStateDispatcher,
    CanvasStateReducer, GraphStateAction, GraphStateReducer, ProjectControls,
};
use crate::app::{
    AppStateAction, AppStateDispatcher, AppStateReducer, ProjectsStateAction,
    ProjectsStateDispatcher, ProjectsStateReducer,
};
use crate::commands::container::UpdatePropertiesArgs as UpdateContainerPropertiesArgs;
use crate::hooks::{use_load_project_graph, use_load_project_scripts};
use crate::routes::Route;
use futures::stream::StreamExt;
use std::io;
use thot_core::project::Project;
use thot_core::types::ResourceId;
use thot_local_database::error::server::LoadProjectGraph;
use thot_local_database::event::{
    Analysis as AnalysisUpdate, Asset as AssetUpdate, Container as ContainerUpdate,
    Graph as GraphUpdate, Project as ProjectUpdate, Script as ScriptUpdate, Update,
};
use thot_ui::components::{Drawer, DrawerPosition};
use thot_ui::types::Message;
use thot_ui::widgets::common::asset as asset_ui;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct ProjectCanvasProps {
    pub project: ResourceId,

    #[prop_or_default]
    pub class: Classes,
}

#[tracing::instrument]
#[function_component(ProjectCanvas)]
pub fn project_canvas(props: &ProjectCanvasProps) -> Html {
    let navigator = use_navigator().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();

    let fallback = html! { <Loading text={"Loading canvas"} /> };
    match projects_state.projects.get(&props.project) {
        Some(project) => {
            html! {
                <Suspense {fallback}>
                    <CanvasView
                        class={props.class.clone()}
                        project={project.clone()} />
                </Suspense>
            }
        }

        None => {
            // app_state.dispatch(AppStateAction::AddMessage(Message::error(
            //     "Could not load project",
            // )));
            tracing::debug!("could not load project");
            navigator.push(&Route::Dashboard);
            html! {
                <h1 class={"align-center"}>{ "Could not load project" }</h1>
            }
        }
    }
}

#[derive(Properties, PartialEq)]
struct CanvasViewProps {
    project: Project,

    #[prop_or_default]
    pub class: Classes,
}

#[function_component(CanvasView)]
fn canvas_view(props: &CanvasViewProps) -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let show_side_bars = use_state(|| true);
    let canvas_state =
        use_reducer(|| CanvasState::new(props.project.rid.clone(), show_side_bars.clone()));

    use_load_project_scripts(&props.project.rid)?;
    let (graph, asset_errors) = match use_load_project_graph(&props.project.rid)? {
        Ok(graph) => (graph, None),
        Err(LoadProjectGraph::ProjectNotFound) => {
            return Ok(html! {
                <div>
                    <h1>{ "Project not loaded" }</h1>
                </div>
            });
        }
        Err(LoadProjectGraph::Project(err)) => {
            return Ok(html! {
                <div>
                    <h1>{ "Project error" }</h1>
                    <div>{ format!("{err:?}") }</div>
                </div>
            });
        }
        Err(LoadProjectGraph::Load { errors, graph: _ }) => {
            return Ok(html! {
                <div>
                    <h1>{ "Could not load graph" }</h1>
                    <div>{ format!("{errors:?}") }</div>
                </div>
            });
        }
        Err(LoadProjectGraph::InsertContainers(errors)) => {
            return Ok(html! {
                <div>
                    <h1>{ "Could not load graph" }</h1>
                    <div>{ format!("{errors:?}") }</div>
                </div>
            });
        }
        Err(LoadProjectGraph::InsertAssets { errors, graph }) => {
            tracing::debug!(?errors);
            (graph, Some(errors))
        }
    };

    let graph_state = use_reducer(|| GraphState::new(graph));
    let drawers_visible_state = use_state(|| None);

    use_effect_with((), {
        let canvas_state = canvas_state.dispatcher();
        move |_| {
            let Some(asset_errors) = asset_errors else {
                return;
            };

            for (asset, err) in asset_errors {
                let message = match err {
                    io::ErrorKind::NotFound => "File not found".to_string(),
                    _ => format!("{err:?}"),
                };

                canvas_state.dispatch(CanvasStateAction::AddFlag {
                    resource: asset,
                    message,
                });
            }
        }
    });

    use_effect_with(graph_state.clone(), {
        let app_state = app_state.dispatcher();
        let canvas_state = canvas_state.dispatcher();
        let projects_state = projects_state.dispatcher();
        let pid = props.project.rid.clone();
        move |graph_state| {
            let graph_state = graph_state.clone();
            spawn_local(async move {
                let mut events = tauri_sys::event::listen::<thot_local_database::Update>(&format!(
                    "thot://database/update/project/{pid}"
                ))
                .await
                .expect(&format!(
                    "could not create `thot://database/update/project/{pid}` listener"
                ));

                while let Some(event) = events.next().await {
                    tracing::debug!(?event.payload);
                    let Update::Project { project, update } = event.payload;
                    assert!(project == pid);
                    handle_file_system_event(
                        update,
                        project,
                        &app_state,
                        &projects_state,
                        &canvas_state,
                        &graph_state,
                    );
                }
            });
        }
    });

    let onkeydown = use_callback(
        drawers_visible_state.clone(),
        move |e: KeyboardEvent, drawers_visible_state| {
            if !e.ctrl_key() {
                return;
            }

            if e.key() == "\\" {
                drawers_visible_state.set(if drawers_visible_state.is_some() {
                    None
                } else {
                    Some("hidden")
                });
            }
        },
    );

    let fallback = html! { <Loading text={"Loading project"} /> };
    Ok(html! {
        <ContextProvider<CanvasStateReducer> context={canvas_state.clone()}>
        <ContextProvider<GraphStateReducer> context={graph_state}>
        <div class={classes!("project-canvas", props.class.clone())}
            tabIndex={"-1"}
            onkeydown={onkeydown}
            data-rid={props.project.rid.clone()}>

            <ProjectControls project={props.project.rid.clone()} />
            <div class={"canvas"}>
                <Drawer class={classes!("layers-bar-drawer", *drawers_visible_state)}
                    position={DrawerPosition::Left}
                    open={show_side_bars.clone()}>

                    <LayersBar />
                </Drawer>

                <div class={classes!("project-canvas-content")} >
                    <Suspense {fallback}>
                        <ProjectUi rid={props.project.rid.clone()} />
                    </Suspense>
                </div>

                <Drawer class={classes!("details-bar-drawer", *drawers_visible_state)}
                    position={DrawerPosition::Right}
                    open={show_side_bars}>

                    <DetailsBar />
                </Drawer>
            </div>
        </div>
        </ContextProvider<GraphStateReducer>>
        </ContextProvider<CanvasStateReducer>>
    })
}

fn handle_file_system_event(
    update: thot_local_database::event::Project,
    project: ResourceId,
    app_state: &AppStateDispatcher,
    projects_state: &ProjectsStateDispatcher,
    canvas_state: &CanvasStateDispatcher,
    graph_state: &GraphStateReducer,
) {
    match update {
        ProjectUpdate::Moved(path) => {
            let mut msg = Message::info("Project moved.");
            msg.set_details(format!("Moved to {path:?}."));
            app_state.dispatch(AppStateAction::AddMessage(msg));
        }

        ProjectUpdate::Removed(prj) => {
            projects_state.dispatch(ProjectsStateAction::RemoveProject(project.clone()));

            let msg = match prj {
                None => Message::info("Project was removed."),
                Some(project) => Message::info(format!("Project `{}` was removed.", project.name)),
            };
            app_state.dispatch(AppStateAction::AddMessage(msg));
        }

        ProjectUpdate::Container(update) => match update {
            ContainerUpdate::Properties {
                container,
                properties,
            } => graph_state.dispatch(GraphStateAction::UpdateContainerProperties(
                UpdateContainerPropertiesArgs {
                    rid: container,
                    properties,
                },
            )),
        },

        ProjectUpdate::Graph(update) => match update {
            GraphUpdate::Created { parent, graph } => {
                graph_state.dispatch(GraphStateAction::InsertSubtree { parent, graph });
            }

            GraphUpdate::Removed(graph) => {
                let mut rids = Vec::with_capacity(graph.nodes().len());
                for (cid, container) in graph.nodes() {
                    rids.push(cid.clone());
                    for aid in container.assets.keys() {
                        rids.push(aid.clone());
                    }
                }

                canvas_state.dispatch(CanvasStateAction::RemoveMany(rids));
                graph_state.dispatch(GraphStateAction::RemoveSubtree(graph.root().clone()));
            }

            GraphUpdate::Moved { parent, root, name } => {
                graph_state.dispatch(GraphStateAction::MoveSubtree { parent, root, name });
            }
        },

        ProjectUpdate::Asset(update) => match update {
            AssetUpdate::Created { container, asset } => {
                graph_state.dispatch(GraphStateAction::InsertContainerAssets(
                    container.clone(),
                    vec![asset],
                ));
            }

            AssetUpdate::Moved {
                asset,
                container,
                path,
            } => {
                graph_state.dispatch(GraphStateAction::MoveAsset {
                    asset,
                    container,
                    path,
                });
            }

            AssetUpdate::Removed(asset) => {
                canvas_state.dispatch(CanvasStateAction::Remove(asset.clone()));
                graph_state.dispatch(GraphStateAction::RemoveAsset(asset));
            }

            AssetUpdate::PathChanged { asset, path } => {
                graph_state.dispatch(GraphStateAction::UpdateAssetPath { asset, path });
            }
        },

        ProjectUpdate::Script(update) => match update {
            ScriptUpdate::Created(script) => {
                projects_state
                    .dispatch(ProjectsStateAction::InsertProjectScript { project, script });
            }

            ScriptUpdate::Removed(script) => {
                projects_state.dispatch(ProjectsStateAction::RemoveProjectScript(script.clone()));
                graph_state.dispatch(GraphStateAction::RemoveContainerScriptAssociations(script));
            }

            ScriptUpdate::Moved { script, path } => {
                projects_state.dispatch(ProjectsStateAction::MoveProjectScript {
                    script: script.clone(),
                    path,
                });
            }
        },

        ProjectUpdate::Analysis(update) => match update {
            AnalysisUpdate::Flag { resource, message } => {
                let name = if let Some(container) = graph_state.graph.get(&resource) {
                    container.properties.name.clone()
                } else if let Some(container) = graph_state.asset_map.get(&resource) {
                    let container = graph_state.graph.get(&container).unwrap();
                    let asset = container.assets.get(&resource).unwrap();
                    asset_ui::asset_display_name(asset)
                } else {
                    tracing::debug!("could not find resource `{resource:?}`");
                    let mut msg = Message::error("Could not find resource");
                    msg.set_details("Could not find `{resource:?}` when flagging");
                    app_state.dispatch(AppStateAction::AddMessage(msg));
                    return;
                };

                canvas_state.dispatch(CanvasStateAction::AddFlag {
                    resource,
                    message: message.clone(),
                });

                let mut msg = Message::warning(format!("Resource `{name}` was flagged"));
                msg.set_details(message);
                app_state.dispatch(AppStateAction::AddMessage(msg));
            }
        },
    }
}
