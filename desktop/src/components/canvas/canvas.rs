//! Project canvas.
use super::{
    canvas_state::CanvasState, container_tree_state::ContainerTreeState, CanvasStateReducer,
    ContainerTreeStateReducer,
};
use crate::app::{ProjectsStateAction, ProjectsStateReducer};
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
use crate::components::details_bar::DetailsBar;
use crate::components::project::Project as ProjectUi;
use crate::hooks::use_project_scripts;
use serde_wasm_bindgen as swb;
use thot_core::project::Scripts as ProjectScripts;
use thot_core::types::ResourceId;
use thot_ui::components::{Drawer, DrawerPosition};
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectCanvasProps {
    pub project: ResourceId,

    #[prop_or_default]
    pub class: Option<Classes>,
}

#[function_component(ProjectCanvas)]
pub fn project_canvas(props: &ProjectCanvasProps) -> Html {
    let show_side_bars = use_state(|| true);
    let canvas_state =
        use_reducer(|| CanvasState::new(props.project.clone(), show_side_bars.clone()));

    let tree_state = use_reducer(|| ContainerTreeState::new());

    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let scripts = use_project_scripts(props.project.clone());
    if scripts.is_none() {
        // get project scripts
        let project = props.project.clone();
        let projects_state = projects_state.clone();

        spawn_local(async move {
            let prj_scripts = invoke(
                "get_project_scripts",
                ResourceIdArgs {
                    rid: project.clone(),
                },
            )
            .await
            .expect("could not invoke `get_project_scripts`");

            let scripts: ProjectScripts = swb::from_value(prj_scripts)
                .expect("could not convert result of `get_project_scripts` to `Scripts`");

            projects_state.dispatch(ProjectsStateAction::InsertProjectScripts(project, scripts));
        })
    }

    let fallback = html! { <Loading text={"Loading project"} /> };
    html! {
        <ContextProvider<CanvasStateReducer> context={canvas_state.clone()}>
        <ContextProvider<ContainerTreeStateReducer> context={tree_state}>
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
        </ContextProvider<ContainerTreeStateReducer>>
        </ContextProvider<CanvasStateReducer>>
    }
}

#[cfg(test)]
#[path = "./canvas_test.rs"]
mod canvas_test;
