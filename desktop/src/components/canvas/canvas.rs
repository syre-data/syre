//! Project canvas.
use super::canvas_state::ResourceType;
use super::details_bar::DetailsBar;
use super::layers_bar::LayersBar;
use super::project::Project as ProjectUi;
use super::{
    canvas_state::CanvasState, graph_state::GraphState, CanvasStateAction, CanvasStateReducer,
    GraphStateAction, GraphStateReducer,
};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateAction, ProjectsStateReducer};
use crate::commands::container::UpdatePropertiesArgs as UpdateContainerPropertiesArgs;
use crate::constants::MESSAGE_TIMEOUT;
use crate::hooks::{use_load_project_scripts, use_project_graph};
use crate::routes::Route;
use futures::stream::StreamExt;
use thot_core::types::ResourceId;
use thot_local_database::event::{
    Analysis as AnalysisUpdate, Asset as AssetUpdate, Container as ContainerUpdate,
    Graph as GraphUpdate, Project as ProjectUpdate, Script as ScriptUpdate, Update,
};
use thot_ui::components::{drawer, Drawer, DrawerPosition};
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
    pub class: Option<Classes>,
}

#[tracing::instrument]
#[function_component(ProjectCanvas)]
pub fn project_canvas(props: &ProjectCanvasProps) -> HtmlResult {
    let show_side_bars = use_state(|| true);
    let navigator = use_navigator().expect("could not get navigator");

    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let canvas_state =
        use_reducer(|| CanvasState::new(props.project.clone(), show_side_bars.clone()));

    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project = projects_state.projects.get(&props.project);
    let Some(project) = project else {
        app_state.dispatch(AppStateAction::AddMessage(Message::error(
            "Could not load project",
        )));
        navigator.push(&Route::Dashboard);
        return Ok(html! {{ "Could not load project" }});
    };

    use_load_project_scripts(&project.rid)?;
    let graph = use_project_graph(&project.rid)?;
    let graph_state = use_reducer(|| GraphState::new(graph));

    let drawers_visible_state = use_state(|| None);

    {
        let canvas_state = canvas_state.clone();
        let projects_state = projects_state.clone();
        let graph_state = graph_state.clone();
        let pid = project.rid.clone();

        use_effect_with((), move |_| {
            let canvas_state = canvas_state.clone();
            let projects_state = projects_state.clone();
            let graph_state = graph_state.clone();
            let pid = pid.clone();

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

                    match update {
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
                                graph_state
                                    .dispatch(GraphStateAction::InsertSubtree { parent, graph });
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
                                graph_state.dispatch(GraphStateAction::RemoveSubtree(
                                    graph.root().clone(),
                                ));
                            }

                            GraphUpdate::Moved { parent, root, name } => {
                                graph_state.dispatch(GraphStateAction::MoveSubtree {
                                    parent,
                                    root,
                                    name,
                                });
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
                                graph_state
                                    .dispatch(GraphStateAction::UpdateAssetPath { asset, path });
                            }
                        },

                        ProjectUpdate::Script(update) => match update {
                            ScriptUpdate::Created(script) => {
                                projects_state.dispatch(ProjectsStateAction::InsertProjectScript {
                                    project: pid.clone(),
                                    script,
                                });
                            }

                            ScriptUpdate::Removed(script) => {
                                projects_state.dispatch(ProjectsStateAction::RemoveProjectScript(
                                    script.clone(),
                                ));

                                graph_state.dispatch(
                                    GraphStateAction::RemoveContainerScriptAssociations(script),
                                );
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
                                let name = if let Some(container) = graph_state.graph.get(&resource)
                                {
                                    container.properties.name.clone()
                                } else if let Some(container) = graph_state.asset_map.get(&resource)
                                {
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

                                let mut msg =
                                    Message::warning(format!("Resource `{name}` was flagged"));
                                msg.set_details(message);
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                            }
                        },
                    }
                }
            });
        });
    }

    {
        let canvas_state = canvas_state.clone();
        let graph_state = graph_state.clone();
        use_effect_with(graph_state, move |graph_state| {
            let mut resources = Vec::new();
            for (cid, container) in graph_state.graph.iter_nodes() {
                resources.push(cid);

                for asset in container.assets.keys() {
                    resources.push(asset);
                }
            }

            let unselect = canvas_state
                .selected
                .iter()
                .filter_map(|rid| match resources.contains(&rid) {
                    true => None,
                    false => Some(rid.clone()),
                })
                .collect::<Vec<_>>();

            canvas_state.dispatch(CanvasStateAction::UnselectMany(unselect));
        });
    }

    let onkeydown = {
        let drawers_visible_state = drawers_visible_state.clone();
        Callback::from(move |e: KeyboardEvent| {
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
        })
    };

    let fallback = html! { <Loading text={"Loading project"} /> };
    Ok(html! {
        <ContextProvider<CanvasStateReducer> context={canvas_state.clone()}>
        <ContextProvider<GraphStateReducer> context={graph_state}>
        <div class={classes!("project-canvas", props.class.clone())}
            tabIndex={"-1"}
            onkeydown={onkeydown}
            data-rid={props.project.clone()}>

            <Drawer class={classes!("layers-bar-drawer", *drawers_visible_state)}
                position={DrawerPosition::Left}
                open={show_side_bars.clone()}>

                <LayersBar />
            </Drawer>
            <div class={classes!("project-canvas-content")} >
                <Suspense {fallback}>
                    <ProjectUi rid={props.project.clone()} />
                </Suspense>
            </div>
            <Drawer class={classes!("details-bar-drawer", *drawers_visible_state)}
                position={DrawerPosition::Right}
                open={show_side_bars}>

                <DetailsBar />
            </Drawer>
        </div>
        </ContextProvider<GraphStateReducer>>
        </ContextProvider<CanvasStateReducer>>
    })
}
