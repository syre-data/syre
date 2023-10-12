//! Project canvas.
use super::details_bar::DetailsBar;
use super::graph_state;
use super::project::Project as ProjectUi;
use super::{
    canvas_state::CanvasState, graph_state::GraphState, CanvasStateReducer, GraphStateAction,
    GraphStateReducer,
};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::commands::container::UpdatePropertiesArgs as UpdateContainerPropertiesArgs;
use crate::commands::graph;
use crate::hooks::{use_load_project_scripts, use_project_graph};
use crate::routes::Route;
use futures::stream::StreamExt;
use thot_core::types::ResourceId;
use thot_local_database::events::{Container as ContainerUpdate, Project as ProjectUpdate, Update};
use thot_ui::components::{Drawer, DrawerPosition};
use thot_ui::types::Message;
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

#[tracing::instrument(level = "debug")]
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

    {
        let graph_state = graph_state.clone();
        let pid = project.rid.clone();

        use_effect_with((), move |_| {
            let graph_state = graph_state.clone();
            let pid = pid.clone();

            spawn_local(async move {
                let mut events = tauri_sys::event::listen::<thot_local_database::Update>(&format!(
                    "thot://database/update/project/{}",
                    pid
                ))
                .await
                .expect("could not create `thot://database/update/project/{rid}` listener");

                while let Some(event) = events.next().await {
                    tracing::debug!(?event);
                    let Update::Project { project, update } = event.payload else {
                        tracing::debug!("Unhandled `Update` event");
                        continue;
                    };
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
                    }
                }
            });
        });
    }

    let fallback = html! { <Loading text={"Loading project"} /> };
    Ok(html! {
        <ContextProvider<CanvasStateReducer> context={canvas_state.clone()}>
        <ContextProvider<GraphStateReducer> context={graph_state}>
        <div class={classes!("project-canvas", props.class.clone())}>
            <div class={classes!("project-canvas-content")} >
                <Suspense {fallback}>
                    <ProjectUi rid={props.project.clone()} />
                </Suspense>
            </div>
            <Drawer class={classes!("details-bar-drawer")}
                position={DrawerPosition::Right}
                open={show_side_bars}>

                <DetailsBar />
            </Drawer>
        </div>
        </ContextProvider<GraphStateReducer>>
        </ContextProvider<CanvasStateReducer>>
    })
}
