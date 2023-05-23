//! Project component with suspense.
use super::super::container::ContainerTreeController;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::hooks::use_project;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct ProjectProps {
    pub rid: ResourceId,
}

#[tracing::instrument(level = "debug")]
#[function_component(Project)]
pub fn project(props: &ProjectProps) -> HtmlResult {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project_ref = use_node_ref();
    let project = use_project(&props.rid);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
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
                <ContainerTreeController />
            </div>
        </div>

    })
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
