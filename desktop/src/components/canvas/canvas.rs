//! Project canvas.
use super::{
    canvas_state::CanvasState, graph_state::GraphState, CanvasStateReducer, GraphStateReducer,
};
use crate::app::{AppStateAction, AppStateReducer, ProjectsStateReducer};
use crate::components::details_bar::DetailsBar;
use crate::components::project::Project as ProjectUi;
use crate::hooks::{use_load_project_scripts, use_project_graph};
use crate::routes::Route;
use thot_core::types::ResourceId;
use thot_ui::components::{Drawer, DrawerPosition};
use thot_ui::types::Message;
use thot_ui::widgets::suspense::Loading;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectCanvasProps {
    pub project: ResourceId,

    #[prop_or_default]
    pub class: Option<Classes>,
}

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
        app_state.dispatch(AppStateAction::AddMessage(Message::error("Could not load project")));
        navigator.push(&Route::Dashboard);
        return Ok(html! {{ "Could not load project" }});
    };

    use_load_project_scripts(&project.rid)?;
    let graph = use_project_graph(&project.rid)?;
    let graph_state = use_reducer(|| GraphState::new(graph));

    let fallback = html! { <Loading text={"Loading project"} /> };
    Ok(html! {
        <ContextProvider<CanvasStateReducer> context={canvas_state.clone()}>
        <ContextProvider<GraphStateReducer> context={graph_state}>
        <div class={classes!("project-canvas", props.class.clone())}>
            // <NavBar />
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

#[cfg(test)]
#[path = "./canvas_test.rs"]
mod canvas_test;
