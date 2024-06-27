//! Gets the canvas' `Project`.
use crate::components::canvas::CanvasStateReducer;
use syre_core::types::ResourceId;
use yew::prelude::*;

#[hook]
pub fn use_canvas_project() -> UseStateHandle<ResourceId> {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project = use_state(|| canvas_state.project.clone());

    {
        let canvas_state = canvas_state.clone();
        let project = project.clone();

        use_effect_with(canvas_state, move |canvas_state| {
            project.set(canvas_state.project.clone());
        });
    };

    project
}
