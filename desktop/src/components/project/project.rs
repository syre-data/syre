//! Project component with suspense.
use crate::app::{AppStateAction, AppStateReducer};
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::container::ContainerTreeController;
use crate::hooks::use_project;
use crate::routes::Route;
use thot_core::types::ResourceId;
use thot_ui::types::Message;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectProps {
    pub rid: ResourceId,
}

#[function_component(Project)]
pub fn project(props: &ProjectProps) -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");

    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let navigator = use_navigator().expect("`navigator` not found");
    let project_ref = use_node_ref();
    let project = use_project(&props.rid);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let Some(root) = project.data_root.clone() else {
        app_state.dispatch(AppStateAction::AddMessage(Message::error("Data root of project not set")));
        navigator.push(&Route::Dashboard);
        return Ok(html! {{ "Could not load project" }});
    };

    let clear_selection = {
        let canvas_state = canvas_state.clone();

        Callback::from(move |_: MouseEvent| {
            canvas_state.dispatch(CanvasStateAction::ClearSelected);
        })
    };

    Ok(html! {
        <div ref={project_ref}
            class={classes!("project")}
            onclick={clear_selection} >

            <div class={classes!("header")}>
                <h1 class={classes!("title", "inline-block")}>{
                    &project.name
                }</h1>
                <span>{ "\u{2699}" }</span>
            </div>
            <div class={classes!("content")}>
                <ContainerTreeController {root} />
            </div>
        </div>

    })
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
